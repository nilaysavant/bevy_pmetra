use bevy::prelude::*;
use truck_meshalgo::rexport_polymesh::PolygonMesh;

use crate::{bevy_mesh::BevyMeshBuilder, cad_core::meshing::BuildBevyMesh};

impl BuildBevyMesh for PolygonMesh {
    fn build_mesh(&self) -> Mesh {
        BevyMeshBuilder::from(self).build()
    }
}
