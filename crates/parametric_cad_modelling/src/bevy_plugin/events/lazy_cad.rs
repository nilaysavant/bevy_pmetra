use bevy::prelude::*;

use crate::cad_core::lazy_builders::ParametricLazyCad;

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
