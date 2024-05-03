use bevy::prelude::*;
use bevy_pmetra::prelude::*;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new() // app
        .add_plugins(DefaultPlugins)
        // orbit camera...
        .add_plugins((LookTransformPlugin, OrbitCameraPlugin::default()))
        // scene...
        .add_systems(Startup, scene_setup)
        // rest...
        .add_systems(Startup, || info!("SimpleCube example started!"))
        .run();
}

pub fn scene_setup(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                near: 0.001,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert((
            OrbitCameraBundle::new(
                OrbitCameraController {
                    mouse_rotate_sensitivity: Vec2::splat(0.25),
                    mouse_translate_sensitivity: Vec2::splat(0.5),
                    mouse_wheel_zoom_sensitivity: 0.06,
                    smoothing_weight: 0.,
                    ..default()
                },
                Vec3::new(-2.0, 5.0, 5.0),
                Vec3::new(0., 0., 0.),
                Vec3::Y,
            ),
            CadCamera, // Mark the camera to be used for CAD.
        ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
