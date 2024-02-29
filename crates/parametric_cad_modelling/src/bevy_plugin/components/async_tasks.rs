use std::marker::PhantomData;

use anyhow::Result;
use bevy::{prelude::*, tasks::Task};

use crate::cad_core::builders::CadMeshes;

/// Component for async task to compute CAD meshes.
#[derive(Component)]
pub struct ComputeCadMeshesTask<Params>(pub Task<ComputeCadMeshesResult<Params>>);

/// Component for async task to compute CAD meshes.
#[derive(Component)]
pub struct ComputeCadMeshesResult<Params> {
    pub cad_generated_root: Entity,
    pub cad_gen_meshes_result: Result<CadMeshes>,
    pub _phantom_data: PhantomData<Params>,
}
