use bevy::{
    color::palettes::css, ecs::component::Mutable, pbr::{NotShadowCaster, NotShadowReceiver}, prelude::*
};

use crate::{
    math::get_rotation_from_normals,
    pmetra_core::builders::{CadSliderName, CadSliderType, PmetraInteractions},
    pmetra_plugins::{
        cleanup_manager::Cleanup,
        components::{
            cad::{
                BelongsToCadGeneratedRoot, BelongsToCadGeneratedSlider, CadGeneratedMesh,
                CadGeneratedRoot, CadGeneratedRootSelectionState, CadGeneratedSlider,
                CadGeneratedSliderConfig, CadGeneratedSliderDragPlane,
                CadGeneratedSliderPreviousTransform, CadGeneratedSliderState,
            },
            params_ui::ParamDisplayUi,
        },
        resources::PmetraGlobalSettings,
        systems::gizmos::PmetraSliderOutlineGizmos,
    },
    prelude::CadCamera,
};

use super::params_ui::show_params_display_ui_on_pointer_move_drag_plane;

pub fn update_slider_visibility_based_on_root_selection(
    cad_generated: Query<(Entity, &CadGeneratedRootSelectionState), With<CadGeneratedRoot>>,
    mut cad_sliders: Query<(&BelongsToCadGeneratedRoot, &mut Visibility), With<CadGeneratedSlider>>,
) {
    for (root_ent, root_selection) in cad_generated.iter() {
        for (&BelongsToCadGeneratedRoot(cur_root_ent), mut visibility) in cad_sliders.iter_mut() {
            if cur_root_ent != root_ent {
                continue;
            }
            // if any mesh is selected show sliders else hide sliders...
            if matches!(root_selection, CadGeneratedRootSelectionState::Selected) {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub fn slider_drag_start<Params: PmetraInteractions + Component>(
    drag_event: Trigger<Pointer<DragStart>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cad_meshes: Query<(Entity, &BelongsToCadGeneratedRoot), With<CadGeneratedMesh>>,
    mut cad_sliders: Query<
        (
            &CadGeneratedSliderConfig,
            &mut CadGeneratedSliderState,
            &Transform,
            &BelongsToCadGeneratedRoot,
        ),
        With<CadGeneratedSlider>,
    >,
    global_settings: Res<PmetraGlobalSettings>,
) {
    let slider = drag_event.target;
    let Ok((
        CadGeneratedSliderConfig {
            drag_plane_normal, ..
        },
        mut slider_state,
        slider_transform,
        BelongsToCadGeneratedRoot(cad_root),
    )) = cad_sliders.get_mut(slider)
    else {
        return;
    };
    let PmetraGlobalSettings {
        slider_drag_plane_size,
        slider_drag_plane_debug,
        ..
    } = *global_settings;
    // set state to dragging
    *slider_state = CadGeneratedSliderState::Dragging;

    // Get transform from slider normal...
    let rotation = get_rotation_from_normals(Vec3::Y, *drag_plane_normal);
    let transform = Transform::from_translation(slider_transform.translation)
        // Get rotation by sub rotations...
        .with_rotation(rotation);
    let drag_plane = commands
        .spawn((
            Mesh3d(
                meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(slider_drag_plane_size / 2.)).mesh()),
            ),
            MeshMaterial3d(
                materials.add(StandardMaterial {
                    base_color: css::GREEN
                        .with_alpha(if slider_drag_plane_debug { 0.75 } else { 0.0 })
                        .into(),
                    alpha_mode: AlphaMode::Blend,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                }),
            ),
            transform,
            NotShadowCaster,
            NotShadowReceiver,
            CadGeneratedSliderDragPlane,
            BelongsToCadGeneratedSlider(slider),
            BelongsToCadGeneratedRoot(*cad_root),
            // picking
            RayCastBackfaces,
        ))
        .observe(transform_slider_on_pointer_move)
        .observe(show_params_display_ui_on_pointer_move_drag_plane::<Params>)
        .id();
    // Add drag plane as child of root for proper transform...
    commands.entity(*cad_root).add_child(drag_plane);
    // Disable picking on slider, etc...
    commands.entity(slider).insert(Pickable::IGNORE);
    // Disable picking on all meshes (not just belonging to current root)...
    // This makes sure that the drag plane picking is not occluded by any other meshes.
    for (entity, BelongsToCadGeneratedRoot(_cad_root_ent_cur)) in cad_meshes.iter() {
        commands.entity(entity).insert(Pickable::IGNORE);
    }
    commands.entity(*cad_root).insert(Pickable::IGNORE);
}

pub fn slider_drag_end(
    drag_event: Trigger<Pointer<DragEnd>>,
    mut commands: Commands,
    cad_slider_drag_planes: Query<
        (Entity, &BelongsToCadGeneratedSlider),
        With<CadGeneratedSliderDragPlane>,
    >,
    cad_meshes: Query<(Entity, &BelongsToCadGeneratedRoot), With<CadGeneratedMesh>>,
    mut sliders: Query<
        (
            &mut Transform,
            &CadGeneratedSliderPreviousTransform,
            &CadGeneratedSliderConfig,
            &mut CadGeneratedSliderState,
            &BelongsToCadGeneratedRoot,
        ),
        With<CadGeneratedSlider>,
    >,
    mut ui_nodes: Query<&mut Visibility, With<ParamDisplayUi>>,
) {
    let slider = drag_event.target;
    // Remove drag planes...
    for (entity, BelongsToCadGeneratedSlider(cur_slider_entity)) in cad_slider_drag_planes.iter() {
        if *cur_slider_entity != slider {
            continue;
        }
        commands.entity(entity).despawn_recursive();
    }
    // Update prev transform with new transform...
    let Ok((
        mut slider_transform,
        prev_transform,
        _config,
        mut slider_state,
        BelongsToCadGeneratedRoot(cad_root),
    )) = sliders.get_mut(slider)
    else {
        error!("Slider not found!");
        return;
    };
    // reset current transform to prev (as now prev would have been updated)
    *slider_transform = prev_transform.0;
    // reset state to default
    *slider_state = CadGeneratedSliderState::default();

    // Make slider, etc pick-able again...
    commands.entity(slider).insert(Pickable::default());
    // Enable picking on all meshes (not just belonging to current root)...
    // Since previously (on drag) all were set to ignore picking.
    for (entity, BelongsToCadGeneratedRoot(_cad_root_ent_cur)) in cad_meshes.iter() {
        commands.entity(entity).insert(Pickable::default());
    }
    commands
        .entity(*cad_root)
        .insert(Pickable::default());
    // Make params ui visible...
    let Ok(mut params_ui_visibility) = ui_nodes.get_single_mut() else {
        return;
    };
    *params_ui_visibility = Visibility::Hidden;
}

pub fn transform_slider_on_pointer_move(
    trigger: Trigger<Pointer<Move>>,
    cad_generated: Query<(Entity, &Transform), (With<CadGeneratedRoot>, Without<Cleanup>)>,
    slider_drag_planes: Query<
        (&BelongsToCadGeneratedRoot, &BelongsToCadGeneratedSlider),
        With<CadGeneratedSliderDragPlane>,
    >,
    mut sliders: Query<
        (&mut Transform, &CadGeneratedSliderConfig),
        (With<CadGeneratedSlider>, Without<CadGeneratedRoot>),
    >,
) {
    let target = trigger.target;
    let hit = trigger.hit.clone();
    let drag_plane = target;
    let Some(hit_point) = hit.position else {
        error!("No hit point found!");
        return;
    };
    let Ok((&BelongsToCadGeneratedRoot(root_ent), BelongsToCadGeneratedSlider(slider))) =
        slider_drag_planes.get(drag_plane)
    else {
        warn!("drag plane not found!");
        return;
    };
    let Ok((_, root_transform)) = cad_generated.get(root_ent) else {
        return;
    };
    let root_transform_inverse_affine = root_transform.compute_affine().inverse();
    let hit_point_local_space = root_transform_inverse_affine.transform_point3(hit_point);
    let Ok((mut transform, CadGeneratedSliderConfig { slider_type, .. })) =
        sliders.get_mut(*slider)
    else {
        error!("Slider not found!");
        return;
    };
    match slider_type {
        CadSliderType::Planer => {
            transform.translation = hit_point_local_space;
        }
        CadSliderType::Linear {
            direction,
            limit_min,
            limit_max,
        } => {
            let original_translation = transform.translation;
            let new_local_translation =
                (hit_point_local_space - original_translation).project_onto_normalized(*direction);
            if let (Some(limit_min), Some(limit_max)) = (limit_min, limit_max) {
                transform.translation =
                    (transform.translation + new_local_translation).clamp(*limit_min, *limit_max);
            } else {
                transform.translation += new_local_translation;
            }
        }
    }
}

pub fn update_params_from_sliders<Params: PmetraInteractions + Component<Mutability = Mutable>>(
    mut generated_roots: Query<(Entity, &mut Params), With<CadGeneratedRoot>>,
    sliders: Query<
        (
            &CadSliderName,
            &BelongsToCadGeneratedRoot,
            &Transform,
            &CadGeneratedSliderPreviousTransform,
            &CadGeneratedSliderConfig,
            &CadGeneratedSliderState,
        ),
        With<CadGeneratedSlider>,
    >,
) {
    for (
        slider_name,
        BelongsToCadGeneratedRoot(cad_generated_root),
        transform,
        previous_transform,
        _config,
        state,
    ) in sliders.iter()
    {
        let Ok((_cad_generated_ent, mut params)) = generated_roots.get_mut(*cad_generated_root)
        else {
            continue;
        };
        let is_transforms_equal = transform
            .translation
            .abs_diff_eq(previous_transform.0.translation, 0.01);
        if !is_transforms_equal && matches!(state, CadGeneratedSliderState::Dragging) {
            // run event handler on params...
            params.on_slider_transform(slider_name.clone(), previous_transform.0, *transform);
        }
    }
}

pub fn draw_slider_gizmo(
    cad_generated: Query<(Entity, &CadGeneratedRootSelectionState), With<CadGeneratedRoot>>,
    sliders: Query<
        (
            &BelongsToCadGeneratedRoot,
            &CadGeneratedSliderConfig,
            &GlobalTransform,
        ),
        (With<CadGeneratedSlider>, Without<CadGeneratedMesh>),
    >,
    mut gizmos: Gizmos<PmetraSliderOutlineGizmos>,
) {
    for (root_ent, selection_state) in cad_generated.iter() {
        for (&BelongsToCadGeneratedRoot(cur_root_ent), config, glob_transform) in sliders.iter() {
            if cur_root_ent != root_ent {
                continue;
            }
            if !matches!(selection_state, CadGeneratedRootSelectionState::Selected) {
                // if not selected don't draw outline...
                continue;
            }
            let transform = glob_transform.compute_transform();
            // draw outline circle...
            gizmos.circle(
                Isometry3d::new(
                    transform.translation,
                    get_rotation_from_normals(Vec3::Z, *transform.local_z()),
                ),
                config.thumb_radius * transform.scale.x,
                Color::WHITE,
            );
        }
    }
}

pub fn scale_sliders_based_on_zoom_level(
    cameras: Query<(&Camera, &Transform), (With<CadCamera>, Changed<Transform>)>,
    cad_gen_root: Query<
        (Entity, &CadGeneratedRootSelectionState),
        (With<CadGeneratedRoot>, Without<CadCamera>),
    >,
    mut sliders: Query<
        (&mut Transform, &GlobalTransform),
        (
            With<CadGeneratedSlider>,
            Without<CadCamera>,
            Without<CadGeneratedMesh>,
        ),
    >,
) {
    let Some((_, camera_transform)) = cameras.iter().find(|(cam, ..)| cam.is_active) else {
        return;
    };
    let Some((_selected_cad_mesh, ..)) = cad_gen_root
        .iter()
        .find(|(_, selection, ..)| matches!(selection, CadGeneratedRootSelectionState::Selected))
    else {
        return;
    };
    for (mut transform, glob_transform) in sliders.iter_mut() {
        let camera_to_slider_dist = camera_transform
            .translation
            .distance(glob_transform.translation());
        transform.scale = Vec3::ONE * camera_to_slider_dist.clamp(0., 5.) / 5.;
    }
}
