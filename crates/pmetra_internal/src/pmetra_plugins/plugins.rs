use std::marker::PhantomData;

use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

use bevy_mod_picking::{debug::DebugPickingMode, picking_core, DefaultPickingPlugins};

use crate::{
    pmetra_plugins::components::{
        cad::CadGeneratedRootSelectionState, wire_frame::WireFrameDisplaySettings,
    },
    pmetra_core::builders::ParametricCad,
};

use super::{
    cleanup_manager::CleanupManagerPlugin,
    events::{
        cad::{GenerateCadModel, SpawnMeshesBuilder},
        cursor::{CursorPointerMoveEvent, CursorPointerOutEvent, TransformCursorEvent},
    },
    resources::{MeshesBuilderFinishedResultsMap, MeshesBuilderQueue, MeshesBuilderQueueInspector},
    systems::{
        cad::{
            cursor::{
                draw_cursor_gizmo, scale_cursors_based_on_zoom_level, transform_cursor,
                update_cursor_visibility_based_on_root_selection, update_params_from_cursors,
            },
            mesh::{
                handle_mesh_selection, show_mesh_local_debug_axis,
                update_root_selection_based_on_mesh_selection,
            },
            model::{
                handle_spawn_meshes_builder_events, mesh_builder_to_bundle, shells_to_cursors,
                shells_to_mesh_builder_events, spawn_shells_by_name_on_generate,
                update_shells_by_name_on_params_change,
            },
            outlines::generate_mesh_outlines,
            params_ui::{
                hide_params_display_ui_on_out_cursor, move_params_display_ui_on_transform_cursor,
                setup_param_display_ui, show_params_display_ui_on_hover_cursor,
            },
        },
        wire_frame::control_wire_frame_display,
    },
};

/// Base [`bevy`] [`Plugin`] for Interactive/Parametric/CAD modelling.
#[derive(Default)]
pub struct PmetraBasePlugin {
    /// Allows setting the wire-frame display on meshes via [`WireFramePlugin`].
    ///
    /// PS: **Only available in native environments.**
    pub allow_wire_frames: bool,
}

impl Plugin for PmetraBasePlugin {
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
                .insert_resource(DebugPickingMode::Disabled); // to disable debug overlay
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
                    update_root_selection_based_on_mesh_selection,
                    show_mesh_local_debug_axis,
                ),
            )
            // cursor systems...
            .add_systems(
                Update,
                (
                    (
                        update_cursor_visibility_based_on_root_selection,
                        transform_cursor,
                        scale_cursors_based_on_zoom_level,
                    )
                        .chain(),
                    draw_cursor_gizmo,
                ),
            )
            // cleanup...
            .add_plugins(CleanupManagerPlugin)
            // Register component types..
            .register_type::<CadGeneratedRootSelectionState>()
            // wire frame...
            .register_type::<WireFrameDisplaySettings>()
            .add_systems(
                Update,
                control_wire_frame_display.run_if(move || allow_wire_frames),
            )
            .add_systems(Startup, || {
                info!("PmetraBasePlugin started!")
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
            .add_event::<GenerateCadModel<Params>>()
            .add_event::<SpawnMeshesBuilder<Params>>()
            .init_resource::<MeshesBuilderQueue<Params>>()
            .init_resource::<MeshesBuilderQueueInspector>()
            .init_resource::<MeshesBuilderFinishedResultsMap<Params>>()
            // Generate Model systems...
            .add_systems(
                Update,
                (
                    // Model...
                    spawn_shells_by_name_on_generate::<Params>,
                    update_shells_by_name_on_params_change::<Params>,
                    shells_to_cursors::<Params>,
                    shells_to_mesh_builder_events::<Params>,
                    handle_spawn_meshes_builder_events::<Params>,
                    mesh_builder_to_bundle::<Params>,
                )
                    // chain seems to make the model update run more stable/smooth (less jittery).
                    .chain(),
            )
            // Cursors...
            .add_systems(Update, update_params_from_cursors::<Params>)
            // Params UI...
            .add_systems(
                Update,
                (
                    show_params_display_ui_on_hover_cursor::<Params>,
                    move_params_display_ui_on_transform_cursor::<Params>,
                ),
            )
            // rest...
            .add_systems(Startup, || info!("ParametricCadParamsPlugin started!"));
    }
}
