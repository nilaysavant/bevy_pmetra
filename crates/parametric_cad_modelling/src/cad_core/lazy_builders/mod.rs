use anyhow::Result;
use bevy::{prelude::*, utils::HashMap};

use super::builders::{CadCursor, CadCursorName, CadCursors, CadMeshName, CadShell};

/// Meshes builders.
pub mod meshes;
/// Shells builders.
pub mod shells;

pub use {meshes::*, shells::*};

/// Trait for parametrically generating [`CadShell`]s from struct.
pub trait ParametricLazyModelling: Clone + Default {
    /// Gets the [`CadShellsLazyBuilders`] for this params struct.
    fn shells_builders(&self) -> Result<CadShellsLazyBuilders<Self>>;
}

/// Trait for parametrically generating models with cursors from struct.
pub trait ParametricLazyCad: ParametricLazyModelling {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesLazyBuildersByCadShell<Self>>;

    /// Configure Cursors.
    fn cursors(&self, shells_by_name: &CadShellsByName) -> Result<CadCursors>;

    /// Handler called whenever a cursor is Transformed.
    fn on_cursor_transform(
        &mut self,
        cursor_name: CadCursorName,
        prev_transform: Transform,
        new_transform: Transform,
    );
    /// Handler called to get [`CadCursor`] tooltip UI text.
    fn on_cursor_tooltip(&self, cursor_name: CadCursorName) -> Result<String>;
}

mod test {
    use anyhow::anyhow;

    use crate::cad_core::builders::{BuildCadMeshOutlines, CadCursor, CadMeshOutlines, CadShell};

    use super::*;

    #[test]
    pub fn test_basic_parametric_modelling_trait() {
        #[derive(Debug, Clone, Default)]
        pub struct Cube {
            pub width: f64,
        }

        impl ParametricLazyModelling for Cube {
            fn shells_builders(&self) -> Result<CadShellsLazyBuilders<Self>> {
                CadShellsLazyBuilders::default().add_shell_builder(
                    CadShellName("s1".into()),
                    |p: &Self| Ok(CadShell::default()),
                )
            }
        }

        impl ParametricLazyCad for Cube {
            fn meshes_builders_by_shell(
                &self,
                shells_by_name: &CadShellsByName,
            ) -> Result<CadMeshesLazyBuildersByCadShell<Self>> {
                CadMeshesLazyBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                    .add_mesh_builder(
                        CadShellName("s1".into()),
                        "m1".into(),
                        CadMeshLazyBuilder::new(self.clone(), CadShellName("s1".into()))?
                            .set_transform(Transform::default())?
                            .set_base_material(StandardMaterial::default())?,
                    )
            }

            fn cursors(&self, shells_by_name: &CadShellsByName) -> Result<CadCursors> {
                let mut cursors = CadCursors::default();
                cursors.insert(CadCursorName("c1".into()), CadCursor::default());

                Ok(cursors)
            }

            fn on_cursor_transform(
                &mut self,
                cursor_name: CadCursorName,
                prev_transform: Transform,
                new_transform: Transform,
            ) {
                // TODO
            }

            fn on_cursor_tooltip(&self, cursor_name: CadCursorName) -> Result<String> {
                // TODO
                Ok("Test".into())
            }
        }

        pub fn build_cursor_c1(
            builder: &CadMeshLazyBuilder<Cube>,
            shell: &CadShell,
        ) -> Result<CadCursor> {
            Ok(CadCursor::default())
        }

        let cube = Cube { width: 1. };
        let cube_shell_builders = cube.shells_builders().unwrap();
        let build_result = (cube_shell_builders
            .builders
            .get(&CadShellName("s1".into()))
            .unwrap()
            .build_cad_shell)(&cube)
        .unwrap();
    }
}
