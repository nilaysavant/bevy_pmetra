use anyhow::Result;
use bevy::prelude::*;

pub mod meshes;
pub mod shells;
pub mod sliders;
pub mod tags;

pub use {meshes::*, shells::*, sliders::*, tags::*};

/// Used for generating [`CadShell`]s using this struct via `truck`'s modelling APIs.
pub trait PmetraCad: Clone + Default {
    /// Get the [`CadShellsBuilders`] for this params struct.
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>>;
}

/// Trait for parametrically generating [`Mesh`]s from struct.
pub trait PmetraModelling: PmetraCad {
    /// Configure the [`CadMesh`]s to be generated for each of the [`CadShell`]s.
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesBuildersByCadShell<Self>>;
}

/// Setup interactions for live manipulations on models using [`CadSlider`]s.
pub trait PmetraInteractions: PmetraModelling {
    /// Configure sliders.
    fn sliders(&self, shells_by_name: &CadShellsByName) -> Result<CadSliders>;

    /// Handler called whenever a [`CadSlider`] is Transformed.
    fn on_slider_transform(
        &mut self,
        name: CadSliderName,
        prev_transform: Transform,
        new_transform: Transform,
    );
    /// Handler called to get [`CadSlider`] tooltip UI text.
    ///
    /// Return `None` if no tooltip should be displayed.
    fn on_slider_tooltip(&self, name: CadSliderName) -> Result<Option<String>>;
}

mod test {

    #[allow(unused_imports)]
    use super::*;

    #[test]
    pub fn test_basic_parametric_modelling_trait() {
        #[allow(dead_code)]
        #[derive(Debug, Clone, Default)]
        pub struct Cube {
            pub width: f64,
        }

        impl PmetraCad for Cube {
            fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
                CadShellsBuilders::default().add_shell_builder(
                    CadShellName("s1".into()),
                    |_p: &Self| Ok(CadShell::default()),
                )
            }
        }

        impl PmetraModelling for Cube {
            fn meshes_builders_by_shell(
                &self,
                shells_by_name: &CadShellsByName,
            ) -> Result<CadMeshesBuildersByCadShell<Self>> {
                CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                    .add_mesh_builder_with_outlines(
                        CadShellName("s1".into()),
                        "m1".into(),
                        CadMeshBuilder::new(self.clone(), CadShellName("s1".into()))?
                            .set_transform(Transform::default())?
                            .set_base_material(StandardMaterial::default())?,
                    )
            }
        }

        impl PmetraInteractions for Cube {
            fn sliders(&self, _shells_by_name: &CadShellsByName) -> Result<CadSliders> {
                let mut sliders = CadSliders::default();
                sliders.insert(CadSliderName("c1".into()), CadSlider::default());

                Ok(sliders)
            }

            fn on_slider_transform(
                &mut self,
                _name: CadSliderName,
                _prev_transform: Transform,
                _new_transform: Transform,
            ) {
                // TODO
            }

            fn on_slider_tooltip(&self, _name: CadSliderName) -> Result<Option<String>> {
                Ok(None)
            }
        }

        pub fn _build_slider_c1(
            _builder: &CadMeshBuilder<Cube>,
            _shell: &CadShell,
        ) -> Result<CadSlider> {
            Ok(CadSlider::default())
        }

        let cube = Cube { width: 1. };
        let cube_shell_builders = cube.shells_builders().unwrap();
        let _build_result = (cube_shell_builders
            .builders
            .get(&CadShellName("s1".into()))
            .unwrap()
            .build_cad_shell)(&cube)
        .unwrap();
    }
}
