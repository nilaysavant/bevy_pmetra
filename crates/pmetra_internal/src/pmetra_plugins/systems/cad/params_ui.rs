use bevy::prelude::*;

use crate::{
    constants::PARAMS_UI_BOTTOM_SHIFT_PX,
    pmetra_core::builders::{CadSliderName, PmetraInteractions, PmetraModelling},
    pmetra_plugins::{
        components::{
            cad::{
                BelongsToCadGeneratedRoot, BelongsToCadGeneratedSlider, CadGeneratedRoot,
                CadGeneratedSlider, CadGeneratedSliderDragPlane,
            },
            camera::CadCamera,
            params_ui::ParamDisplayUi,
        },
        events::slider::{SliderPointerMoveEvent, SliderPointerOutEvent, TransformSliderEvent},
    },
};

pub fn setup_param_display_ui(mut commands: Commands, cameras: Query<Entity, Added<CadCamera>>) {
    if cameras.is_empty() {
        // Wait for a camera to be added.
        return;
    }
    debug!("Spawning ParamDisplayUi...");
    commands.spawn((
        Text::new("Params Text"),
        TextLayout {
            // Set the alignment of the Text
            justify: JustifyText::Center,
            linebreak: LineBreak::NoWrap,
        },
        TextFont {
            font_size: 16.,
            ..default()
        },
        // Set the style of the Node itself.
        Node {
            // Abs pos allows for ui that can be tracking a world pos, ie. of slider.
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.8)),
        Visibility::Hidden,
        ParamDisplayUi,
        PickingBehavior::IGNORE,
    ));
}

pub fn show_params_display_ui_on_hover_slider<Params: PmetraInteractions + Component>(
    mut events: EventReader<SliderPointerMoveEvent>,
    cameras: Query<(&Camera, &GlobalTransform), With<CadCamera>>,
    mut ui_nodes: Query<(&mut Text, &mut Node, &mut Visibility), With<ParamDisplayUi>>,
    generated_roots: Query<&Params, With<CadGeneratedRoot>>,
    sliders: Query<(&CadSliderName, &BelongsToCadGeneratedRoot), With<CadGeneratedSlider>>,
) {
    if events.is_empty() {
        return;
    }
    let Ok((camera, cam_glob_transform)) = cameras.get_single() else {
        return;
    };
    let Ok((mut text, mut ui_node_style, mut visibility)) = ui_nodes.get_single_mut() else {
        warn!("NO UI!");
        return;
    };
    for SliderPointerMoveEvent { target, hit } in events.read() {
        let Ok((slider_name, BelongsToCadGeneratedRoot(cad_root_ent))) = sliders.get(*target)
        else {
            continue;
        };
        let Ok(params) = generated_roots.get(*cad_root_ent) else {
            continue;
        };

        // Update UI text...
        let Ok(Some(tooltip)) = params.on_slider_tooltip(slider_name.clone()) else {
            continue;
        };
        text.0 = tooltip;
        // Update UI position...
        let Some(slider_pos) = hit.position else {
            continue;
        };
        // Get view translation to set the UI pos from world slider pos.
        let Ok(viewport_pos) = camera.world_to_viewport(cam_glob_transform, slider_pos) else {
            error!("Could not find world_to_viewport pos!");
            return;
        };
        ui_node_style.top = Val::Px(viewport_pos.y - PARAMS_UI_BOTTOM_SHIFT_PX);
        ui_node_style.left = Val::Px(viewport_pos.x);
        *visibility = Visibility::Visible;
    }
}

pub fn hide_params_display_ui_on_pointer_out_slider(
    _trigger: Trigger<Pointer<Out>>,
    mut ui_nodes: Query<&mut Visibility, With<ParamDisplayUi>>,
) {
    let Ok(mut visibility) = ui_nodes.get_single_mut() else {
        return;
    };
    *visibility = Visibility::Hidden;
}

