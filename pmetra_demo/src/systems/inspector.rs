use bevy::prelude::*;

/// Run condition used to toggle inspector show/hide based on a
/// standardized input key.
///
/// - Toggles the `is_active` output state based on `F2` keypress.
/// - Internally checks if Debug is enabled on the app. Useful for disabling in prod release.
///
/// ### Usage:
/// ```ignore
/// app // App
///     .init_resource::<Configuration>()
///     .register_type::<Configuration>()
///     .add_plugins(
///         ResourceInspectorPlugin::<Configuration>::default()
///             .run_if(toggle_inspector_is_active) // <-- Add this run condition here
///     );
/// ```
///
pub fn toggle_inspector_is_active(
    input: Res<ButtonInput<KeyCode>>,
    mut is_active: Local<bool>,
) -> bool {
    // toggle `is_active` if debug tools is enabled.
    if input.just_pressed(KeyCode::F2) {
        *is_active = !*is_active;
    }
    *is_active
}
