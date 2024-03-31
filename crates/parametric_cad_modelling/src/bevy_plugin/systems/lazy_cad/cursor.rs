use bevy::prelude::*;
use bevy_mod_picking::{
    backend::HitData, backends::raycast::bevy_mod_raycast::markers::NoBackfaceCulling, prelude::*,
};

use crate::{
    bevy_plugin::{
        components::{
            cad::{
                BelongsToCadGeneratedCursor, BelongsToCadGeneratedMesh, BelongsToCadGeneratedRoot,
                CadGeneratedCursor, CadGeneratedCursorConfig, CadGeneratedCursorDragPlane,
                CadGeneratedCursorPreviousTransform, CadGeneratedCursorState, CadGeneratedMesh,
                CadGeneratedRoot,
            },
            camera::CadCamera,
            params_ui::ParamDisplayUi,
        },
        events::cursor::TransformCursorEvent,
    },
    cad_core::{
        builders::{CadCursorName, CadCursorType, CadMeshName, ParametricCad},
        lazy_builders::ParametricLazyCad,
    },
    math::get_rotation_from_normals,
    prelude::CadGeneratedRootSelectionState,
};

pub fn update_cursor_visibility_based_on_root_selection(
    cad_generated: Query<(Entity, &CadGeneratedRootSelectionState), With<CadGeneratedRoot>>,
    mut cad_cursors: Query<(&BelongsToCadGeneratedRoot, &mut Visibility), With<CadGeneratedCursor>>,
) {
    for (root_ent, root_selection) in cad_generated.iter() {
        for (&BelongsToCadGeneratedRoot(cur_root_ent), mut visibility) in cad_cursors.iter_mut() {
            if cur_root_ent != root_ent {
                continue;
            }
            // if any mesh is selected show cursors else hide cursors...
            if matches!(root_selection, CadGeneratedRootSelectionState::Selected) {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub fn cursor_drag_start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    drag_event: Listener<Pointer<DragStart>>,
    cad_meshes: Query<(Entity, &BelongsToCadGeneratedRoot), With<CadGeneratedMesh>>,
    mut cad_cursors: Query<
        (
            &CadGeneratedCursorConfig,
            &mut CadGeneratedCursorState,
            &Transform,
            &BelongsToCadGeneratedRoot,
        ),
        With<CadGeneratedCursor>,
    >,
) {
    let cursor = drag_event.target();
    let Ok((
        CadGeneratedCursorConfig {
            cursor_radius,
            drag_plane_normal,
            cursor_type,
        },
        mut cursor_state,
        cursor_transform,
        BelongsToCadGeneratedRoot(cad_root),
    )) = cad_cursors.get_mut(cursor)
    else {
        return;
    };
    // set state to dragging
    *cursor_state = CadGeneratedCursorState::Dragging;

    // Get transform from cursor normal...
    let rotation = get_rotation_from_normals(Vec3::Y, *drag_plane_normal);
    let transform = Transform::from_translation(cursor_transform.translation)
        // Get rotation by sub rotations...
        .with_rotation(rotation);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(100.)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE.with_a(0.0),
                alpha_mode: AlphaMode::Blend,
                double_sided: true,
                cull_mode: None,
                ..default()
            }),
            transform,
            ..default()
        },
        CadGeneratedCursorDragPlane,
        BelongsToCadGeneratedCursor(cursor),
        BelongsToCadGeneratedRoot(*cad_root),
        // picking
        PickableBundle::default(), // <- Makes the mesh pickable.
        NoBackfaceCulling,
        // Disable highlighting...
        Highlight::<StandardMaterial> {
            hovered: Some(HighlightKind::new_dynamic(|mat| StandardMaterial {
                ..mat.to_owned()
            })),
            pressed: Some(HighlightKind::new_dynamic(|mat| StandardMaterial {
                ..mat.to_owned()
            })),
            selected: Some(HighlightKind::new_dynamic(|mat| StandardMaterial {
                ..mat.to_owned()
            })),
        },
        On::<Pointer<Move>>::send_event::<TransformCursorEvent>(),
    ));
    // Disable picking on cursor, etc...
    commands.entity(cursor).insert(Pickable::IGNORE);
    // Disable picking on all meshes belonging to current root...
    for (entity, BelongsToCadGeneratedRoot(cad_root_ent_cur)) in cad_meshes.iter() {
        if cad_root_ent_cur != cad_root {
            continue;
        }
        commands.entity(entity).insert(Pickable::IGNORE);
    }
    commands.entity(*cad_root).insert(Pickable::IGNORE);
}

