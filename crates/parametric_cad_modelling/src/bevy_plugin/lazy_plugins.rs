use std::marker::PhantomData;

use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

use bevy_mod_picking::{debug::DebugPickingMode, picking_core, DefaultPickingPlugins};

use crate::{
    bevy_plugin::components::wire_frame::WireFrameDisplaySettings,
    cad_core::lazy_builders::ParametricLazyCad,
};

use super::{
    events::{
        cursor::{CursorPointerMoveEvent, CursorPointerOutEvent, TransformCursorEvent},
        lazy_cad::{GenerateLazyCadModel, SpawnMeshesBuilder},
    },
    systems::{
        cad::{
            cursor::{draw_cursor_gizmo, scale_cursors_based_on_zoom_level, transform_cursor},
            mesh::{handle_mesh_selection, show_mesh_local_debug_axis},
            outlines::generate_mesh_outlines,
            params_ui::{hide_params_display_ui_on_out_cursor, setup_param_display_ui},
        },
        lazy_cad::model::{
            handle_spawn_meshes_builder_events, mesh_builder_to_bundle, mesh_builder_to_cursors,
            shells_to_mesh_builder_events, spawn_shells_lazy_builders_on_generate,
        },
        wire_frame::control_wire_frame_display,
    },
};

/// Base [`bevy`] [`Plugin`] for Parametric CAD Modelling.
#[derive(Default)]
pub struct ParametricLazyCadModellingBasePlugin {
    /// Allows setting the wire-frame display on meshes via [`WireFramePlugin`].
    ///
    /// PS: **Only available in native environments.**
    pub allow_wire_frames: bool,
}

impl Plugin for ParametricLazyCadModellingBasePlugin {
    fn build(&self, app: &mut App) {
        let Self {
            allow_wire_frames, ..
        } = *self;

        if allow_wire_frames {
            // Add wire frame setup only in native environment...
            #[cfg(not(target_arch = "wasm32"))]
            if !app.is_plugin_added::<WireframePlugin>() {
                app.add_plugins(
                    // You need to add this plugin to enable wireframe rendering
                    WireframePlugin,
                )
                // Wireframes can be configured with this resource. This can be changed at runtime.
                .insert_resource(WireframeConfig {
                    // The global wireframe config enables drawing of wireframes on every mesh,
                    // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
                    // regardless of the global configuration.
                    global: false,
                    // Controls the default color of all wireframes. Used as the default color for global wireframes.
                    // Can be changed per mesh using the `WireframeColor` component.
                    default_color: Color::YELLOW,
                });
            }
        }

        if !app.is_plugin_added::<picking_core::CorePlugin>() {
            app // picking
                .add_plugins(DefaultPickingPlugins.build())
                .insert_resource(State::new(DebugPickingMode::Disabled)); // to disable debug overlay
        }

        // Add all the plugins/systems/resources/events that are not specific to params...
        app // app
            // picking events...
            .add_event::<TransformCursorEvent>()
            .add_event::<CursorPointerMoveEvent>()
            .add_event::<CursorPointerOutEvent>()
            // UI for params and dimensions...
            .add_systems(
                Update,
                (setup_param_display_ui, hide_params_display_ui_on_out_cursor),
            )
            // mesh systems...
            .add_systems(
                Update,
                (
                    generate_mesh_outlines,
                    handle_mesh_selection,
                    show_mesh_local_debug_axis,
                ),
            )
            // cursor systems...
            .add_systems(
                Update,
                (
                    (
                        //
                        transform_cursor,
                        scale_cursors_based_on_zoom_level,
                    )
                        .chain(),
                    draw_cursor_gizmo,
                ),
            )
            // wire frame...
            .register_type::<WireFrameDisplaySettings>()
            .add_systems(
                Update,
                control_wire_frame_display.run_if(move || allow_wire_frames),
            )
            .add_systems(Startup, || {
                info!("ParametricLazyCadModellingBasePlugin started!")
            });
    }
}

/// Parametric CAD Modelling [`bevy`] [`Plugin`].
///
/// Accepts [`Params`] (generic parameter) that should implement: [`ParametricCad`] + [`Component`].
///
/// The [`Plugin`] then allows generating CAD models using the passed [`Params`].
#[derive(Default)]
pub struct ParametricLazyCadParamsPlugin<Params: ParametricLazyCad + Component> {
    /// Owns the params type to prevent compiler complains.
    _params_type: PhantomData<Params>,
}

impl<Params: ParametricLazyCad + Component + Clone> Plugin
    for ParametricLazyCadParamsPlugin<Params>
{
    fn build(&self, app: &mut App) {
        // now add param specific stuff...
        app // App
            // truck...
            .add_event::<GenerateLazyCadModel<Params>>()
            .add_event::<SpawnMeshesBuilder<Params>>()
            .add_systems(
                Update,
                (
                    spawn_shells_lazy_builders_on_generate::<Params>,
                    shells_to_mesh_builder_events::<Params>,
                    handle_spawn_meshes_builder_events::<Params>,
                    mesh_builder_to_bundle::<Params>,
                    mesh_builder_to_cursors::<Params>,
                ),
            )
            // rest...
            .add_systems(Startup, || info!("ParametricLazyCadParamsPlugin started!"));
    }
}
