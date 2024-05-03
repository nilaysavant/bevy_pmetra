use bevy::prelude::*;

use crate::prelude::PmetraGlobalSettings;

/// Custom Gizmo Config Group for Pmetra mesh outlines.
#[derive(Debug, Default, Reflect, GizmoConfigGroup)]
pub struct PmetraMeshOutlineGizmos;

/// Custom Gizmo Config Group for Pmetra slider outlines.
#[derive(Debug, Default, Reflect, GizmoConfigGroup)]
pub struct PmetraSliderOutlineGizmos;

pub fn configure_custom_gizmos(
    global_settings: Res<PmetraGlobalSettings>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    // Apply mesh outline width...
    let (gizmo_config, _) = config_store.config_mut::<PmetraMeshOutlineGizmos>();
    gizmo_config.line_width = global_settings.selected_mesh_outlines_width;
    // Apply slider outline width...
    let (gizmo_config, _) = config_store.config_mut::<PmetraSliderOutlineGizmos>();
    gizmo_config.line_width = global_settings.slider_outlines_width;
}
