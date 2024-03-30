use anyhow::{anyhow, Result};
use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_async_task::{AsyncTaskPool, AsyncTaskStatus};
use bevy_mod_picking::{
    backends::raycast::bevy_mod_raycast::markers::NoBackfaceCulling, prelude::*, PickableBundle,
};
use truck_base::id;

use crate::{
    bevy_plugin::{
        components::cad::{self, CadGeneratedRoot},
        events::{
            cursor::{CursorPointerMoveEvent, CursorPointerOutEvent},
            lazy_cad::{GenerateLazyCadModel, SpawnMeshesBuilder},
        },
        resources::{MeshesBuilderQueue, MeshesBuilderQueueInspector},
    },
    cad_core::{
        builders::{CadCursor, CadCursorName, CadMeshName, CadShell},
        lazy_builders::{
            CadLazyMesh, CadMeshLazyBuilder, CadShellLazyBuilder, CadShellName, CadShellsByName,
            ParametricLazyCad,
        },
    },
    prelude::{
        BelongsToCadGeneratedMesh, BelongsToCadGeneratedRoot, CadGeneratedCursor,
        CadGeneratedCursorConfig, CadGeneratedCursorPreviousTransform, CadGeneratedCursorState,
        CadGeneratedMesh, CadGeneratedMeshOutlines, CadGeneratedMeshOutlinesState,
        WireFrameDisplaySettings,
    },
};

use super::{
    cursor::{cursor_drag_end, cursor_drag_start},
    mesh::{mesh_pointer_move, mesh_pointer_out},
};

pub fn spawn_shells_by_name_on_generate<Params: ParametricLazyCad + Component + Clone>(
    mut commands: Commands,
    mut events: EventReader<GenerateLazyCadModel<Params>>,
    cad_generated: Query<Entity, (With<Params>, With<CadGeneratedRoot>)>,
) {
    for GenerateLazyCadModel {
        params,
        remove_existing_models,
    } in events.read()
    {
        if *remove_existing_models {
            for entity in cad_generated.iter() {
                let Some(ent_commands) = commands.get_entity(entity) else {
                    continue;
                };
                ent_commands.despawn_recursive();
            }
        }

        // Spawn root...
        let root = commands
            .spawn((SpatialBundle::default(), CadGeneratedRoot, params.clone()))
            .id();

        // Get the shell builders from params...
        let shells_lazy_builders = match params.shells_builders() {
            Ok(result) => result,
            Err(e) => {
                error!("shells_builders failed with error: {:?}", e);
                return;
            }
        };

        let mut shells_by_name = CadShellsByName::default();
        // Build Shells from Builders and add to common resource for later use.
        for (shell_name, shell_builder) in shells_lazy_builders.builders.iter() {
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
        commands.spawn((shells_by_name, BelongsToCadGeneratedRoot(root)));
    }
}

pub fn update_shells_by_name_on_params_change<Params: ParametricLazyCad + Component + Clone>(
    cad_generated: Query<(Entity, &Params), (Changed<Params>, With<CadGeneratedRoot>)>,
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
            let shells_lazy_builders = match params.shells_builders() {
                Ok(result) => result,
                Err(e) => {
                    error!("shells_builders failed with error: {:?}", e);
                    return;
                }
            };
            // Clear existing shells...
            shells_by_name.clear();
            // Build Shells from Builders and add to shells_by_name...
            for (shell_name, shell_builder) in shells_lazy_builders.builders.iter() {
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

pub fn shells_to_cursors<Params: ParametricLazyCad + Component + Clone>(
    mut commands: Commands,
    cad_generated: Query<&Params, With<CadGeneratedRoot>>,
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
            warn!("Could not get `CadGeneratedRoot` with associated params!");
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
                    .insert(NoDeselect) // Prevent de-select other ent when cursor is interacted with.
                    .id();
                // Add cursor to root
                commands.entity(root_ent).add_child(cursor);
            }
        }
    }
}

pub fn shells_to_mesh_builder_events<Params: ParametricLazyCad + Component + Clone>(
    cad_generated: Query<&Params, With<CadGeneratedRoot>>,
    shells_by_name_entities: Query<
        (Entity, &CadShellsByName, &BelongsToCadGeneratedRoot),
        Changed<CadShellsByName>,
    >,
    mut spawn_meshes_builder_evt: EventWriter<SpawnMeshesBuilder<Params>>,
    mut builder_queue: ResMut<MeshesBuilderQueue<Params>>,
) {
    for (entity, shells_by_name, &BelongsToCadGeneratedRoot(root_ent)) in
        shells_by_name_entities.iter()
    {
        // Get params from root...
        let Ok(params) = cad_generated.get(root_ent) else {
            warn!("Could not get `CadGeneratedRoot` with associated params!");
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
            builder_queue.push_back(SpawnMeshesBuilder {
                shell_name: shell_name.clone(),
                meshes_builder: meshes_builder.clone(),
                belongs_to_root: BelongsToCadGeneratedRoot(root_ent),
            });
        }
    }
}

