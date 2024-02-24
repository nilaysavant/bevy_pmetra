use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_egui::EguiPlugin;

use super::{systems::fps_text_update_system, FpsDisplayPluginSettings};

/// # FPS Display Plugin
///
/// Plugin to display frames per second stat.
pub struct FpsDisplayPlugin;

impl Plugin for FpsDisplayPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        
        app // app
            .insert_resource(FpsDisplayPluginSettings::default())
            .add_systems(Update, fps_text_update_system)
            // rest...
            .add_systems(Startup, || info!("Starting FpsDisplayPlugin..."));
    }
}
