use anyhow::{anyhow, Result};
use bevy::{prelude::*, utils::HashMap};
use truck_meshalgo::tessellation::{MeshableShape, MeshedShape};
use truck_modeling::{builder, Shell};

use crate::{
    pmetra_core::{
        dimensions::AsBevyVec3,
        meshing::{BuildBevyMesh, BuildPolygon},
    },
    constants::CUSTOM_TRUCK_TOLERANCE_1,
};

use super::{CadShellName, CadShellsByName};

#[derive(Debug, Clone, Default)]
pub struct CadMeshesBuildersByCadShell<P: Default + Clone> {
    pub params: P,
    pub shells_by_name: CadShellsByName,
    pub meshes_builders: HashMap<CadShellName, CadMeshesBuilder<P>>,
}

impl<P: Default + Clone> CadMeshesBuildersByCadShell<P> {
    pub fn new(params: P, shells_by_name: CadShellsByName) -> Result<Self> {
        let builder = Self {
            params,
            shells_by_name,
            ..default()
        };
        Ok(builder)
    }

    /// Add a new [`CadMeshBuilder`] to the builders.
    pub fn add_mesh_builder(
        &mut self,
        shell_name: CadShellName,
        mesh_name: String,
        mesh_builder: CadMeshBuilder<P>,
    ) -> Result<Self> {
        // Get the shell and set outlines on mesh builder...
        let shell = self
            .shells_by_name
            .get(&shell_name)
            .ok_or_else(|| anyhow!("Could not find shell with name: {:?}", shell_name))?;
        let mut mesh_builder = mesh_builder;
        mesh_builder.set_outlines(shell.shell.build_outlines())?;
        if let Some(meshes_builder) = self.meshes_builders.get_mut(&shell_name) {
            // Add the mesh builder to the existing meshes builder...
            meshes_builder.add_mesh_builder(mesh_name, mesh_builder)?;
        } else {
            // create new and add the mesh builder to the meshes builders...
            let mut meshes_builder = CadMeshesBuilder::new(
                self.params.clone(),
                self.shells_by_name.clone(),
                shell_name.clone(),
            )?;
            meshes_builder.add_mesh_builder(mesh_name, mesh_builder)?;
            // insert the new meshes builder...
            self.meshes_builders.insert(shell_name, meshes_builder);
        }
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CadMeshesBuilder<P: Default + Clone> {
    pub params: P,
    pub shells_by_name: CadShellsByName,
    /// Name of the singular shell used to build the meshes.
    pub shell_name: CadShellName,
    pub mesh_builders: HashMap<CadMeshName, CadMeshBuilder<P>>,
}

impl<P: Default + Clone> CadMeshesBuilder<P> {
    pub fn new(
        params: P,
        shells_by_name: CadShellsByName,
        shell_name: CadShellName,
    ) -> Result<Self> {
        let builder = Self {
            params,
            shells_by_name,
            shell_name,
            ..default()
        };
        Ok(builder)
    }

    pub fn build_bevy_mesh(&self) -> Result<Mesh> {
        let cad_shell = self
            .shells_by_name
            .get(&self.shell_name)
            .ok_or_else(|| anyhow!("Could not find shell with name: {:?}", self.shell_name))?;
        let mesh = cad_shell.build_polygon()?.build_mesh();
        Ok(mesh)
    }

    pub fn add_mesh_builder(
        &mut self,
        mesh_name: String,
        mesh_builder: CadMeshBuilder<P>,
    ) -> Result<Self> {
        self.mesh_builders.insert(mesh_name.into(), mesh_builder);
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, Deref, DerefMut, Hash, PartialEq, Eq, Component)]
pub struct CadMeshName(pub String);

impl From<String> for CadMeshName {
    fn from(value: String) -> Self {
        CadMeshName(value)
    }
}

/// Builder for building a [`CadMesh`].
#[derive(Debug, Clone, Default, Component)]
pub struct CadMeshBuilder<P: Default + Clone> {
    pub params: P,
    pub shell_name: CadShellName,
    pub mesh_hdl: Option<Handle<Mesh>>,
    pub base_material: StandardMaterial,
    pub outlines: CadMeshOutlines,
    pub transform: Transform,
    // pub cursors: HashMap<CadCursorName, CadCursor>,
}

impl<P: Default + Clone> CadMeshBuilder<P> {
    pub fn new(params: P, shell_name: CadShellName) -> Result<Self> {
        let builder = Self {
            params,
            shell_name,
            ..default()
        };
        Ok(builder)
    }

    pub fn set_mesh_hdl(&mut self, mesh_hdl: Handle<Mesh>) -> Result<Self> {
        self.mesh_hdl = Some(mesh_hdl);
        Ok(self.clone())
    }

    pub fn set_base_material(&mut self, material: StandardMaterial) -> Result<Self> {
        self.base_material = material;
        Ok(self.clone())
    }

    pub fn set_transform(&mut self, transform: Transform) -> Result<Self> {
        self.transform = transform;
        Ok(self.clone())
    }

    pub fn set_outlines(&mut self, outlines: CadMeshOutlines) -> Result<Self> {
        self.outlines = outlines;
        Ok(self.clone())
    }

    pub fn build(&self) -> Result<CadMesh> {
        Ok(CadMesh {
            mesh_hdl: self
                .mesh_hdl
                .clone()
                .ok_or_else(|| anyhow!("Mesh Handle is None!"))?,
            base_material: self.base_material.clone(),
            transform: self.transform,
            outlines: self.outlines.clone(),
            // cursors: self.cursors.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CadMesh {
    pub mesh_hdl: Handle<Mesh>,
    pub base_material: StandardMaterial,
    pub transform: Transform,
    pub outlines: CadMeshOutlines,
}

/// Outlines for [`InteractiveCadMesh`]
#[derive(Debug, Clone, Default, Deref, DerefMut, Reflect)]
pub struct CadMeshOutlines(pub Vec<Vec<Vec3>>);

/// Trait which allows building [`CadMeshOutlines`] for given truck primitive elem.
pub trait BuildCadMeshOutlines {
    /// Build [`CadMeshOutlines`] for given struct.
    fn build_outlines(&self) -> CadMeshOutlines;
}

impl BuildCadMeshOutlines for Shell {
    fn build_outlines(&self) -> CadMeshOutlines {
        let outlines = self
            .face_iter()
            .flat_map(|face| {
                let face_outlines = face
                    .boundaries()
                    .iter()
                    .filter_map(|wire| {
                        let Some(face) = builder::try_attach_plane(&[wire.clone()]).ok() else {
                            return None;
                        };
                        let shell = Shell::from(vec![face]);
                        let polygon_mesh = shell
                            .compress()
                            .triangulation(CUSTOM_TRUCK_TOLERANCE_1)
                            .to_polygon();
                        let pos_vectors = polygon_mesh
                            .positions()
                            .iter()
                            .map(|p| p.as_bevy_vec3())
                            .collect::<Vec<_>>();
                        Some(pos_vectors)
                    })
                    .collect::<Vec<_>>();
                face_outlines
            })
            .collect::<Vec<_>>();

        CadMeshOutlines(outlines)
    }
}
