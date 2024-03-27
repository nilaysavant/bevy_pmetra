use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    egui::{self, epaint::Shadow, Color32, Margin, RichText, Rounding, Stroke},
    EguiContexts,
};

use super::FpsDisplayPluginSettings;

pub fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut egui_contexts: EguiContexts,
    settings: Res<FpsDisplayPluginSettings>,
) {
    if !settings.show_fps {
        return;
    }
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        let Some(avg_fps) = fps.average() else {
            return;
        };
        // println!("fps: {}", avg_fps);
        let frame = get_default_egui_frame();
        egui::Window::new("FPS Display")
            .title_bar(false)
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .frame(frame)
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
            .show(egui_contexts.ctx_mut(), |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(format!("FPS: {:.0}", avg_fps)));
                });
            });
    }
}

pub fn get_default_egui_frame() -> egui::Frame {
    let frame = egui::Frame {
        rounding: Rounding::ZERO,
        shadow: Shadow::NONE,
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 200),
        stroke: Stroke::NONE,
        inner_margin: Margin::symmetric(3.0, 3.0),
        outer_margin: Margin::symmetric(3.0, 3.0),
    };
    frame
}
