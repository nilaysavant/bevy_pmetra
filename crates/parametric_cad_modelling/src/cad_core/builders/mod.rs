use anyhow::{Context, Result};
use bevy::prelude::*;

pub mod cursors;
/// Unused Interactive Faces Module.
#[deprecated]
mod faces;
/// Materials Builders.
pub mod materials;
/// Meshes Builders.
pub mod meshes;
/// Shells builders.
pub mod shells;
pub mod tags;

pub use {cursors::*, materials::*, meshes::*, shells::*, tags::*};

/// Trait for parametrically generating models from struct.
pub trait ParametricModelling {
    /// Tries building [`CadShells`] for given parametric struct.
    fn build_shells(&self) -> Result<CadShells>;
}

/// Trait to generate CAD like interface (with [`FaceCursor`]s) for [`ParametricSolid`].
pub trait ParametricCad: ParametricModelling {
    fn build_cad_meshes_from_shells(&self, shells: CadShells) -> Result<CadMeshes>;
    fn build_cad_meshes(&self) -> Result<CadMeshes> {
        let cad_shells = self
            .build_shells()
            .with_context(|| "Could not build CadShells for parametric struct!")?;

        self.build_cad_meshes_from_shells(cad_shells)
    }
    /// Handler called whenever a cursor is Transformed.
    fn on_cursor_transform(
        &mut self,
        mesh_name: CadMeshName,
        cursor_name: CadCursorName,
        prev_transform: Transform,
        new_transform: Transform,
    );
    /// Handler called to get [`CadCursor`] tooltip UI text.
    fn on_cursor_tooltip(
        &self,
        mesh_name: CadMeshName,
        cursor_name: CadCursorName,
    ) -> Result<String>;
}

mod tests {
    use bevy::utils::hashbrown::HashMap;

    use super::*;

    #[test]
    pub fn test_cad_solid_builder() -> Result<()> {
        #[derive(Debug, Default, Clone)]
        pub struct Params {
            width: f64,
        }

        let params = Params { width: 1. };
        let cad_shells_builder = CadShellsBuilder::<Params>::default()
            .add_shell("Main".into(), build_main_solid)?
            .add_shell("Second".into(), build_second_solid)?;

        pub fn build_main_solid(builder: &CadShellsBuilder<Params>) -> Result<CadShell> {
            Ok(CadShell::default())
        }
        pub fn build_second_solid(builder: &CadShellsBuilder<Params>) -> Result<CadShell> {
            Ok(CadShell::default())
        }
        println!("cad_shells_builder: {:?}", cad_shells_builder);

        assert_eq!(cad_shells_builder.shells.len(), 2);

        Ok(())
    }
}
