use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::{FilterQueryInspectorPlugin, ResourceInspectorPlugin};
use bevy_rapier3d::prelude::*;

use bevy_pmetra::{pmetra_plugins::resources::MeshesBuilderQueueInspector, prelude::*};
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
        info_ui::{setup_info_ui, update_info_ui},
        inspector::toggle_inspector_is_active,
        orbit_cam::{fire_balls_at_look_point, orbit_cam_custom_input_map_controller},
        rapier::{control_debug_render, setup_debug_render},
        scene::scene_setup,
        window::close_on_esc,
    },
    utils::cad_models::{
        simple_primitives::simple_cube_at_cylinder::SimpleCubeAtCylinder,
        space_station::{
            round_cabin_segment::RoundCabinSegment, tower_extension::TowerExtension,
            RoundRectCuboid,
        },
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
                    ("pmetra_internal", "debug"),
                ],
                log_filter_release: vec![(env!("CARGO_PKG_NAME"), "info")],
            });
        }

        app // App
            // UI
            // EGUI init...
            .add_plugins(EguiPlugin {
                enable_multipass_for_primary_context: false,
            })
            // window...
            .add_systems(Update, close_on_esc)
            // ENVIRONMENT...
            .insert_resource(ClearColor(Color::BLACK))
            .insert_resource(DirectionalLightShadowMap { size: 4096 })
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 400.,
                ..Default::default()
            })
            // physics
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
            .add_plugins((
                PmetraBasePlugin {
                    allow_wire_frames: true,
                },
                // SimpleCubeAtCylinder
                PmetraModellingPlugin::<SimpleCubeAtCylinder>::default(),
                PmetraInteractionsPlugin::<SimpleCubeAtCylinder>::default(),
                // TowerExtension
                PmetraModellingPlugin::<TowerExtension>::default(),
                PmetraInteractionsPlugin::<TowerExtension>::default(),
                // RoundCabinSegment
                PmetraModellingPlugin::<RoundCabinSegment>::default(),
                PmetraInteractionsPlugin::<RoundCabinSegment>::default(),
            ))
            .init_resource::<CadGeneratedModelSpawner>()
            .register_type::<CadGeneratedModelSpawner>()
            .add_plugins(ResourceInspectorPlugin::<CadGeneratedModelSpawner>::default())
            .add_systems(Update, (spawn_cad_model, fire_balls_at_look_point))
            .add_systems(PostUpdate, add_collider_to_generated_cad_model)
            // scene...
            .add_systems(Startup, scene_setup)
            // info...
            .add_systems(Update, (setup_info_ui, update_info_ui))
            // exporter...
            .add_plugins(GltfExporterPlugin)
            // inspectors...
            .register_type::<SimpleCubeAtCylinder>()
            .register_type::<TowerExtension>()
            .register_type::<RoundCabinSegment>()
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
            // Settings Inspector
            .register_type::<PmetraGlobalSettings>()
            .add_plugins(ResourceInspectorPlugin::<PmetraGlobalSettings>::default())
            // rest...
            .add_systems(Startup, || info!("TruckIntegrationTestPlugin started!"));
    }
}
