use bevy::prelude::*;

use crate::{cad_core::lazy_builders::{CadMeshesLazyBuilder, CadShellName, ParametricLazyCad}, prelude::BelongsToCadGeneratedRoot};

/// Event when fired, **generates CAD Model** for the passed [`Params`].
///
/// Prerequisites:
/// - Add plugin: [`ParametricCadModellingPlugin<Params>`].
#[derive(Debug, Event, Reflect)]
pub struct GenerateLazyCadModel<Params: ParametricLazyCad + Component> {
    /// Params used to generate cad model.
    pub params: Params,
    /// Remove any existing models generated with these [`Params`].
    pub remove_existing_models: bool,
}

impl<Params: ParametricLazyCad + Component + Default> Default for GenerateLazyCadModel<Params> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            remove_existing_models: true,
        }
    }
}

/// Event used to spawn individual mesh builders for parallel meshing.
#[derive(Debug, Event, Reflect, Clone)]
pub struct SpawnMeshesBuilder<Params: ParametricLazyCad + Component> {
    pub belongs_to_root: BelongsToCadGeneratedRoot,
    pub shell_name: CadShellName,
    pub meshes_builder: CadMeshesLazyBuilder<Params>,
}
