use std::marker::PhantomData;

use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

use bevy_mod_picking::{debug::DebugPickingMode, DefaultPickingPlugins};

use crate::{cad_core::builders::ParametricCad, prelude::WireFrameDisplaySettings};

use super::{
    events::{
        cad::GenerateCadModel,
        cursor::{CursorPointerMoveEvent, CursorPointerOutEvent, TransformCursorEvent},
    },
    systems::{
        cad::{
            cursor::{
                draw_cursor_gizmo, scale_cursors_based_on_zoom_level, transform_cursor,
                update_params_from_cursors,
            },
            mesh::{handle_mesh_selection, show_mesh_local_debug_axis},
            model::{
                generate_cad_model_on_event, update_cad_model_on_params_change_handle_task,
                update_cad_model_on_params_change_spawn_task,
            },
            outlines::generated_mesh_outlines,
            params_ui::{
                hide_params_display_ui_on_out_cursor, move_params_display_ui_on_transform_cursor,
                setup_param_display_ui, show_params_display_ui_on_hover_cursor,
            },
        },
        wire_frame::control_wire_frame_display,
    },
};

/// Base [`bevy`] [`Plugin`] for Parametric CAD Modelling.
#[derive(Default)]
pub struct ParametricCadModellingBasePlugin {
    /// Allows setting the wire-frame display on meshes via [`WireFramePlugin`].
    ///
    /// PS: **Only available in native environments.**
    pub allow_wire_frames: bool,
}

impl Plugin for ParametricCadModellingBasePlugin {
    fn build(&self, app: &mut App) {
        let Self {
            allow_wire_frames, ..
        } = *self;

        if allow_wire_frames {
            // Add wire frame setup only in native environment...
            #[cfg(not(target_arch = "wasm32"))]
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
        // Add all the plugins/systems/resources/events that are not specific to params...
        app // app
            // picking
            .add_plugins(DefaultPickingPlugins.build())
            .insert_resource(State::new(DebugPickingMode::Disabled)) // to disable debug overlay
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
                    generated_mesh_outlines,
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
                info!("ParametricCadModellingBasePlugin started!")
            });
    }
}

/// Parametric CAD Modelling [`bevy`] [`Plugin`].
///
/// Accepts [`Params`] (generic parameter) that should implement: [`ParametricCad`] + [`Component`].
///
/// The [`Plugin`] then allows generating CAD models using the passed [`Params`].
#[derive(Default)]
pub struct ParametricCadParamsPlugin<Params: ParametricCad + Component> {
    /// Owns the params type to prevent compiler complains.
    _params_type: PhantomData<Params>,
}

impl<Params: ParametricCad + Component + Clone> Plugin for ParametricCadParamsPlugin<Params> {
    fn build(&self, app: &mut App) {
        // now add param specific stuff...
        app // App
            // truck...
            .add_event::<GenerateCadModel<Params>>()
            .add_systems(
                Update,
                (
                    generate_cad_model_on_event::<Params>,
                    update_cad_model_on_params_change_spawn_task::<Params>,
                    update_cad_model_on_params_change_handle_task::<Params>,
                    update_params_from_cursors::<Params>,
                    show_params_display_ui_on_hover_cursor::<Params>,
                    move_params_display_ui_on_transform_cursor::<Params>,
                ),
            )
            // rest...
            .add_systems(Startup, || info!("ParametricCadParamsPlugin started!"));
    }
}
