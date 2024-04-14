use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashMap};

use crate::cad_core::builders::{CadShellName, ParametricCad};

use super::events::cad::SpawnMeshesBuilder;

#[derive(Debug, Default, Clone, Resource, Reflect, Deref, DerefMut)]
pub struct MeshesBuilderQueue<Params: ParametricCad + Component + Clone>(
    pub VecDeque<SpawnMeshesBuilder<Params>>,
);

#[derive(Debug, Default, Clone, Resource, Reflect, Deref, DerefMut)]
pub struct MeshesBuilderQueueInspector {
    pub meshes_builder_queue_size: usize,
}

#[derive(Debug, Default, Clone, Resource, Reflect, Deref, DerefMut)]
pub struct MeshesBuilderFinishedResultsMap<Params: ParametricCad + Component + Clone>(
    HashMap<(Entity, CadShellName), (Mesh, SpawnMeshesBuilder<Params>)>,
);
