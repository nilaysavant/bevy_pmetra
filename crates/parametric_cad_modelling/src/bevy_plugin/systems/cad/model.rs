use anyhow::{anyhow, Result};
use bevy::{pbr::NotShadowCaster, prelude::*, render::primitives::Aabb};
use bevy_async_task::{AsyncTaskPool, AsyncTaskStatus};
use bevy_mod_picking::{
    backends::raycast::bevy_mod_raycast::markers::NoBackfaceCulling, prelude::*, PickableBundle,
};

use crate::{
    bevy_plugin::{
        cleanup_manager::Cleanup,
        components::{
            cad::{
                BelongsToCadGeneratedRoot, CadGeneratedCursor, CadGeneratedCursorConfig,
                CadGeneratedCursorPreviousTransform, CadGeneratedCursorState, CadGeneratedMesh,
                CadGeneratedMeshOutlines, CadGeneratedMeshOutlinesState, CadGeneratedRoot,
                CadGeneratedRootSelectionState,
            },
            wire_frame::WireFrameDisplaySettings,
        },
        events::{
            cad::{GenerateCadModel, SpawnMeshesBuilder},
            cursor::{CursorPointerMoveEvent, CursorPointerOutEvent},
        },
        resources::{
            MeshesBuilderFinishedResultsMap, MeshesBuilderQueue, MeshesBuilderQueueInspector,
        },
    },
    cad_core::builders::{CadCursor, CadCursorName, CadMesh, CadMeshBuilder, CadMeshName, CadShellName, CadShellsByName, ParametricCad},
};

use super::{
    cursor::{cursor_drag_end, cursor_drag_start},
    mesh::{mesh_pointer_move, mesh_pointer_out},
};

pub fn spawn_shells_by_name_on_generate<Params: ParametricCad + Component + Clone>(
    mut commands: Commands,
    mut events: EventReader<GenerateCadModel<Params>>,
    cad_generated: Query<Entity, (With<Params>, With<CadGeneratedRoot>, Without<Cleanup>)>,
) {
    for GenerateCadModel {
        params,
        remove_existing_models,
    } in events.read()
    {
        if *remove_existing_models {
            for root_ent in cad_generated.iter() {
                // Remove root and its descendants...
                let Some(mut ent_commands) = commands.get_entity(root_ent) else {
                    continue;
                };
                ent_commands.insert(Cleanup::Recursive);
            }
        }

        // Spawn root...
        let root_ent = commands
            .spawn((
                SpatialBundle::default(),
                CadGeneratedRoot,
                CadGeneratedRootSelectionState::default(),
                params.clone(),
            ))
            .id();

        // Get the shell builders from params...
        let shells_builders = match params.shells_builders() {
            Ok(result) => result,
            Err(e) => {
                error!("shells_builders failed with error: {:?}", e);
                return;
            }
        };

        let mut shells_by_name = CadShellsByName::default();
        // Build Shells from Builders and add to common resource for later use.
        for (shell_name, shell_builder) in shells_builders.builders.iter() {
            let cad_shell = match shell_builder.build_cad_shell() {
                Ok(shell) => shell,
                Err(e) => {
                    error!(
                        "build_cad_shell for shell_name: {:?} failed, error: {:?}",
                        shell_name, e
                    );
                    continue;
                }
            };
            shells_by_name.insert(shell_name.clone(), cad_shell);
        }
        // Spawn shells by name and add to root...
        let shells_by_name_ent = commands
            .spawn((shells_by_name, BelongsToCadGeneratedRoot(root_ent)))
            .id();
        commands.entity(root_ent).add_child(shells_by_name_ent);
    }
}

