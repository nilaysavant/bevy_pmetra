use bevy::{pbr::DirectionalLightShadowMap, prelude::*, window::close_on_esc};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::{
    FilterQueryInspectorPlugin, ResourceInspectorPlugin, WorldInspectorPlugin,
};
use bevy_rapier3d::prelude::*;

use bevy_pmetra::{
    bevy_plugin::{
        lazy_plugins::{ParametricLazyCadModellingBasePlugin, ParametricLazyCadParamsPlugin},
        resources::MeshesBuilderQueueInspector,
    },
    prelude::*,
};
use smooth_bevy_cameras::{controllers::orbit::OrbitCameraPlugin, LookTransformPlugin};

use crate::{
    plugins::{
        fps_display::FpsDisplayPlugin,
        gltf_exporter::plugin::GltfExporterPlugin,
        my_bevy_defaults::{MyBevyCustomLogPlugin, MyBevyDefaultPluginsConfig},
        origin_axis_gizmo::OriginAxisGizmoPlugin,
    },
    resources::CadGeneratedModelSpawner,
    systems::{
        cad::{add_collider_to_generated_cad_model, spawn_cad_model},
        inspector::toggle_inspector_is_active,
        orbit_cam::{fire_balls_at_look_point, orbit_cam_custom_input_map_controller},
        rapier::{control_debug_render, setup_debug_render},
        scene::{scene_setup, test_manual_mesh_gen},
    },
    utils::cad_models::{
        mechanical_parts::simple_gear::SimpleGear,
        simple_primitives::{
            simple_cube_at_cylinder::SimpleCubeAtCylinder,
            simple_lazy_cube_at_cylinder::SimpleLazyCubeAtCylinder,
        },
        space_station::{lazy_tower_extension::LazyTowerExtension, RoundCabinSegment, RoundRectCuboid},
    },
};

pub struct PmetraDemoPlugin;

impl Plugin for PmetraDemoPlugin {
    fn build(&self, app: &mut App) {
        // Init my bevy defaults...
        app.add_plugins(
            MyBevyDefaultPluginsConfig {
                game_title: "Pmetra Demo".to_string(),
                #[cfg(not(target_arch = "wasm32"))]
                enable_polygon_line_mode: true,
                ..default()
            }
            .get_builder(), // .set(low_latency_window_plugin())
        );
        if !app.is_plugin_added::<MyBevyCustomLogPlugin>() {
            app.add_plugins(MyBevyCustomLogPlugin {
                log_filter_debug: vec![
                    (env!("CARGO_PKG_NAME"), "debug"),
                    ("bevy_pmetra", "debug"),
                    ("parametric_cad_modelling", "debug"),
                ],
                log_filter_release: vec![(env!("CARGO_PKG_NAME"), "info")],
            });
        }

        app // App
            // UI
            // EGUI init...
            .add_plugins(EguiPlugin)
            // window...
            .add_systems(Update, close_on_esc)
            // ENVIRONMENT...
            .insert_resource(ClearColor(Color::BLACK))
            .insert_resource(DirectionalLightShadowMap { size: 4096 })
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 400.,
            })
            // physics
            .insert_resource(RapierConfiguration::default())
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(Startup, setup_debug_render)
            .add_systems(Update, control_debug_render)
            // Utils and Camera Plugins...
            .add_plugins((FpsDisplayPlugin, OriginAxisGizmoPlugin))
            .add_plugins((
                LookTransformPlugin,
                OrbitCameraPlugin {
                    override_input_system: true,
                },
            ))
            .add_systems(Update, orbit_cam_custom_input_map_controller)
            // cad...
            // .add_plugins((
            //     ParametricCadModellingBasePlugin {
            //         allow_wire_frames: true,
            //     },
            //     ParametricCadParamsPlugin::<SimpleCubeAtCylinder>::default(),
            //     ParametricCadParamsPlugin::<RoundCabinSegment>::default(),
            //     ParametricCadParamsPlugin::<SimpleGear>::default(),
            // ))
            .add_plugins((
                ParametricLazyCadModellingBasePlugin {
                    allow_wire_frames: true,
                },
                ParametricLazyCadParamsPlugin::<SimpleLazyCubeAtCylinder>::default(),
                ParametricLazyCadParamsPlugin::<LazyTowerExtension>::default(),
            ))
            .init_resource::<CadGeneratedModelSpawner>()
            .register_type::<CadGeneratedModelSpawner>()
            .add_plugins(ResourceInspectorPlugin::<CadGeneratedModelSpawner>::default())
            .add_systems(Update, (spawn_cad_model, fire_balls_at_look_point))
            .add_systems(
                PostUpdate,
                add_collider_to_generated_cad_model::<SimpleLazyCubeAtCylinder>,
            )
            // scene...
            .add_systems(
                Startup,
                (
                    //
                    scene_setup,
                    // test_manual_mesh_gen,
                ),
            )
            // exporter
            .add_plugins(GltfExporterPlugin)
            // inspectors...
            .register_type::<RoundCabinSegment>()
            .register_type::<SimpleCubeAtCylinder>()
            .register_type::<SimpleLazyCubeAtCylinder>()
            .register_type::<LazyTowerExtension>()
            .register_type::<SimpleGear>()
            .register_type::<RoundRectCuboid>()
            .add_plugins(
                FilterQueryInspectorPlugin::<With<CadGeneratedRoot>>::default()
                    .run_if(toggle_inspector_is_active),
            )
            // Queue Inspector
            .register_type::<MeshesBuilderQueueInspector>()
            .add_plugins(
                ResourceInspectorPlugin::<MeshesBuilderQueueInspector>::default()
                    .run_if(toggle_inspector_is_active),
            )
            .add_plugins(WorldInspectorPlugin::default().run_if(toggle_inspector_is_active))
            // rest...
            .add_systems(Startup, || info!("TruckIntegrationTestPlugin started!"));
    }
}
