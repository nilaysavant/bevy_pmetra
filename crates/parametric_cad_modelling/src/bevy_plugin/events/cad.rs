use bevy::prelude::*;

use crate::{
    cad_core::builders::{CadMaterialTextures, ParametricCad},
    prelude::ParametricCadParamsPlugin,
};

/// Event when fired, **generates CAD Model** for the passed [`Params`].
///
/// Prerequisites:
/// - Add plugin: [`ParametricCadModellingPlugin<Params>`].
#[derive(Debug, Event, Reflect)]
pub struct GenerateCadModel<Params: ParametricCad + Component> {
    /// Params used to generate cad model.
    pub params: Params,
    /// Textures used to generate materials for cad model.
    pub textures: CadMaterialTextures<Option<Handle<Image>>>,
    /// Remove any existing models generated with these [`Params`].
    pub remove_existing_models: bool,
}

impl<Params: ParametricCad + Component + Default> Default for GenerateCadModel<Params> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            textures: Default::default(),
            remove_existing_models: true,
        }
    }
}
