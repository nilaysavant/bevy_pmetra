use anyhow::Result;
use bevy::prelude::*;

pub mod cursors;
pub mod meshes;
pub mod shells;
pub mod tags;

pub use {cursors::*, meshes::*, shells::*, tags::*};

/// Trait for parametrically generating [`CadShell`]s from struct.
pub trait ParametricModelling: Clone + Default {
    /// Gets the [`CadShellsBuilders`] for this params struct.
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>>;
}

/// Trait for parametrically generating models with cursors from struct.
pub trait ParametricCad: ParametricModelling {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesBuildersByCadShell<Self>>;

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
    ///
    /// Return `None` if no tooltip should be displayed.
    fn on_cursor_tooltip(&self, cursor_name: CadCursorName) -> Result<Option<String>>;
}

mod test {
    use crate::cad_core::builders::CadCursor;

    use super::*;

    #[test]
    pub fn test_basic_parametric_modelling_trait() {
        #[derive(Debug, Clone, Default)]
        pub struct Cube {
            pub width: f64,
        }

        impl ParametricModelling for Cube {
            fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
                CadShellsBuilders::default().add_shell_builder(
                    CadShellName("s1".into()),
                    |p: &Self| Ok(CadShell::default()),
                )
            }
        }

        impl ParametricCad for Cube {
            fn meshes_builders_by_shell(
                &self,
                shells_by_name: &CadShellsByName,
            ) -> Result<CadMeshesBuildersByCadShell<Self>> {
                CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                    .add_mesh_builder(
                        CadShellName("s1".into()),
                        "m1".into(),
                        CadMeshBuilder::new(self.clone(), CadShellName("s1".into()))?
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

            fn on_cursor_tooltip(&self, cursor_name: CadCursorName) -> Result<Option<String>> {
                Ok(None)
            }
        }

        pub fn build_cursor_c1(
            builder: &CadMeshBuilder<Cube>,
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
