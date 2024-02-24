use anyhow::Result;
use bevy::prelude::*;
use truck_meshalgo::rexport_polymesh::PolygonMesh;
use truck_modeling::Surface;

use super::tessellation::CadMeshedShell;

/// Trait to build Bevy [`Mesh`].
pub trait BuildBevyMesh {
    /// Build bevy [`Mesh`] for struct.
    fn build_mesh(&self) -> Mesh;
}

/// Trait to allow building [`PolygonMesh`] for any truck primitive element.
pub trait BuildPolygon {
    /// Build [`PolygonMesh`] for the given primitive struct.
    fn build_polygon(&self) -> Result<PolygonMesh>;

    /// Build [`PolygonMesh`] given the tolerance for triangulation.
    fn build_polygon_with_tol(&self, tol: f64) -> Result<PolygonMesh>;
}

/// Trait to allow building [`CadMeshedShell`] for any truck primitive element.
pub trait BuildCadMeshedShell {
    /// Build [`CadPolygonMesh`] for the given primitive struct.
    fn build_cad_meshed_shell(&self) -> Result<CadMeshedShell<Surface>>;

    /// Build [`CadPolygonMesh`] given the tolerance for triangulation.
    fn build_cad_meshed_shell_with_tol(&self, tol: f64) -> Result<CadMeshedShell<Surface>>;
}
