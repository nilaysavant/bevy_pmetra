use bevy::prelude::*;

use crate::pmetra_plugins::resources::PmetraGlobalSettings;

/// Run condition for showing the selected mesh's local axis/orientation for debugging.
pub fn show_selected_mesh_local_debug_axis(global_settings: Res<PmetraGlobalSettings>) -> bool {
    global_settings.show_selected_mesh_local_debug_axis
}

/// Run condition for showing the selected mesh's outlines.
pub fn show_selected_mesh_outlines(global_settings: Res<PmetraGlobalSettings>) -> bool {
    global_settings.show_selected_mesh_outlines
}
