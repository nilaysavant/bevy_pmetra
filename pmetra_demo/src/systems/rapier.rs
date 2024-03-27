use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Inits the debug render.
pub fn setup_debug_render(mut debug_render: ResMut<DebugRenderContext>) {
    debug_render.enabled = false;
}

/// Controls debug render via keybind.
pub fn control_debug_render(
    mut debug_render: ResMut<DebugRenderContext>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    if key_input.just_pressed(KeyCode::F2) {
        debug_render.enabled = !debug_render.enabled
    }
}
