use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use smooth_bevy_cameras::{
    controllers::orbit::{ControlEvent, OrbitCameraController},
    LookTransform,
};

pub fn orbit_cam_custom_input_map_controller(
    mut events: EventWriter<ControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    controllers: Query<&OrbitCameraController>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        mouse_wheel_zoom_sensitivity,
        pixels_per_line,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        cursor_delta += event.delta;
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        if keyboard.pressed(KeyCode::ShiftLeft) {
            events.send(ControlEvent::TranslateTarget(
                mouse_translate_sensitivity * cursor_delta,
            ));
        } else {
            events.send(ControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));
        }
    }

    let mut scalar = 1.0;
    for event in mouse_wheel_reader.read() {
        // scale the event magnitude per pixel or per line
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / pixels_per_line,
        };
        scalar *= 1.0 - scroll_amount * mouse_wheel_zoom_sensitivity;
    }
    events.send(ControlEvent::Zoom(scalar));
}

const IMPULSE_MAG: f32 = 0.0007;
const BULLET_SPHERE_RADIUS: f32 = 0.03;

pub fn fire_balls_at_look_point(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query_orbit_cam: Query<&LookTransform, With<OrbitCameraController>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }
    let Ok(LookTransform { eye, target, .. }) = query_orbit_cam.get_single() else {
        return;
    };

    let impulse_dir = (*target - *eye).normalize();
    let ext_impulse = ExternalImpulse {
        impulse: impulse_dir * IMPULSE_MAG,
        ..default()
    };

    debug!("Spawning bullet ball...");
    // Spawn bullet ball...
    commands.spawn((
        Mesh3d(meshes.add(Sphere {
            radius: BULLET_SPHERE_RADIUS,
        })),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_translation(*eye),
        RigidBody::Dynamic,
        Collider::ball(BULLET_SPHERE_RADIUS),
        Ccd::enabled(),
        ext_impulse,
    ));
}
