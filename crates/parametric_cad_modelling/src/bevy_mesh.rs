use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use truck_meshalgo::rexport_polymesh::PolygonMesh;

/// Indices type used for Bevy [`Mesh`].
pub type BevyIndices = Vec<u32>;

/// Vertices type used for Bevy [`Mesh`].
pub type BevyVertices = Vec<[f32; 3]>;

/// Normals type used for Bevy [`Mesh`].
pub type BevyNormals = Vec<[f32; 3]>;

/// UV coords type used for Bevy [`Mesh`].
pub type BevyUvCoords = Vec<[f32; 2]>;

/// Builder for generating Bevy [`Mesh`] using the provided attributes.
pub struct BevyMeshBuilder {
    pub vertices: BevyVertices,
    pub indices: BevyIndices,
    pub normals: BevyNormals,
    pub uvs: BevyUvCoords,
}

impl BevyMeshBuilder {
    /// Builds a Bevy [`Mesh`] from builder.
    pub fn build(self) -> Mesh {
        Mesh::from(self)
    }
}

impl From<BevyMeshBuilder> for Mesh {
    /// Builds a Bevy [`Mesh`] from [`BevyMeshBuilder`].
    fn from(value: BevyMeshBuilder) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(value.indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, value.vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, value.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, value.uvs);

        mesh
    }
}

impl From<&PolygonMesh> for BevyMeshBuilder {
    /// Create a [`BevyMeshBuilder`] from [`PolygonMesh`].
    fn from(value: &PolygonMesh) -> Self {
        let vertices = value
            .positions()
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32])
            .collect::<Vec<_>>();
        let indices = value
            .tri_faces()
            .iter()
            .flat_map(|face| [face[0].pos as u32, face[1].pos as u32, face[2].pos as u32])
            .collect::<Vec<_>>();
        let normals = value
            .normals()
            .iter()
            .map(|n| [n.x as f32, n.y as f32, n.z as f32])
            .collect::<Vec<_>>();
        let uvs = value
            .uv_coords()
            .iter()
            .map(|u| [u.x as f32, u.y as f32])
            .collect::<Vec<_>>();

        BevyMeshBuilder {
            vertices,
            indices,
            normals,
            uvs,
        }
    }
}
