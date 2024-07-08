use bevy::prelude::*;
use bevy_mod_picking::picking_core::Pickable;

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
        TextBundle {
            text: Text::from_section(
                "Params Text",
                TextStyle {
                    font_size: 16.,
                    ..default()
                },
            )
            // Set the alignment of the Text
            .with_justify(JustifyText::Center)
            .with_no_wrap(),
            // Set the style of the TextBundle itself.
            style: Style {
                // Abs pos allows for ui that can be tracking a world pos, ie. of slider.
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: BackgroundColor(Color::BLACK.with_alpha(0.8)),
            visibility: Visibility::Hidden,
            ..default()
        },
        ParamDisplayUi,
        Pickable::IGNORE, // Ignore picking events on the UI.
    ));
}

pub fn show_params_display_ui_on_hover_slider<Params: PmetraInteractions + Component>(
    mut events: EventReader<SliderPointerMoveEvent>,
    cameras: Query<(&Camera, &GlobalTransform), With<CadCamera>>,
    mut ui_nodes: Query<(&mut Text, &mut Style, &mut Visibility), With<ParamDisplayUi>>,
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
        text.sections.first_mut().unwrap().value = tooltip;
        // Update UI position...
        let Some(slider_pos) = hit.position else {
            continue;
        };
        // Get view translation to set the UI pos from world slider pos.
        let Some(viewport_pos) = camera.world_to_viewport(cam_glob_transform, slider_pos) else {
            error!("Could not find world_to_viewport pos!");
            return;
        };
        ui_node_style.top = Val::Px(viewport_pos.y - PARAMS_UI_BOTTOM_SHIFT_PX);
        ui_node_style.left = Val::Px(viewport_pos.x);
        *visibility = Visibility::Visible;
    }
}

pub fn hide_params_display_ui_on_out_slider(
    mut events: EventReader<SliderPointerOutEvent>,
    mut ui_nodes: Query<&mut Visibility, With<ParamDisplayUi>>,
) {
    if events.is_empty() {
        return;
    }
    let Ok(mut visibility) = ui_nodes.get_single_mut() else {
        return;
    };
    for SliderPointerOutEvent { target, hit } in events.read() {
        *visibility = Visibility::Hidden;
    }
}

pub fn move_params_display_ui_on_transform_slider<Params: PmetraInteractions + Component>(
    mut events: EventReader<TransformSliderEvent>,
    slider_drag_planes: Query<&BelongsToCadGeneratedSlider, With<CadGeneratedSliderDragPlane>>,
    cameras: Query<(&Camera, &GlobalTransform), With<CadCamera>>,
    mut ui_nodes: Query<(&mut Text, &mut Style, &mut Visibility), With<ParamDisplayUi>>,
    generated_roots: Query<&Params, With<CadGeneratedRoot>>,
    sliders: Query<
        (&GlobalTransform, &CadSliderName, &BelongsToCadGeneratedRoot),
        With<CadGeneratedSlider>,
    >,
) {
    if events.is_empty() {
        return;
    }
    let Ok((camera, cam_glob_transform)) = cameras.get_single() else {
        return;
    };
    let Ok((mut text, mut ui_node_style, mut visibility)) = ui_nodes.get_single_mut() else {
        return;
    };
    for TransformSliderEvent { target, hit } in events.read() {
        let drag_plane = *target;
        let Ok(BelongsToCadGeneratedSlider(slider)) = slider_drag_planes.get(drag_plane) else {
            warn!("drag plane not found!");
            return;
        };
        let Ok((slider_glob_transform, slider_name, BelongsToCadGeneratedRoot(cad_root_ent))) =
            sliders.get(*slider)
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
        text.sections.first_mut().unwrap().value = tooltip;
        // Update UI position...
        let Some(slider_pos) = hit.position else {
            continue;
        };
        // Get view translation to set the UI pos from world slider pos.
        let Some(viewport_pos) =
            camera.world_to_viewport(cam_glob_transform, slider_glob_transform.translation())
        else {
            error!("Could not find world_to_viewport pos!");
            return;
        };
        ui_node_style.top = Val::Px(viewport_pos.y - PARAMS_UI_BOTTOM_SHIFT_PX);
        ui_node_style.left = Val::Px(viewport_pos.x);
        *visibility = Visibility::Visible;
    }
}
