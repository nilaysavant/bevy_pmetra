use bevy::prelude::*;

use crate::{
    pmetra_core::builders::{CadMeshesBuilder, CadShellName, PmetraModelling},
    prelude::BelongsToCadGeneratedRoot,
};

/// Event when fired, **generates CAD Model** for the passed [`Params`].
///
/// Prerequisites:
/// - Add plugin: [`PmetraModelling<Params>`].
#[derive(Debug, Event, Reflect)]
pub struct GenerateCadModel<Params: PmetraModelling + Component> {
    /// Params used to generate cad model.
    pub params: Params,
    /// Transform to apply to the (root of) generated model.
    pub transform: Transform,
    /// Remove any existing models generated with these [`Params`].
    pub remove_existing_models: bool,
}

impl<Params: PmetraModelling + Component + Default> Default for GenerateCadModel<Params> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            transform: Transform::default(),
            remove_existing_models: true,
        }
    }
}

/// Event used to spawn individual mesh builders for parallel meshing.
#[derive(Debug, Event, Reflect, Clone)]
pub struct SpawnMeshesBuilder<Params: PmetraModelling + Component> {
    pub belongs_to_root: BelongsToCadGeneratedRoot,
    pub shell_name: CadShellName,
    pub meshes_builder: CadMeshesBuilder<Params>,
    /// Index count at the time of creation. Used to check for the latest mesh build.
    pub created_at_idx: usize,
}
