use bevy::{
    app::PluginGroupBuilder,
    log::{Level, LogPlugin},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
    utils::HashMap,
    window::PresentMode,
};

/// Html Canvas selector
const CANVAS_SELECTOR: &str = "#bevy";

/// My custom config for [`DefaultPlugins`] building.
#[derive(Debug)]
pub struct MyBevyDefaultPluginsConfig {
    /// Title of the game.
    pub game_title: String,
    /// Canvas selector used for rendering the wasm game in.
    pub wasm_canvas_selector: Option<String>,
    /// Init present mode of the game.
    pub present_mode: PresentMode,
    /// Used for wire frame display. Warning! Does not work in web/wasm.
    pub enable_polygon_line_mode: bool,
}

impl Default for MyBevyDefaultPluginsConfig {
    fn default() -> Self {
        Self {
            game_title: "Bevy game".to_string(),
            wasm_canvas_selector: Some(CANVAS_SELECTOR.to_string()),
            present_mode: PresentMode::AutoVsync,
            enable_polygon_line_mode: false,
        }
    }
}

impl MyBevyDefaultPluginsConfig {
    /// Get [`DefaultPlugins`] builder.
    pub fn get_builder(&self) -> PluginGroupBuilder {
        let default_plugin = DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: self.game_title.clone(),
                    canvas: self.wasm_canvas_selector.clone(),
                    present_mode: self.present_mode,
                    ..default()
                }),
                ..default()
            })
            .disable::<LogPlugin>();
        let mut render_plugin = RenderPlugin::default();
        if self.enable_polygon_line_mode {
            render_plugin.render_creation = RenderCreation::Automatic(WgpuSettings {
                // WARN this is a native only feature. It will not work with webgl or webgpu
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            });
        }

        default_plugin.set(render_plugin)
    }
}

/// My custom default Bevy [`Plugin`].
///
/// Used for default plugin init via given builder.
pub struct MyBevyCustomLogPlugin {
    /// Additive Log filter for `debug` mode.
    pub log_filter_debug: Vec<(&'static str, &'static str)>,
    /// Additive Log filter for `release` mode.
    pub log_filter_release: Vec<(&'static str, &'static str)>,
}

impl Plugin for MyBevyCustomLogPlugin {
    fn build(&self, app: &mut App) {
        // Extend the log filter with values from plugin input...
        let mut log_filter_debug_map = HashMap::from([
            (env!("CARGO_PKG_NAME"), "debug"),
            ("wgpu", "error"),
            ("bevy_render", "error"),
            ("bevy_ecs", "info"),
        ]);
        log_filter_debug_map.extend(self.log_filter_debug.clone());

        let mut log_filter_release_map = HashMap::from([
            (env!("CARGO_PKG_NAME"), "info"),
            ("wgpu", "error"),
            ("bevy_render", "error"),
            ("bevy_ecs", "error"),
        ]);
        log_filter_release_map.extend(self.log_filter_release.clone());

        // this code is compiled only if debug assertions are enabled (debug mode)
        #[cfg(debug_assertions)]
        app.add_plugins(LogPlugin {
            level: Level::INFO,
            filter: log_filter_debug_map
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(","),
            ..Default::default()
        });

        // this code is compiled only if debug assertions are disabled (release mode)
        #[cfg(not(debug_assertions))]
        app.add_plugins(LogPlugin {
            level: Level::WARN,
            filter: log_filter_release_map
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(","),
            ..Default::default()
        });
    }
}
