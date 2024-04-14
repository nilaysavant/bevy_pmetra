use bevy::prelude::*;
use bevy_mod_picking::picking_core::Pickable;

use crate::{
    bevy_plugin::{
        components::{
            cad::{
                BelongsToCadGeneratedCursor, BelongsToCadGeneratedRoot, CadGeneratedCursor,
                CadGeneratedCursorDragPlane, CadGeneratedRoot,
            },
            camera::CadCamera,
            params_ui::ParamDisplayUi,
        },
        events::cursor::{CursorPointerMoveEvent, CursorPointerOutEvent, TransformCursorEvent},
    },
    pmetra_core::builders::{CadCursorName, ParametricCad},
    constants::PARAMS_UI_BOTTOM_SHIFT_PX,
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
                // Abs pos allows for ui that can be tracking a world pos, ie. of cursor.
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: BackgroundColor(Color::BLACK.with_a(0.8)),
            visibility: Visibility::Hidden,
            ..default()
        },
        ParamDisplayUi,
        Pickable::IGNORE, // Ignore picking events on the UI.
    ));
}

pub fn show_params_display_ui_on_hover_cursor<Params: ParametricCad + Component>(
    mut events: EventReader<CursorPointerMoveEvent>,
    cameras: Query<(&Camera, &GlobalTransform), With<CadCamera>>,
    mut ui_nodes: Query<(&mut Text, &mut Style, &mut Visibility), With<ParamDisplayUi>>,
    generated_roots: Query<&Params, With<CadGeneratedRoot>>,
    cursors: Query<(&CadCursorName, &BelongsToCadGeneratedRoot), With<CadGeneratedCursor>>,
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
    for CursorPointerMoveEvent { target, hit } in events.read() {
        let Ok((cursor_name, BelongsToCadGeneratedRoot(cad_root_ent))) = cursors.get(*target)
        else {
            continue;
        };
        let Ok(params) = generated_roots.get(*cad_root_ent) else {
            continue;
        };

        // Update UI text...
        let Ok(Some(tooltip)) = params.on_cursor_tooltip(cursor_name.clone()) else {
            continue;
        };
        text.sections.first_mut().unwrap().value = tooltip;
        // Update UI position...
        let Some(cursor_pos) = hit.position else {
            continue;
        };
        // Get view translation to set the UI pos from world cursor pos.
        let Some(viewport_pos) = camera.world_to_viewport(cam_glob_transform, cursor_pos) else {
            error!("Could not find world_to_viewport pos!");
            return;
        };
        ui_node_style.top = Val::Px(viewport_pos.y - PARAMS_UI_BOTTOM_SHIFT_PX);
        ui_node_style.left = Val::Px(viewport_pos.x);
        *visibility = Visibility::Visible;
    }
}

pub fn hide_params_display_ui_on_out_cursor(
    mut events: EventReader<CursorPointerOutEvent>,
    mut ui_nodes: Query<&mut Visibility, With<ParamDisplayUi>>,
) {
    if events.is_empty() {
        return;
    }
    let Ok(mut visibility) = ui_nodes.get_single_mut() else {
        return;
    };
    for CursorPointerOutEvent { target, hit } in events.read() {
        *visibility = Visibility::Hidden;
    }
}

pub fn move_params_display_ui_on_transform_cursor<Params: ParametricCad + Component>(
    mut events: EventReader<TransformCursorEvent>,
    cursor_drag_planes: Query<&BelongsToCadGeneratedCursor, With<CadGeneratedCursorDragPlane>>,
    cameras: Query<(&Camera, &GlobalTransform), With<CadCamera>>,
    mut ui_nodes: Query<(&mut Text, &mut Style, &mut Visibility), With<ParamDisplayUi>>,
    generated_roots: Query<&Params, With<CadGeneratedRoot>>,
    cursors: Query<(&CadCursorName, &BelongsToCadGeneratedRoot), With<CadGeneratedCursor>>,
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
    for TransformCursorEvent { target, hit } in events.read() {
        let drag_plane = *target;
        let Ok(BelongsToCadGeneratedCursor(cursor)) = cursor_drag_planes.get(drag_plane) else {
            error!("drag plane not found!");
            return;
        };
        let Ok((cursor_name, BelongsToCadGeneratedRoot(cad_root_ent))) = cursors.get(*cursor)
        else {
            continue;
        };
        let Ok(params) = generated_roots.get(*cad_root_ent) else {
            continue;
        };
        // Update UI text...
        let Ok(Some(tooltip)) = params.on_cursor_tooltip(cursor_name.clone()) else {
            continue;
        };
        text.sections.first_mut().unwrap().value = tooltip;
        // Update UI position...
        let Some(cursor_pos) = hit.position else {
            continue;
        };
        // Get view translation to set the UI pos from world cursor pos.
        let Some(viewport_pos) = camera.world_to_viewport(cam_glob_transform, cursor_pos) else {
            error!("Could not find world_to_viewport pos!");
            return;
        };
        ui_node_style.top = Val::Px(viewport_pos.y - PARAMS_UI_BOTTOM_SHIFT_PX);
        ui_node_style.left = Val::Px(viewport_pos.x);
        *visibility = Visibility::Visible;
    }
}