pub fn update_shells_by_name_on_params_change<Params: ParametricCad + Component + Clone>(
    cad_generated: Query<
        (Entity, &Params),
        (Changed<Params>, With<CadGeneratedRoot>, Without<Cleanup>),
    >,
    mut shells_by_name_entities: Query<(Entity, &BelongsToCadGeneratedRoot, &mut CadShellsByName)>,
) {
    for (root_ent, params) in cad_generated.iter() {
        for (entity, &BelongsToCadGeneratedRoot(cur_root), mut shells_by_name) in
            shells_by_name_entities.iter_mut()
        {
            if cur_root != root_ent {
                continue;
            }
            // Get the shell builders from params...
            let shells_builders = match params.shells_builders() {
                Ok(result) => result,
                Err(e) => {
                    error!("shells_builders failed with error: {:?}", e);
                    return;
                }
            };
            // Clear existing shells...
            shells_by_name.clear();
            // Build Shells from Builders and add to shells_by_name...
            for (shell_name, shell_builder) in shells_builders.builders.iter() {
                let cad_shell = match shell_builder.build_cad_shell() {
                    Ok(shell) => shell,
                    Err(e) => {
                        error!(
                            "build_cad_shell for shell_name: {:?} failed, error: {:?}",
                            shell_name, e
                        );
                        continue;
                    }
                };
                shells_by_name.insert(shell_name.clone(), cad_shell);
            }
        }
    }
}

pub fn shells_to_cursors<Params: ParametricCad + Component + Clone>(
    mut commands: Commands,
    cad_generated: Query<&Params, (With<CadGeneratedRoot>, Without<Cleanup>)>,
    shells_by_name_entities: Query<
        (Entity, &CadShellsByName, &BelongsToCadGeneratedRoot),
        Changed<CadShellsByName>,
    >,
    mut cad_cursors: Query<
        (
            Entity,
            &CadCursorName,
            &BelongsToCadGeneratedRoot,
            &mut Transform,
            &mut CadGeneratedCursorPreviousTransform,
            &CadGeneratedCursorState,
        ),
        (With<CadGeneratedCursor>, Without<CadGeneratedRoot>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, shells_by_name, &BelongsToCadGeneratedRoot(root_ent)) in
        shells_by_name_entities.iter()
    {
        // Get params from root...
        let Ok(params) = cad_generated.get(root_ent) else {
            // warn!("Could not get `CadGeneratedRoot` with associated params!");
            continue;
        };
        // Cursors...
        let Ok(cursors) = params.cursors(shells_by_name) else {
            warn!("Could not get cursors!");
            continue;
        };
        for (cursor_name, cursor) in cursors.iter() {
            let CadCursor {
                normal,
                transform,
                cursor_radius,
                cursor_type,
            } = cursor;

            if let Some((_, _, _, mut cursor_transform, mut prev_transform, cursor_state)) =
                cad_cursors.iter_mut().find(|(_, name, bel_root, _, _, _)| {
                    *name == cursor_name && bel_root.0 == root_ent
                })
            {
                // If cursor already exists, update it...
                // Update transform only in normal state...
                match cursor_state {
                    CadGeneratedCursorState::Normal => {
                        // Update transform using new translation/rotation but use original scale...
                        // Since we're setting scale for adaptive cursors, changing it causes flickering.
                        *cursor_transform = Transform {
                            translation: transform.translation,
                            rotation: transform.rotation,
                            scale: cursor_transform.scale,
                        };
                        *prev_transform = CadGeneratedCursorPreviousTransform(*cursor_transform);
                    }
                    CadGeneratedCursorState::Dragging => {
                        // Update prev transform using new translation/rotation but use original scale...
                        // Since we're setting scale for adaptive cursors, changing it causes flickering.
                        *prev_transform = CadGeneratedCursorPreviousTransform(Transform {
                            translation: transform.translation,
                            rotation: transform.rotation,
                            scale: cursor_transform.scale,
                        });
                    }
                }
            } else {
                // Spawn new cursor...
                let cursor = commands
                    .spawn((
                        PbrBundle {
                            material: materials.add(StandardMaterial {
                                base_color: Color::WHITE.with_a(0.4),
                                alpha_mode: AlphaMode::Blend,
                                unlit: true,
                                double_sided: true,
                                cull_mode: None,
                                ..default()
                            }),
                            mesh: meshes.add(shape::Circle::new(*cursor_radius)),
                            transform: *transform,
                            // visibility: Visibility::Hidden,
                            ..default()
                        },
                        cursor_name.clone(),
                        CadGeneratedCursor,
                        CadGeneratedCursorConfig {
                            cursor_radius: *cursor_radius,
                            drag_plane_normal: *normal,
                            cursor_type: cursor_type.clone(),
                        },
                        CadGeneratedCursorState::default(),
                        CadGeneratedCursorPreviousTransform(*transform),
                        BelongsToCadGeneratedRoot(root_ent),
                        NotShadowCaster,
                        // picking...
                        PickableBundle::default(), // <- Makes the mesh pickable.
                        NoBackfaceCulling,
                        // Disable highlight cursor...
                        Highlight::<StandardMaterial> {
                            hovered: Some(HighlightKind::new_dynamic(|mat| StandardMaterial {
                                base_color: mat.base_color.with_a(0.6),
                                ..mat.to_owned()
                            })),
                            pressed: Some(HighlightKind::new_dynamic(|mat| StandardMaterial {
                                base_color: mat.base_color.with_a(0.8),
                                ..mat.to_owned()
                            })),
                            selected: Some(HighlightKind::new_dynamic(|mat| StandardMaterial {
                                ..mat.to_owned()
                            })),
                        },
                    ))
                    .insert((
                        // events...
                        On::<Pointer<Move>>::send_event::<CursorPointerMoveEvent>(),
                        On::<Pointer<Out>>::send_event::<CursorPointerOutEvent>(),
                        // Add drag plane on drag start...
                        On::<Pointer<DragStart>>::run(cursor_drag_start),
                        On::<Pointer<DragEnd>>::run(cursor_drag_end),
                    ))
                    // Prevent de-select other ent when cursor is interacted with.
                    .insert(NoDeselect)
                    .id();
                // Add cursor to root...
                commands.entity(root_ent).add_child(cursor);
            }
        }
    }
}

