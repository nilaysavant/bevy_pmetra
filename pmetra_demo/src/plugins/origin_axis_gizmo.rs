use bevy::{color::palettes::css, prelude::*};

pub struct OriginAxisGizmoPlugin;

impl Plugin for OriginAxisGizmoPlugin {
    fn build(&self, app: &mut App) {
        app // App
            .insert_resource(OriginAxisGizmoSettings::default())
            .add_systems(Update, show_origin_gizmo)
            .add_systems(Startup, || info!("OriginAxisGizmoPlugin started..."));
    }
}

#[derive(Debug, Resource)]
pub struct OriginAxisGizmoSettings {
    pub hide_gizmos: bool,
    pub axis_length: f32,
}

impl Default for OriginAxisGizmoSettings {
    fn default() -> Self {
        Self {
            hide_gizmos: false,
            axis_length: 1.,
        }
    }
}

pub fn show_origin_gizmo(settings: Res<OriginAxisGizmoSettings>, mut gizmos: Gizmos) {
    if settings.hide_gizmos {
        return;
    }
    // x
    gizmos.line(Vec3::ZERO, Vec3::X * settings.axis_length, css::RED);
    // y
    gizmos.line(Vec3::ZERO, Vec3::Y * settings.axis_length, css::GREEN);
    // z
    gizmos.line(Vec3::ZERO, Vec3::Z * settings.axis_length, css::BLUE);
}
