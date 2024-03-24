use anyhow::Result;
use bevy::prelude::*;

use super::builders::{CadCursorName, CadMeshName, CadShell};

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
        shell_name: CadShellName,
        shell: CadShell,
    ) -> Result<CadMeshesLazyBuilder<Self>>;

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
                shell_name: CadShellName,
                cad_shell: CadShell,
            ) -> Result<CadMeshesLazyBuilder<Self>> {
                match shell_name.0.as_str() {
                    "s1" => {
                        let meshes_builder =
                            CadMeshesLazyBuilder::new(self.clone(), cad_shell.clone())?
                                .add_mesh_builder(
                                    "m1".into(),
                                    CadMeshLazyBuilder::new(self.clone(), cad_shell.clone())?
                                        .set_transform(Transform::default())?
                                        .set_base_material(StandardMaterial::default())?
                                        .set_outlines(cad_shell.shell.build_outlines())?
                                        .add_cursor("c1".into(), build_cursor_c1)?,
                                )?;
                        Ok(meshes_builder)
                    }
                    _ => Err(anyhow!("Could not find shell with name: {:?}", shell_name)),
                }
            }
            
            fn on_cursor_transform(
                &mut self,
                mesh_name: CadMeshName,
                cursor_name: CadCursorName,
                prev_transform: Transform,
                new_transform: Transform,
            ) {
                // TODO
            }
            
            fn on_cursor_tooltip(
                &self,
                mesh_name: CadMeshName,
                cursor_name: CadCursorName,
            ) -> Result<String> {
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