pub fn shells_to_mesh_builder_events<Params: ParametricCad + Component + Clone>(
    cad_generated: Query<&Params, (With<CadGeneratedRoot>, Without<Cleanup>)>,
    shells_by_name_entities: Query<
        (Entity, &CadShellsByName, &BelongsToCadGeneratedRoot),
        Changed<CadShellsByName>,
    >,
    mut spawn_meshes_builder_evt: EventWriter<SpawnMeshesBuilder<Params>>,
    mut builder_queue: ResMut<MeshesBuilderQueue<Params>>,
    mut builder_creation_index: Local<usize>,
) {
    for (entity, shells_by_name, &BelongsToCadGeneratedRoot(root_ent)) in
        shells_by_name_entities.iter()
    {
        // Get params from root...
        let Ok(params) = cad_generated.get(root_ent) else {
            // warn!("Could not get `CadGeneratedRoot` with associated params!");
            continue;
        };
        let Ok(meshes_builders_by_shell) = params.meshes_builders_by_shell(shells_by_name) else {
            warn!("Could not get meshes_builders_by_shell!");
            continue;
        };
        for (shell_name, meshes_builder) in meshes_builders_by_shell.meshes_builders.iter() {
            // spawn_meshes_builder_evt.send(SpawnMeshesBuilder {
            //     shell_name: shell_name.clone(),
            //     meshes_builder: meshes_builder.clone(),
            //     belongs_to_root: BelongsToCadGeneratedRoot(root_ent),
            // });
            *builder_creation_index += 1;
            builder_queue.push_back(SpawnMeshesBuilder {
                shell_name: shell_name.clone(),
                meshes_builder: meshes_builder.clone(),
                belongs_to_root: BelongsToCadGeneratedRoot(root_ent),
                created_at_idx: *builder_creation_index,
            });
        }
    }
}

