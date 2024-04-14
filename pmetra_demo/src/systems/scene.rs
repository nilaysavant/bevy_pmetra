use bevy::{math::DVec3, pbr::wireframe::Wireframe, prelude::*};
use bevy_pmetra::{
    prelude::*,
    re_exports::{
        truck_meshalgo::{
            filters::OptimizingFilter,
            tessellation::{MeshableShape, MeshedShape},
        },
        truck_modeling::{builder, cgmath::Vector3},
    },
};
use smooth_bevy_cameras::controllers::orbit::{OrbitCameraBundle, OrbitCameraController};

use crate::utils::cad_models::space_station::common::{
    get_corner_arcs_for_corner_vertices, get_profile_from_corner_arcs,
};

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

pub fn test_manual_mesh_gen(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // alignment vertices...
    let a0 = DVec3::ZERO;
    let a1 = a0 + DVec3::X * 1.0;
    let a2 = a1 + DVec3::Y * 1.0;
    let a3 = a2 - DVec3::X * 1.0;

    let (arc0, arc1, arc2, arc3) = get_corner_arcs_for_corner_vertices(a0, a1, a2, a3, 0.2);
    // connect all arcs with intermediate wires and create a profile...
    let profile = get_profile_from_corner_arcs(&arc0, &arc1, &arc2, &arc3).unwrap();

    let shell = builder::tsweep(&profile, Vector3::unit_z());

    let mut polygon_mesh = shell.triangulation(CUSTOM_TRUCK_TOLERANCE_1).to_polygon();
    polygon_mesh.remove_degenerate_faces().remove_unused_attrs();

    let mesh = polygon_mesh.build_mesh();

    // spawn mesh...
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::GREEN),
            transform: Transform::from_translation(Vec3::X * 3.),
            ..default()
        },
        Wireframe,
    ));
}
