use anyhow::{Context, Ok, Result};
use bevy::{prelude::*, utils::HashMap};
use truck_meshalgo::{
    filters::OptimizingFilter, rexport_polymesh::PolygonMesh, tessellation::MeshedShape,
};
use truck_modeling::{Shell, Surface};

use crate::{
    cad_core::{
        meshing::{BuildCadMeshedShell, BuildPolygon},
        tessellation::{CadMeshedShell, CustomMeshableShape},
    },
    constants::CUSTOM_TRUCK_TOLERANCE_1,
};

use super::{CadElement, CadElementTag, CadTaggedElements};

#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct CadShells(pub HashMap<String, CadShell>);

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
