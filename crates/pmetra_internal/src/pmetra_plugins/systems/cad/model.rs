use std::task::Poll;

use anyhow::{anyhow, Result};
use bevy::{color::palettes::css, pbr::NotShadowCaster, prelude::*, render::primitives::Aabb};
use bevy_async_task::TaskPool;

use crate::{
    pmetra_core::builders::{
        CadMesh, CadMeshBuilder, CadMeshName, CadShellName, CadShellsByName, CadSlider,
        CadSliderName, PmetraInteractions, PmetraModelling,
    },
    pmetra_plugins::{
        cleanup_manager::Cleanup,
        components::{
            cad::{
                BelongsToCadGeneratedRoot, CadGeneratedMesh, CadGeneratedMeshOutlines,
                CadGeneratedRoot, CadGeneratedRootSelectionState, CadGeneratedSlider,
                CadGeneratedSliderConfig, CadGeneratedSliderPreviousTransform,
                CadGeneratedSliderState,
            },
            wire_frame::WireFrameDisplaySettings,
        },
        events::cad::{GenerateCadModel, SpawnMeshesBuilder},
        resources::{
            MeshesBuilderFinishedResultsMap, MeshesBuilderQueue, MeshesBuilderQueueInspector,
        },
    },
};

use super::{
    params_ui::{
        hide_params_display_ui_on_pointer_out_slider, show_params_display_ui_on_pointer_over_slider,
    },
    root::{root_on_click, root_pointer_move, root_pointer_out},
    slider::{slider_drag_end, slider_drag_start},
};

