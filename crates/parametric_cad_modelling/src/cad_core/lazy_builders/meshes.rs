use std::hash::Hash;

use anyhow::{anyhow, Context, Result};
use bevy::{prelude::*, utils::HashMap};
use truck_meshalgo::tessellation::{MeshableShape, MeshedShape};
use truck_modeling::{builder, Shell, Solid};

use crate::{
    cad_core::{
        builders::{CadCursor, CadCursorName, CadMeshName, CadMeshOutlines, CadMeshes, CadShell},
        dimensions::AsBevyVec3,
        meshing::{BuildBevyMesh, BuildPolygon},
    },
    constants::CUSTOM_TRUCK_TOLERANCE_1,
};

use super::CadShellName;

// CadMeshesLazyBuildersByCadShell
// - HashMap<ShellId, CadMeshesLazyBuilder>

/// Builder for building [`CadSolid`]s.
#[derive(Debug, Clone, Default)]
pub struct CadMeshesLazyBuildersByCadShell<P: Default + Clone>(
    pub HashMap<CadShellName, CadMeshesLazyBuilder<P>>,
);

impl<P: Default + Clone> CadMeshesLazyBuildersByCadShell<P> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_meshes_builder(
        &mut self,
        shell_name: CadShellName,
        mesh_builder: CadMeshesLazyBuilder<P>,
    ) -> Result<Self> {
        self.0.insert(shell_name, mesh_builder);
        Ok(self.clone())
    }

    pub fn build(&self) -> Result<HashMap<CadShellName, CadMeshesLazyBuilder<P>>> {
        Ok(self.0.clone())
    }
}

/// Builder for building [`CadSolid`]s.
#[derive(Debug, Clone, Default)]
pub struct CadMeshesLazyBuilder<P: Default + Clone> {
    pub params: P,
    pub shell: CadShell,
    pub mesh_builders: HashMap<CadMeshName, CadMeshLazyBuilder<P>>,
}

impl<P: Default + Clone> CadMeshesLazyBuilder<P> {
    pub fn new(params: P, shell: CadShell) -> Result<Self> {
        let builder = Self {
            params,
            shell,
            ..default()
        };
        Ok(builder)
    }

    pub fn build_bevy_mesh(&self) -> Result<Mesh> {
        let mesh = self.shell.build_polygon()?.build_mesh();
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
    pub shell: CadShell,
    pub mesh_hdl: Option<Handle<Mesh>>,
    pub base_material: StandardMaterial,
    pub outlines: CadMeshOutlines,
    pub transform: Transform,
    pub cursors: HashMap<CadCursorName, CadCursor>,
}

impl<P: Default + Clone> CadMeshLazyBuilder<P> {
    pub fn new(params: P, shell: CadShell) -> Result<Self> {
        let builder = Self {
            params,
            shell,
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

    pub fn add_cursor(
        &mut self,
        cursor_name: String,
        build_fn: fn(&Self, &CadShell) -> Result<CadCursor>,
    ) -> Result<Self> {
        let cursor = build_fn(self, &self.shell).with_context(|| "Failed to build cursor!")?;
        self.cursors.insert(cursor_name.into(), cursor);
        Ok(self.clone())
    }

    pub fn build(&self) -> Result<CadLazyMesh> {
        Ok(CadLazyMesh {
            mesh_hdl: self
                .mesh_hdl
                .clone()
                .ok_or_else(|| anyhow!("Mesh Handle is None!"))?,
            base_material: self.base_material.clone(),
            transform: self.transform,
            outlines: self.outlines.clone(),
            cursors: self.cursors.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CadLazyMesh {
    pub mesh_hdl: Handle<Mesh>,
    pub base_material: StandardMaterial,
    pub transform: Transform,
    pub outlines: CadMeshOutlines,
    pub cursors: HashMap<CadCursorName, CadCursor>,
}
