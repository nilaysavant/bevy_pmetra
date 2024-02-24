use bevy::prelude::*;

mod plugin;
mod systems;

pub use plugin::FpsDisplayPlugin;

#[derive(Debug, Resource)]
pub struct FpsDisplayPluginSettings {
    pub show_fps: bool,
}

impl Default for FpsDisplayPluginSettings {
    fn default() -> Self {
        Self { show_fps: true }
    }
}