pub fn spawn_shells_by_name_on_generate<Params: PmetraModelling + Component + Clone>(
    mut commands: Commands,
    mut events: EventReader<GenerateCadModel<Params>>,
    cad_generated: Query<Entity, (With<Params>, With<CadGeneratedRoot>, Without<Cleanup>)>,
) {
    for GenerateCadModel {
        params,
        transform,
        remove_existing_models,
    } in events.read()
    {
        if *remove_existing_models {
            for root_ent in cad_generated.iter() {
                // Remove root and its descendants...
                let Some(mut ent_commands) = commands.get_entity(root_ent) else {
                    continue;
                };
                // Using try_insert to prevent panic...
                ent_commands.try_insert(Cleanup::Recursive);
            }
        }

        // Spawn root...
        let root_ent = commands
            .spawn((
                *transform,
                Visibility::default(),
                CadGeneratedRoot,
                CadGeneratedRootSelectionState::default(),
                params.clone(),
            ))
            // picking observers...
            .observe(root_pointer_move)
            .observe(root_pointer_out)
            .observe(root_on_click)
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

pub fn update_shells_by_name_on_params_change<Params: PmetraModelling + Component + Clone>(
    cad_generated: Query<
        (Entity, &Params),
        (Changed<Params>, With<CadGeneratedRoot>, Without<Cleanup>),
    >,
    mut shells_by_name_entities: Query<(Entity, &BelongsToCadGeneratedRoot, &mut CadShellsByName)>,
) {
    for (root_ent, params) in cad_generated.iter() {
        for (_entity, &BelongsToCadGeneratedRoot(cur_root), mut shells_by_name) in
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

pub fn shells_to_sliders<Params: PmetraInteractions + Component + Clone>(
    mut commands: Commands,
    cad_generated: Query<&Params, (With<CadGeneratedRoot>, Without<Cleanup>)>,
    shells_by_name_entities: Query<
        (Entity, &CadShellsByName, &BelongsToCadGeneratedRoot),
        Changed<CadShellsByName>,
    >,
    mut slider_comps: Query<
        (
            Entity,
            &CadSliderName,
            &BelongsToCadGeneratedRoot,
            &mut Transform,
            &mut CadGeneratedSliderPreviousTransform,
            &CadGeneratedSliderState,
        ),
        (With<CadGeneratedSlider>, Without<CadGeneratedRoot>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (_entity, shells_by_name, &BelongsToCadGeneratedRoot(root_ent)) in
        shells_by_name_entities.iter()
    {
        // Get params from root...
        let Ok(params) = cad_generated.get(root_ent) else {
            // warn!("Could not get `CadGeneratedRoot` with associated params!");
            continue;
        };
        // Sliders...
        let Ok(sliders) = params.sliders(shells_by_name) else {
            warn!("Could not get sliders!");
            continue;
        };
        for (slider_name, slider) in sliders.iter() {
            let CadSlider {
                drag_plane_normal,
                transform,
                thumb_radius,
                slider_type,
            } = slider;

            if let Some((_, _, _, mut slider_transform, mut prev_transform, slider_state)) =
                slider_comps
                    .iter_mut()
                    .find(|(_, name, bel_root, _, _, _)| {
                        *name == slider_name && bel_root.0 == root_ent
                    })
            {
                // If slider already exists, update it...
                // Update transform only in normal state...
                match slider_state {
                    CadGeneratedSliderState::Normal => {
                        // Update transform using new translation/rotation but use original scale...
                        // Since we're setting scale for adaptive sliders, changing it causes flickering.
                        *slider_transform = Transform {
                            translation: transform.translation,
                            rotation: transform.rotation,
                            scale: slider_transform.scale,
                        };
                        *prev_transform = CadGeneratedSliderPreviousTransform(*slider_transform);
                    }
                    CadGeneratedSliderState::Dragging => {
                        // Update prev transform using new translation/rotation but use original scale...
                        // Since we're setting scale for adaptive sliders, changing it causes flickering.
                        *prev_transform = CadGeneratedSliderPreviousTransform(Transform {
                            translation: transform.translation,
                            rotation: transform.rotation,
                            scale: slider_transform.scale,
                        });
                    }
                }
            } else {
                // Spawn new slider...
                let slider = commands
                    .spawn((
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: css::WHITE.with_alpha(0.4).into(),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            double_sided: true,
                            cull_mode: None,
                            ..default()
                        })),
                        Mesh3d(meshes.add(Circle::new(*thumb_radius))),
                        *transform,
                        slider_name.clone(),
                        CadGeneratedSlider,
                        CadGeneratedSliderConfig {
                            thumb_radius: *thumb_radius,
                            drag_plane_normal: *drag_plane_normal,
                            slider_type: slider_type.clone(),
                        },
                        CadGeneratedSliderState::default(),
                        CadGeneratedSliderPreviousTransform(*transform),
                        BelongsToCadGeneratedRoot(root_ent),
                        NotShadowCaster,
                        // picking...
                        RayCastBackfaces,
                    ))
                    .observe(show_params_display_ui_on_pointer_over_slider::<Params>)
                    .observe(hide_params_display_ui_on_pointer_out_slider)
                    // Add drag plane on drag start...
                    .observe(slider_drag_start::<Params>)
                    .observe(slider_drag_end)
                    // TODO: Re-implement de-select prevention when selection is implemented...
                    // Prevent de-select other ent when slider is interacted with.
                    // .insert(NoDeselect)
                    .id();
                // Add slider to root...
                commands.entity(root_ent).add_child(slider);
            }
        }
    }
}

pub fn shells_to_mesh_builder_events<Params: PmetraModelling + Component + Clone>(
    cad_generated: Query<&Params, (With<CadGeneratedRoot>, Without<Cleanup>)>,
    shells_by_name_entities: Query<
        (Entity, &CadShellsByName, &BelongsToCadGeneratedRoot),
        Changed<CadShellsByName>,
    >,
    mut builder_queue: ResMut<MeshesBuilderQueue<Params>>,
    mut builder_creation_index: Local<usize>,
) {
    for (_entity, shells_by_name, &BelongsToCadGeneratedRoot(root_ent)) in
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

pub fn handle_spawn_meshes_builder_events<Params: PmetraModelling + Component + Clone>(
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
    mut task_pool: TaskPool<Result<(Mesh, SpawnMeshesBuilder<Params>)>>,
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
                shell_name,
                meshes_builder,
                ..
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
            if let Poll::Ready(Ok(result)) = status {
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
            ..
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

pub fn mesh_builder_to_bundle<Params: PmetraModelling + Component + Clone>(
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
        _shell_name,
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
        } = cad_mesh;
        let material_hdl = materials.add(base_material);

        if cad_generated_mesh.is_some() {
            // If mesh already exists, update it...
            ent_commands
                .insert((
                    MeshMaterial3d(material_hdl.clone()),
                    Mesh3d(mesh_hdl),
                    transform,
                    CadGeneratedMeshOutlines(outlines.clone()),
                ))
                // Remove AABB for Bevy to recompute as it wont recompute by itself...
                // ref: https://github.com/bevyengine/bevy/issues/4294#issuecomment-1606056536)
                .remove::<Aabb>();
        } else {
            // Insert a new mesh comp if does not exist...
            ent_commands
                .insert((
                    MeshMaterial3d(material_hdl.clone()),
                    Mesh3d(mesh_hdl),
                    transform,
                    CadGeneratedMeshOutlines(outlines.clone()),
                ))
                .insert((
                    Name::new(mesh_name.0.clone()),
                    CadGeneratedMesh,
                    BelongsToCadGeneratedRoot(*root_ent),
                    WireFrameDisplaySettings::default(),
                ));
        }
    }
}