pub fn show_params_display_ui_on_pointer_over_slider<Params: PmetraInteractions + Component>(
    trigger: Trigger<Pointer<Over>>,
    cameras: Query<(&Camera, &GlobalTransform), With<CadCamera>>,
    mut ui_nodes: Query<(&mut Text, &mut Node, &mut Visibility), With<ParamDisplayUi>>,
    generated_roots: Query<&Params, With<CadGeneratedRoot>>,
    sliders: Query<
        (&GlobalTransform, &CadSliderName, &BelongsToCadGeneratedRoot),
        With<CadGeneratedSlider>,
    >,
) {
    let Ok((camera, cam_glob_transform)) = cameras.get_single() else {
        return;
    };
    let Ok((mut text, mut ui_node_style, mut visibility)) = ui_nodes.get_single_mut() else {
        return;
    };
    let slider = trigger.target;
    let Ok((slider_glob_transform, slider_name, BelongsToCadGeneratedRoot(cad_root_ent))) =
        sliders.get(slider)
    else {
        return;
    };
    let Ok(params) = generated_roots.get(*cad_root_ent) else {
        return;
    };
    // Update UI text...
    let Ok(Some(tooltip)) = params.on_slider_tooltip(slider_name.clone()) else {
        return;
    };
    text.0 = tooltip;
    // Get view translation to set the UI pos from world slider pos.
    let Ok(viewport_pos) =
        camera.world_to_viewport(cam_glob_transform, slider_glob_transform.translation())
    else {
        error!("Could not find world_to_viewport pos!");
        return;
    };
    ui_node_style.top = Val::Px(viewport_pos.y - PARAMS_UI_BOTTOM_SHIFT_PX);
    ui_node_style.left = Val::Px(viewport_pos.x);
    *visibility = Visibility::Visible;
}

pub fn show_params_display_ui_on_pointer_move_drag_plane<Params: PmetraInteractions + Component>(
    trigger: Trigger<Pointer<Move>>,
    cameras: Query<(&Camera, &GlobalTransform), With<CadCamera>>,
    drag_planes: Query<&BelongsToCadGeneratedSlider, With<CadGeneratedSliderDragPlane>>,
    mut ui_nodes: Query<(&mut Text, &mut Node, &mut Visibility), With<ParamDisplayUi>>,
    generated_roots: Query<&Params, With<CadGeneratedRoot>>,
    sliders: Query<
        (&GlobalTransform, &CadSliderName, &BelongsToCadGeneratedRoot),
        With<CadGeneratedSlider>,
    >,
) {
    let Ok((camera, cam_glob_transform)) = cameras.get_single() else {
        return;
    };
    let Ok((mut text, mut ui_node_style, mut visibility)) = ui_nodes.get_single_mut() else {
        return;
    };
    let drag_plane = trigger.target;
    let Ok(BelongsToCadGeneratedSlider(slider)) = drag_planes.get(drag_plane) else {
        return;
    };
    let Ok((slider_glob_transform, slider_name, BelongsToCadGeneratedRoot(cad_root_ent))) =
        sliders.get(*slider)
    else {
        return;
    };
    let Ok(params) = generated_roots.get(*cad_root_ent) else {
        return;
    };
    // Update UI text...
    let Ok(Some(tooltip)) = params.on_slider_tooltip(slider_name.clone()) else {
        return;
    };
    text.0 = tooltip;
    // Get view translation to set the UI pos from world slider pos.
    let Ok(viewport_pos) =
        camera.world_to_viewport(cam_glob_transform, slider_glob_transform.translation())
    else {
        error!("Could not find world_to_viewport pos!");
        return;
    };
    ui_node_style.top = Val::Px(viewport_pos.y - PARAMS_UI_BOTTOM_SHIFT_PX);
    ui_node_style.left = Val::Px(viewport_pos.x);
    *visibility = Visibility::Visible;
}
