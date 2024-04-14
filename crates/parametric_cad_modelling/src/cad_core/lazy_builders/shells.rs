use bevy::{prelude::*, utils::HashMap};

use anyhow::{anyhow, Result};
use truck_meshalgo::{
    filters::OptimizingFilter, rexport_polymesh::PolygonMesh, tessellation::MeshedShape,
};
use truck_modeling::{Shell, Surface};

use crate::{
    cad_core::{
        builders::{CadElement, CadElementTag, CadTaggedElements},
        meshing::{BuildCadMeshedShell, BuildPolygon},
        tessellation::{CadMeshedShell, CustomMeshableShape},
    },
    constants::CUSTOM_TRUCK_TOLERANCE_1,
};

/// Holds multiple [`CadShellLazyBuilder`]s.
#[derive(Clone, Default)]
pub struct CadShellsLazyBuilders<P: Default + Clone> {
    pub params: P,
    pub builders: HashMap<CadShellName, CadShellLazyBuilder<P>>,
}

impl<P: Default + Clone> CadShellsLazyBuilders<P> {
    pub fn new(params: P) -> Result<Self> {
        let builder = Self {
            params,
            ..default()
        };
        Ok(builder)
    }

    /// Add new [`CadShellLazyBuilder`] to builders.
    pub fn add_shell_builder(
        &mut self,
        shell_name: CadShellName,
        build_fn: fn(&P) -> Result<CadShell>,
    ) -> Result<Self> {
        let shell_builder = CadShellLazyBuilder {
            params: self.params.clone(),
            build_cad_shell: build_fn,
        };
        self.builders.insert(shell_name, shell_builder);
        Ok(self.clone())
    }

    /// Build [`CadShell`] using the stored [`CadShellLazyBuilder`] with `shell_name`.
    pub fn build_shell(&self, shell_name: CadShellName) -> Result<CadShell> {
        (self
            .builders
            .get(&shell_name)
            .ok_or_else(|| anyhow!("Could not find shell with name: {:?}", shell_name))?
            .build_cad_shell)(&self.params)
    }
}

/// Builder for building [`CadShell`]s.
#[derive(Clone, Component)]
pub struct CadShellLazyBuilder<P: Default + Clone> {
    pub params: P,
    pub build_cad_shell: fn(&P) -> Result<CadShell>,
}

impl<P: Default + Clone> CadShellLazyBuilder<P> {
    pub fn new(params: P, build_fn: fn(&P) -> Result<CadShell>) -> Self {
        Self {
            params,
            build_cad_shell: build_fn,
        }
    }

    pub fn build_cad_shell(&self) -> Result<CadShell> {
        (self.build_cad_shell)(&self.params)
    }
}

/// Component to store all generated [`CadShell`]s by [`CadShellName`].
#[derive(Debug, Clone, Component, Deref, DerefMut, Default)]
pub struct CadShellsByName(pub HashMap<CadShellName, CadShell>);

/// Name of the [`CadShell`].
///
/// Used to identify the shell.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deref, DerefMut, Reflect, Component)]
pub struct CadShellName(pub String);

/// CAD generated [`Shell`].
///
/// Holds [`Shell`] with [`CadTaggedElements`].
#[derive(Debug, Clone, Default, Component)]
pub struct CadShell {
    pub shell: Shell,
    pub tagged_elements: CadTaggedElements,
}

impl CadShell {
    pub fn get_element_by_tag(&self, tag: CadElementTag) -> Option<&CadElement> {
        let element = self.tagged_elements.get(&tag);

        element
    }
}

impl BuildCadMeshedShell for CadShell {
    fn build_cad_meshed_shell(&self) -> Result<CadMeshedShell<Surface>> {
        self.build_cad_meshed_shell_with_tol(CUSTOM_TRUCK_TOLERANCE_1)
    }

    fn build_cad_meshed_shell_with_tol(&self, tol: f64) -> Result<CadMeshedShell<Surface>> {
        let cad_meshed_shell = self.shell.triangulation(tol);

        Ok(cad_meshed_shell)
    }
}

impl BuildPolygon for CadShell {
    fn build_polygon(&self) -> Result<PolygonMesh> {
        self.build_polygon_with_tol(CUSTOM_TRUCK_TOLERANCE_1)
    }

    fn build_polygon_with_tol(&self, tol: f64) -> Result<PolygonMesh> {
        let mut polygon_mesh = self.shell.triangulation(tol).meshed_shell.to_polygon();
        // Also cleanup any degenerate stuff...
        polygon_mesh.remove_degenerate_faces().remove_unused_attrs();

        Ok(polygon_mesh)
    }
}
