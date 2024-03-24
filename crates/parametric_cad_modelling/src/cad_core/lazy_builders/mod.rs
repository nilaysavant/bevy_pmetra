use anyhow::Result;
use bevy::prelude::*;

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
pub trait ParametricLazyCad: ParametricLazyModelling {}

mod test {
    use crate::cad_core::builders::CadShell;

    use super::*;

    #[test]
    pub fn test_basic_parametric_modelling_trait() {
        #[derive(Debug, Clone, Default)]
        pub struct Cube {
            pub width: f64,
        }

        impl ParametricLazyModelling for Cube {
            fn shells_builders(&self) -> Result<CadShellsLazyBuilders<Self>> {
                CadShellsLazyBuilders::default()
                    .add_shell_builder(CadShellName("a".into()), |p: &Self| Ok(CadShell::default()))
            }
        }

        let cube = Cube { width: 1. };
        let cube_shell_builders = cube.shells_builders().unwrap();
        let build_result = (cube_shell_builders
            .builders
            .get(&CadShellName("a".into()))
            .unwrap()
            .build_cad_shell)(&cube)
        .unwrap();
    }
}