pub fn handle_spawn_meshes_builder_events<Params: ParametricLazyCad + Component + Clone>(
    mut commands: Commands,
    mut mesh_builders: Query<(
        Entity,
        &CadShellName,
        &CadMeshName,
        &mut CadMeshLazyBuilder<Params>,
        &BelongsToCadGeneratedRoot,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut events: EventReader<SpawnMeshesBuilder<Params>>,
    mut task_pool: AsyncTaskPool<Result<(Mesh, SpawnMeshesBuilder<Params>)>>,
    mut builder_queue: ResMut<MeshesBuilderQueue<Params>>,
    mut builder_queue_inspector: ResMut<MeshesBuilderQueueInspector>,
) {
    builder_queue_inspector.meshes_builder_queue_size = builder_queue.len();
    for _ in 0..2 {
        let Some(spawn_meshes_builder) = builder_queue.pop_front() else {
            break;
        };
        let spawn_meshes_builder = spawn_meshes_builder.clone();
        task_pool.spawn(async move {
            let SpawnMeshesBuilder {
                belongs_to_root,
                shell_name,
                meshes_builder,
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

    // Poll for completed tasks to get created meshes and spawn builders...
    for status in task_pool.iter_poll() {
        if let AsyncTaskStatus::Finished(Ok((
            bevy_mesh,
            SpawnMeshesBuilder {
                belongs_to_root: BelongsToCadGeneratedRoot(root_ent),
                shell_name,
                meshes_builder,
            },
        ))) = status
        {
            let mesh_hdl = meshes.add(bevy_mesh);

            for (mesh_name, mesh_builder) in meshes_builder.mesh_builders.iter() {
                let Ok(mesh_builder) = mesh_builder.clone().set_mesh_hdl(mesh_hdl.clone()) else {
                    continue;
                };
                if let Some((_, _, _, mut cur_mesh_builder, _)) = mesh_builders.iter_mut().find(
                    |(_, cur_shell_name, cur_mesh_name, _, cur_bel_root)| {
                        **cur_shell_name == shell_name
                            && *cur_mesh_name == mesh_name
                            && cur_bel_root.0 == root_ent
                    },
                ) {
                    // if mesh_builder already exists, update it...
                    *cur_mesh_builder = mesh_builder;
                } else {
                    // Spawn new mesh_builder...
                    commands.spawn((
                        shell_name.clone(),
                        mesh_name.clone(),
                        mesh_builder,
                        BelongsToCadGeneratedRoot(root_ent),
                    ));
                };
            }
        }
    }
}

pub fn mesh_builder_to_bundle<Params: ParametricLazyCad + Component + Clone>(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_builders: Query<
        (
            Entity,
            Option<&CadGeneratedMesh>,
            &CadShellName,
            &CadMeshName,
            &CadMeshLazyBuilder<Params>,
            &BelongsToCadGeneratedRoot,
        ),
        Changed<CadMeshLazyBuilder<Params>>,
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
        let CadLazyMesh {
            mesh_hdl,
            base_material,
            transform,
            outlines,
            // cursors,
        } = cad_mesh;
        let material_hdl = materials.add(base_material);

        if cad_generated_mesh.is_some() {
            // If mesh already exists, update it...
            commands.entity(entity).insert((
                MaterialMeshBundle {
                    material: material_hdl.clone(),
                    mesh: mesh_hdl,
                    transform,
                    ..Default::default()
                },
                CadGeneratedMeshOutlines(outlines.clone()),
            ));
        } else {
            // Spawn a new mesh...
            let mesh_entity = commands
                .entity(entity)
                .insert((
                    MaterialMeshBundle {
                        material: material_hdl.clone(),
                        mesh: mesh_hdl,
                        transform,
                        ..Default::default()
                    },
                    CadGeneratedMeshOutlines(outlines.clone()),
                ))
                .insert(PickableBundle::default())
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
                ))
                .id();
            // Add the spawned mesh as child of the root...
            let Some(mut root_ent_commands) = commands.get_entity(*root_ent) else {
                warn!("Could not get commands for entity: {:?}", *root_ent);
                continue;
            };
            root_ent_commands.push_children(&[mesh_entity]);
        }
    }
}
