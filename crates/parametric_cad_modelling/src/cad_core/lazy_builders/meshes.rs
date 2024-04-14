use anyhow::{anyhow, Result};
use bevy::{prelude::*, utils::HashMap};

use crate::
    cad_core::{
        builders::{
            BuildCadMeshOutlines, CadMeshName, CadMeshOutlines,
        },
        meshing::{BuildBevyMesh, BuildPolygon},
    }
;

use super::{CadShellName, CadShellsByName};

#[derive(Debug, Clone, Default)]
pub struct CadMeshesLazyBuildersByCadShell<P: Default + Clone> {
    pub params: P,
    pub shells_by_name: CadShellsByName,
    pub meshes_builders: HashMap<CadShellName, CadMeshesLazyBuilder<P>>,
}

impl<P: Default + Clone> CadMeshesLazyBuildersByCadShell<P> {
    pub fn new(params: P, shells_by_name: CadShellsByName) -> Result<Self> {
        let builder = Self {
            params,
            shells_by_name,
            ..default()
        };
        Ok(builder)
    }

    /// Add a new [`CadMeshLazyBuilder`] to the builders.
    pub fn add_mesh_builder(
        &mut self,
        shell_name: CadShellName,
        mesh_name: String,
        mesh_builder: CadMeshLazyBuilder<P>,
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
            let mut meshes_builder = CadMeshesLazyBuilder::new(
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
pub struct CadMeshesLazyBuilder<P: Default + Clone> {
    pub params: P,
    pub shells_by_name: CadShellsByName,
    /// Name of the singular shell used to build the meshes.
    pub shell_name: CadShellName,
    pub mesh_builders: HashMap<CadMeshName, CadMeshLazyBuilder<P>>,
}

impl<P: Default + Clone> CadMeshesLazyBuilder<P> {
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
        mesh_builder: CadMeshLazyBuilder<P>,
    ) -> Result<Self> {
        self.mesh_builders.insert(mesh_name.into(), mesh_builder);
        Ok(self.clone())
    }
}

/// Builder for building a [`CadMesh`].
#[derive(Debug, Clone, Default, Component)]
pub struct CadMeshLazyBuilder<P: Default + Clone> {
    pub params: P,
    pub shell_name: CadShellName,
    pub mesh_hdl: Option<Handle<Mesh>>,
    pub base_material: StandardMaterial,
    pub outlines: CadMeshOutlines,
    pub transform: Transform,
    // pub cursors: HashMap<CadCursorName, CadCursor>,
}

impl<P: Default + Clone> CadMeshLazyBuilder<P> {
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

    // pub fn add_cursor(
    //     &mut self,
    //     cursor_name: String,
    //     build_fn: fn(&Self, &CadShell) -> Result<CadCursor>,
    // ) -> Result<Self> {
    //     let cursor = build_fn(self, &self.shell).with_context(|| "Failed to build cursor!")?;
    //     self.cursors.insert(cursor_name.into(), cursor);
    //     Ok(self.clone())
    // }

    pub fn build(&self) -> Result<CadLazyMesh> {
        Ok(CadLazyMesh {
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
pub struct CadLazyMesh {
    pub mesh_hdl: Handle<Mesh>,
    pub base_material: StandardMaterial,
    pub transform: Transform,
    pub outlines: CadMeshOutlines,
    // pub cursors: HashMap<CadCursorName, CadCursor>,
}
