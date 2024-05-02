use bevy::prelude::*;

use crate::prelude::PmetraGlobalSettings;

/// Custom Gizmo Config Group for Pmetra mesh outlines.
#[derive(Debug, Default, Reflect, GizmoConfigGroup)]
pub struct PmetraOutlineGizmos;

pub fn configure_custom_gizmos(
    global_settings: Res<PmetraGlobalSettings>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let (outline_gizmos_config, _) = config_store.config_mut::<PmetraOutlineGizmos>();
    outline_gizmos_config.line_width = global_settings.selected_mesh_outlines_width;
}
