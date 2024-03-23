use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::{
    backends::raycast::bevy_mod_raycast::markers::NoBackfaceCulling, prelude::*, PickableBundle,
};

use crate::{
    bevy_plugin::{
        components::{
            cad::{
                BelongsToCadGeneratedMesh, BelongsToCadGeneratedRoot, CadGeneratedCursor,
                CadGeneratedCursorConfig, CadGeneratedCursorPreviousTransform,
                CadGeneratedCursorState, CadGeneratedMesh, CadGeneratedMeshOutlines,
                CadGeneratedMeshOutlinesState, CadGeneratedRoot,
            },
            wire_frame::WireFrameDisplaySettings,
        },
        events::{
            cad::GenerateCadModel,
            cursor::{CursorPointerMoveEvent, CursorPointerOutEvent},
        },
        systems::cad::{
            cursor::{cursor_drag_end, cursor_drag_start},
            mesh::{mesh_pointer_move, mesh_pointer_out},
        },
    },
    cad_core::builders::{CadCursor, CadCursorName, CadMesh, CadMeshName, ParametricCad},
};

pub fn generate_cad_model_on_event<Params: ParametricCad + Component + Clone>(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut events: EventReader<GenerateCadModel<Params>>,
    cad_generated: Query<Entity, (With<Params>, With<CadGeneratedRoot>)>,
) {
    for GenerateCadModel {
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

        let cad_generations = match params.build_cad_meshes() {
            Ok(result) => result,
            Err(e) => {
                error!("build_cad_meshes failed with error: {:?}", e);
                return;
            }
        };
        // Spawn root...
        let cad_generated_root = commands
            .spawn((SpatialBundle::default(), CadGeneratedRoot, params.clone()))
            .id();

        for (mesh_name, cad_mesh) in cad_generations.iter() {
            let CadMesh {
                mesh,
                base_material,
                outlines,
                transform,
                cursors,
            } = cad_mesh;

            let mut material = base_material.clone();
            let material_hdl = materials.add(material.clone());

            // Spawn generated mesh...
            let cad_generated_mesh = commands
                .spawn((
                    PbrBundle {
                        material: material_hdl.clone(),
                        mesh: meshes.add(mesh.clone()),
                        transform: *transform,
                        ..default()
                    },
                    mesh_name.clone(),
                    Name::new(mesh_name.0.clone()),
                    CadGeneratedMesh,
                    BelongsToCadGeneratedRoot(cad_generated_root),
                    CadGeneratedMeshOutlines(outlines.clone()),
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
            // Add generated mesh to root...
            commands
                .entity(cad_generated_root)
                .add_child(cad_generated_mesh);

            // Spawn cursors...
            for (cursor_name, face_cursor) in cursors.iter() {
                let CadCursor {
                    normal,
                    transform,
                    cursor_radius,
                    cursor_type,
                } = face_cursor;

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
                            mesh: meshes.add(shape::Circle::new(*cursor_radius).into()),
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
                        BelongsToCadGeneratedMesh(cad_generated_mesh),
                        BelongsToCadGeneratedRoot(cad_generated_root),
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
                commands.entity(cad_generated_root).add_child(cursor);
            }
        }

        info!("Truck setup ran!");
    }
}

pub fn update_cad_model_on_params_change<Params: ParametricCad + Component>(
    cad_generated: Query<(Entity, &Params), (With<CadGeneratedRoot>, Changed<Params>)>,
    mut cad_generated_mesh: Query<
        (
            Entity,
            &CadMeshName,
            &BelongsToCadGeneratedRoot,
            &Handle<Mesh>,
            &Handle<StandardMaterial>,
            &mut Transform,
            &mut CadGeneratedMeshOutlines,
        ),
        (With<CadGeneratedMesh>, Without<CadGeneratedRoot>),
    >,
    mut cad_cursors: Query<
        (
            Entity,
            &CadCursorName,
            &BelongsToCadGeneratedMesh,
            &BelongsToCadGeneratedRoot,
            &mut Transform,
            &mut CadGeneratedCursorPreviousTransform,
            &CadGeneratedCursorState,
        ),
        (
            With<CadGeneratedCursor>,
            Without<CadGeneratedMesh>,
            Without<CadGeneratedRoot>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (cad_generated_root, params) in cad_generated.iter() {
        let cad_generations = match params.build_cad_meshes() {
            Ok(result) => result,
            Err(e) => {
                error!("build_cad_meshes failed with error: {:?}", e);
                return;
            }
        };
        for (
            cad_generated_mesh_entity,
            cad_mesh_name,
            BelongsToCadGeneratedRoot(cad_root_cur),
            mesh_hdl,
            material_hdl,
            mut mesh_transform,
            mut mesh_outlines,
        ) in cad_generated_mesh.iter_mut()
        {
            if *cad_root_cur != cad_generated_root {
                continue;
            }

            let Some(CadMesh {
                mesh,
                base_material,
                outlines,
                transform,
                cursors,
            }) = cad_generations.get(cad_mesh_name)
            else {
                continue;
            };

            // Update mesh...
            let Some(current_mesh) = meshes.get_mut(mesh_hdl) else {
                continue;
            };
            *current_mesh = mesh.clone();

            // Update material...
            let mut material = base_material.clone();
            let Some(current_material) = materials.get_mut(material_hdl) else {
                continue;
            };
            *current_material = material.clone();

            // Update transform & outlines...
            *mesh_transform = *transform;
            mesh_outlines.0 = outlines.clone();

            // Update any cursors associated to mesh...
            for (
                cad_cursor_entity,
                cursor_name,
                BelongsToCadGeneratedMesh(cgm_entity),
                BelongsToCadGeneratedRoot(cad_root_cur),
                mut cursor_transform,
                mut prev_transform,
                cursor_state,
            ) in cad_cursors.iter_mut()
            {
                if *cgm_entity != cad_generated_mesh_entity {
                    continue;
                }
                if *cad_root_cur != cad_generated_root {
                    continue;
                }

                let Some(CadCursor {
                    normal,
                    transform,
                    cursor_radius,
                    cursor_type,
                }) = cursors.get(cursor_name)
                else {
                    continue;
                };
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
            }
        }
    }
}
