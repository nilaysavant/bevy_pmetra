use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashMap};

use crate::pmetra_core::builders::{CadShellName, PmetraModelling};

use super::events::cad::SpawnMeshesBuilder;

#[derive(Debug, Default, Clone, Resource, Reflect, Deref, DerefMut)]
pub struct MeshesBuilderQueue<Params: PmetraModelling + Component + Clone>(
    pub VecDeque<SpawnMeshesBuilder<Params>>,
);

#[derive(Debug, Default, Clone, Resource, Reflect, Deref, DerefMut)]
pub struct MeshesBuilderQueueInspector {
    pub meshes_builder_queue_size: usize,
}

#[derive(Debug, Default, Clone, Resource, Reflect, Deref, DerefMut)]
pub struct MeshesBuilderFinishedResultsMap<Params: PmetraModelling + Component + Clone>(
    HashMap<(Entity, CadShellName), (Mesh, SpawnMeshesBuilder<Params>)>,
);

/// Global Settings for Pmetra.
#[derive(Debug, Clone, Resource, Reflect)]
pub struct PmetraGlobalSettings {
    /// Show selected mesh's local axis/orientation.
    ///
    /// Useful for debugging.
    pub show_selected_mesh_local_debug_axis: bool,
    /// Show outlines of selected mesh.
    pub show_selected_mesh_outlines: bool,
    /// Width of the mesh outlines.
    pub selected_mesh_outlines_width: f32,
    /// Width of the slider outlines.
    pub slider_outlines_width: f32,
    /// Size of the slider drag plane. 
    /// 
    /// This limits the slider drag distance.
    pub slider_drag_plane_size: f32,
    /// Show slider drag plane for debugging.
    pub slider_drag_plane_debug: bool,
}

impl Default for PmetraGlobalSettings {
    fn default() -> Self {
        Self {
            show_selected_mesh_local_debug_axis: true,
            show_selected_mesh_outlines: true,
            selected_mesh_outlines_width: 1.25,
            slider_outlines_width: 1.25,
            slider_drag_plane_size: 100.,
            slider_drag_plane_debug: false,
        }
    }
}
