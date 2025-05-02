use anyhow::{anyhow, Result};
use truck_meshalgo::{rexport_polymesh::PolygonMesh, tessellation::MeshedShape};
use truck_modeling::{Face, Shell, Wire};

use crate::{
    constants::CUSTOM_TRUCK_TOLERANCE_1,
    pmetra_core::{meshing::BuildPolygon, tessellation::CustomMeshableShape},
};

/// Extensions to [`Face`] primitive.
pub trait FaceCadExtension: Sized {
    /// Gets the last boundary [`Wire`] from [`Face`].
    fn get_last_boundary_wire(&self) -> Result<Wire>;
}

impl FaceCadExtension for Face {
    fn get_last_boundary_wire(&self) -> Result<Wire> {
        let boundaries = &self.boundaries();
        let wire = boundaries
            .last()
            .ok_or_else(|| anyhow!("Could not find last wire for face!"))?;

        Ok(wire.clone())
    }
}

impl BuildPolygon for Face {
    fn build_polygon(&self) -> Result<PolygonMesh> {
        self.build_polygon_with_tol(CUSTOM_TRUCK_TOLERANCE_1)
    }

    fn build_polygon_with_tol(&self, tol: f64) -> Result<PolygonMesh> {
        let shell = Shell::from(vec![self.clone()]);
        let polygon = shell.triangulation(tol).meshed_shell.to_polygon();

        Ok(polygon)
    }
}