pub fn cursor_drag_end(
    mut commands: Commands,
    drag_event: Listener<Pointer<DragEnd>>,
    cad_cursor_drag_planes: Query<
        (Entity, &BelongsToCadGeneratedCursor),
        With<CadGeneratedCursorDragPlane>,
    >,
    cad_meshes: Query<(Entity, &BelongsToCadGeneratedRoot), With<CadGeneratedMesh>>,
    mut cursors: Query<
        (
            &mut Transform,
            &CadGeneratedCursorPreviousTransform,
            &CadGeneratedCursorConfig,
            &mut CadGeneratedCursorState,
            &BelongsToCadGeneratedRoot,
        ),
        With<CadGeneratedCursor>,
    >,
    mut ui_nodes: Query<&mut Visibility, With<ParamDisplayUi>>,
) {
    let cursor = drag_event.target();
    // Remove drag planes...
    for (entity, BelongsToCadGeneratedCursor(curr_cad_cursor_entity)) in
        cad_cursor_drag_planes.iter()
    {
        if *curr_cad_cursor_entity != cursor {
            continue;
        }
        commands.entity(entity).despawn_recursive();
    }
    // Update prev transform with new transform...
    let Ok((
        mut cursor_transform,
        prev_transform,
        config,
        mut cursor_state,
        BelongsToCadGeneratedRoot(cad_root),
    )) = cursors.get_mut(cursor)
    else {
        error!("Cursor not found!");
        return;
    };
    // reset current transform to prev (as now prev would have been updated)
    *cursor_transform = prev_transform.0;
    // reset state to default
    *cursor_state = CadGeneratedCursorState::default();

    // Make cursor, etc pick-able again...
    commands.entity(cursor).insert(Pickable::default());
    // Enable picking on all meshes belonging to current root...
    for (entity, BelongsToCadGeneratedRoot(cad_root_ent_cur)) in cad_meshes.iter() {
        if cad_root_ent_cur != cad_root {
            continue;
        }
        commands.entity(entity).insert(Pickable::default());
    }
    commands.entity(*cad_root).insert(Pickable::default());
    // Make params ui visible...
    let Ok(mut params_ui_visibility) = ui_nodes.get_single_mut() else {
        return;
    };
    *params_ui_visibility = Visibility::Hidden;
}

pub fn transform_cursor(
    mut events: EventReader<TransformCursorEvent>,
    cursor_drag_planes: Query<&BelongsToCadGeneratedCursor, With<CadGeneratedCursorDragPlane>>,
    mut cursors: Query<(&mut Transform, &CadGeneratedCursorConfig), With<CadGeneratedCursor>>,
) {
    for TransformCursorEvent { target, hit } in events.read() {
        let drag_plane = *target;
        let Some(hit_point) = hit.position else {
            error!("No hit point found!");
            return;
        };
        let Ok(BelongsToCadGeneratedCursor(cursor)) = cursor_drag_planes.get(drag_plane) else {
            error!("drag plane not found!");
            return;
        };
        let Ok((
            mut transform,
            CadGeneratedCursorConfig {
                cursor_radius,
                drag_plane_normal,
                cursor_type,
            },
        )) = cursors.get_mut(*cursor)
        else {
            error!("cursor not found!");
            return;
        };
        match cursor_type {
            CadCursorType::Planer => {
                transform.translation = hit_point;
            }
            CadCursorType::Linear {
                direction,
                limit_min,
                limit_max,
            } => {
                let original_translation = transform.translation;
                let new_local_translation =
                    (hit_point - original_translation).project_onto_normalized(*direction);
                if let (Some(limit_min), Some(limit_max)) = (limit_min, limit_max) {
                    transform.translation = (transform.translation + new_local_translation)
                        .clamp(*limit_min, *limit_max);
                } else {
                    transform.translation += new_local_translation;
                }
            }
        }
    }
}

pub fn update_params_from_cursors<Params: ParametricLazyCad + Component>(
    mut generated_roots: Query<(Entity, &mut Params), With<CadGeneratedRoot>>,
    cursors: Query<
        (
            &CadCursorName,
            &BelongsToCadGeneratedRoot,
            &Transform,
            &CadGeneratedCursorPreviousTransform,
            &CadGeneratedCursorConfig,
            &CadGeneratedCursorState,
        ),
        With<CadGeneratedCursor>,
    >,
) {
    for (
        cursor_name,
        BelongsToCadGeneratedRoot(cad_generated_root),
        transform,
        mut previous_transform,
        config,
        state,
    ) in cursors.iter()
    {
        let Ok((cad_generated_ent, mut params)) = generated_roots.get_mut(*cad_generated_root)
        else {
            continue;
        };
        let is_transforms_equal = transform
            .translation
            .abs_diff_eq(previous_transform.0.translation, 0.01);
        if !is_transforms_equal && matches!(state, CadGeneratedCursorState::Dragging) {
            // run event handler on params...
            params.on_cursor_transform(cursor_name.clone(), previous_transform.0, *transform);
        }
    }
}

pub fn draw_cursor_gizmo(
    cad_meshes: Query<(Entity, &PickSelection), With<CadGeneratedMesh>>,
    cursors: Query<
        (&CadGeneratedCursorConfig, &Transform),
        (With<CadGeneratedCursor>, Without<CadGeneratedMesh>),
    >,
    mut gizmos: Gizmos,
) {
    let Some((selected_cad_mesh, ..)) = cad_meshes
        .iter()
        .find(|(_, selection, ..)| selection.is_selected)
    else {
        return;
    };
    for (config, transform) in cursors.iter() {
        // draw outline circle...
        gizmos.circle(
            transform.translation,
            transform.local_z(),
            config.cursor_radius * transform.scale.x,
            Color::WHITE,
        );
    }
}

pub fn scale_cursors_based_on_zoom_level(
    cameras: Query<(&Camera, &Transform), (With<CadCamera>, Changed<Transform>)>,
    cad_meshes: Query<(Entity, &PickSelection), (With<CadGeneratedMesh>, Without<CadCamera>)>,
    mut cursors: Query<
        &mut Transform,
        (
            With<CadGeneratedCursor>,
            Without<CadCamera>,
            Without<CadGeneratedMesh>,
        ),
    >,
) {
    let Some((_, camera_transform)) = cameras.iter().find(|(cam, ..)| cam.is_active) else {
        return;
    };
    let Some((selected_cad_mesh, ..)) = cad_meshes
        .iter()
        .find(|(_, selection, ..)| selection.is_selected)
    else {
        return;
    };
    for mut transform in cursors.iter_mut() {
        let camera_to_cursor_dist = camera_transform.translation.distance(transform.translation);
        transform.scale = Vec3::ONE * camera_to_cursor_dist.clamp(0., 5.) / 5.;
    }
}
