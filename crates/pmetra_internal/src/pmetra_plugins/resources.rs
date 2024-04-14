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