pub fn handle_spawn_meshes_builder_events<Params: ParametricCad + Component + Clone>(
    mut commands: Commands,
    cad_generated: Query<Entity, (With<CadGeneratedRoot>, Without<Cleanup>)>,
    mut mesh_builders: Query<
        (
            Entity,
            &CadShellName,
            &CadMeshName,
            &mut CadMeshBuilder<Params>,
            &BelongsToCadGeneratedRoot,
        ),
        Without<Cleanup>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut events: EventReader<SpawnMeshesBuilder<Params>>,
    mut task_pool: AsyncTaskPool<Result<(Mesh, SpawnMeshesBuilder<Params>)>>,
    mut builder_queue: ResMut<MeshesBuilderQueue<Params>>,
    mut builder_queue_inspector: ResMut<MeshesBuilderQueueInspector>,
    mut meshes_builder_task_results_map: ResMut<MeshesBuilderFinishedResultsMap<Params>>,
) {
    // Update inspector...
    builder_queue_inspector.meshes_builder_queue_size = builder_queue.len();

    // Spawn a set num of tasks per frame from queue...
    for _ in 0..10 {
        let Some(spawn_meshes_builder) = builder_queue.pop_front() else {
            break;
        };
        let spawn_meshes_builder = spawn_meshes_builder.clone();
        task_pool.spawn(async move {
            let SpawnMeshesBuilder {
                belongs_to_root,
                shell_name,
                meshes_builder,
                created_at_idx,
            } = spawn_meshes_builder.clone();
            let Ok(bevy_mesh) = meshes_builder.build_bevy_mesh() else {
                warn!("Could not build bevy_mesh for shell_name: {:?}", shell_name);
                return Err(anyhow!(
                    "Could not build bevy_mesh for shell_name: {:?}",
                    shell_name
                ));
            };

            Ok((bevy_mesh, spawn_meshes_builder))
        });
    }

    // Collect finished tasks...
    let finished_task_results = task_pool
        .iter_poll()
        .filter_map(|status| {
            if let AsyncTaskStatus::Finished(Ok(result)) = status {
                Some(result)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if finished_task_results.is_empty() {
        // clear map if no tasks...
        meshes_builder_task_results_map.clear();
    }
    // Store only the latest result per shell/root in the map.
    // This will prevent overriding from older tasks and prevent flashing jitter...
    for task_result in finished_task_results.iter() {
        let SpawnMeshesBuilder {
            belongs_to_root: BelongsToCadGeneratedRoot(root_ent),
            created_at_idx,
            shell_name,
            ..
        } = &task_result.1;
        if !cad_generated.contains(*root_ent) {
            // If root is not available, skip...
            continue;
        }
        if let Some(current_result) =
            meshes_builder_task_results_map.get(&(*root_ent, shell_name.clone()))
        {
            if current_result.1.created_at_idx < *created_at_idx {
                meshes_builder_task_results_map
                    .insert((*root_ent, shell_name.clone()), task_result.clone());
            }
        } else {
            meshes_builder_task_results_map
                .insert((*root_ent, shell_name.clone()), task_result.clone());
        }
    }

    // Spawn builders from task results map...
    for (
        bevy_mesh,
        SpawnMeshesBuilder {
            belongs_to_root: BelongsToCadGeneratedRoot(root_ent),
            shell_name,
            meshes_builder,
            created_at_idx,
        },
    ) in meshes_builder_task_results_map.values()
    {
        let mesh_hdl = meshes.add(bevy_mesh.clone());

        // cleanup old mesh builders + mesh bundles (that are not being updated/reused anymore)...
        let mesh_builders_to_be_cleaned =
            mesh_builders
                .iter()
                .filter(|(_, cur_shell_name, cur_mesh_name, _, cur_bel_root)| {
                    **cur_shell_name == *shell_name
                        && !meshes_builder.mesh_builders.contains_key(*cur_mesh_name)
                        && cur_bel_root.0 == *root_ent
                });
        for (entity, _, _, _, _) in mesh_builders_to_be_cleaned {
            commands.entity(entity).insert(Cleanup::Recursive);
        }

        for (mesh_name, mesh_builder) in meshes_builder.mesh_builders.iter() {
            let Ok(mesh_builder) = mesh_builder.clone().set_mesh_hdl(mesh_hdl.clone()) else {
                continue;
            };
            if let Some((_, _, _, mut cur_mesh_builder, _)) = mesh_builders.iter_mut().find(
                |(_, cur_shell_name, cur_mesh_name, _, cur_bel_root)| {
                    **cur_shell_name == *shell_name
                        && *cur_mesh_name == mesh_name
                        && cur_bel_root.0 == *root_ent
                },
            ) {
                // if mesh_builder already exists, update it...
                *cur_mesh_builder = mesh_builder;
            } else {
                // Spawn new mesh_builder and add to root(if exists)...
                let Some(mut root_ent_commands) = commands.get_entity(*root_ent) else {
                    continue;
                };
                root_ent_commands.with_children(|commands| {
                    commands.spawn((
                        shell_name.clone(),
                        mesh_name.clone(),
                        mesh_builder,
                        BelongsToCadGeneratedRoot(*root_ent),
                    ));
                });
            };
        }
    }
}

pub fn mesh_builder_to_bundle<Params: ParametricCad + Component + Clone>(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cad_generated: Query<Entity, (With<CadGeneratedRoot>, Without<Cleanup>)>,
    mut mesh_builders: Query<
        (
            Entity,
            Option<&CadGeneratedMesh>,
            &CadShellName,
            &CadMeshName,
            &CadMeshBuilder<Params>,
            &BelongsToCadGeneratedRoot,
        ),
        (Changed<CadMeshBuilder<Params>>, Without<Cleanup>),
    >,
) {
    for (
        entity,
        cad_generated_mesh,
        shell_name,
        mesh_name,
        mesh_builder,
        BelongsToCadGeneratedRoot(root_ent),
    ) in mesh_builders.iter_mut()
    {
        if !cad_generated.contains(*root_ent) {
            // If root is not available, skip...
            continue;
        }
        let Some(mut ent_commands) = commands.get_entity(entity) else {
            continue;
        };
        let cad_mesh = match mesh_builder.build() {
            Ok(cad_mesh) => cad_mesh,
            Err(e) => {
                error!(
                    "Failed to build cad_mesh with name {:?} with error: {:?}",
                    mesh_name, e
                );
                continue;
            }
        };
        let CadMesh {
            mesh_hdl,
            base_material,
            transform,
            outlines,
            // cursors,
        } = cad_mesh;
        let material_hdl = materials.add(base_material);

        if cad_generated_mesh.is_some() {
            // If mesh already exists, update it...
            ent_commands
                .insert((
                    MaterialMeshBundle {
                        material: material_hdl.clone(),
                        mesh: mesh_hdl,
                        transform,
                        ..Default::default()
                    },
                    CadGeneratedMeshOutlines(outlines.clone()),
                ))
                // Remove AABB for Bevy to recompute as it wont recompute by itself...
                // ref: https://github.com/bevyengine/bevy/issues/4294#issuecomment-1606056536)
                .remove::<Aabb>();
        } else {
            // Insert a new mesh comp if does not exist...
            ent_commands
                .insert((
                    MaterialMeshBundle {
                        material: material_hdl.clone(),
                        mesh: mesh_hdl,
                        transform,
                        ..Default::default()
                    },
                    CadGeneratedMeshOutlines(outlines.clone()),
                ))
                .insert((
                    Name::new(mesh_name.0.clone()),
                    CadGeneratedMesh,
                    BelongsToCadGeneratedRoot(*root_ent),
                    CadGeneratedMeshOutlinesState::default(),
                    WireFrameDisplaySettings::default(),
                    // picking...
                    PickableBundle::default(), // <- Makes the mesh pickable.
                    // Pickable::IGNORE,
                    // Disable highlight...
                    Highlight::<StandardMaterial> {
                        hovered: Some(HighlightKind::Fixed(material_hdl.clone())),
                        pressed: Some(HighlightKind::Fixed(material_hdl.clone())),
                        selected: Some(HighlightKind::Fixed(material_hdl.clone())),
                    },
                    // Add drag plane on drag start...
                    On::<Pointer<Move>>::run(mesh_pointer_move),
                    On::<Pointer<Out>>::run(mesh_pointer_out),
                ));
        }
    }
}