use anyhow::{anyhow, Context, Ok, Result};
use bevy::{prelude::*, utils::HashMap};
use truck_meshalgo::tessellation::{MeshableShape, MeshedShape};
use truck_modeling::{builder, Shell, Solid};

use crate::{cad_core::dimensions::AsBevyVec3, constants::CUSTOM_TRUCK_TOLERANCE_1};

use super::{CadCursor, CadMaterialTextureSet, CadMaterialTextures, CadShell, CadShells};

/// Builder for building [`CadSolid`]s.
#[derive(Debug, Clone, Default)]
pub struct CadMeshesBuilder<P: Default + Clone> {
    pub params: P,
    pub shells: CadShells,
    pub textures: CadMaterialTextures<Option<Image>>,
    pub meshes: CadMeshes,
}

impl<P: Default + Clone> CadMeshesBuilder<P> {
    pub fn new(
        params: P,
        shells: CadShells,
        textures: CadMaterialTextures<Option<Image>>,
    ) -> Result<Self> {
        let builder = Self {
            params,
            shells,
            textures,
            ..default()
        };
        Ok(builder)
    }

    /// Add mesh from previously created [`CadSolid`] given `solid_name`.
    ///
    /// Create [`CadMesh`] with `mesh_name` using the provided `build_fn` closure.
    pub fn add_mesh(
        &mut self,
        shell_name: String,
        mesh_name: String,
        build_fn: fn(&Self, &CadShell, &CadMaterialTextures<Option<Image>>) -> Result<CadMesh>,
    ) -> Result<Self> {
        let shell = self
            .shells
            .get(&shell_name)
            .ok_or_else(|| anyhow!("Could not find solid!"))?;
        let mesh =
            build_fn(self, shell, &self.textures).with_context(|| "Failed to build mesh!")?;
        self.meshes.insert(mesh_name.into(), mesh);
        Ok(self.clone())
    }

    pub fn build(&self) -> Result<CadMeshes> {
        Ok(self.meshes.clone())
    }
}

#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct CadMeshes(pub HashMap<CadMeshName, CadMesh>);

#[derive(Debug, Clone, Deref, DerefMut, Hash, PartialEq, Eq, Component)]
pub struct CadMeshName(pub String);

impl From<String> for CadMeshName {
    fn from(value: String) -> Self {
        CadMeshName(value)
    }
}

/// Builder for building a [`CadMesh`].
#[derive(Debug, Clone, Default)]
pub struct CadMeshBuilder<P: Default + Clone> {
    pub params: P,
    pub shell: CadShell,
    pub cad_mesh: Option<CadMesh>,
}

impl<P: Default + Clone> CadMeshBuilder<P> {
    pub fn new(params: P, shell: CadShell) -> Result<Self> {
        let builder = Self {
            params,
            shell,
            ..default()
        };
        Ok(builder)
    }

    pub fn set_bevy_mesh(&mut self, mesh: Mesh) -> Result<Self> {
        self.cad_mesh = Some(CadMesh {
            mesh,
            base_material: Default::default(),
            material_texture_set: Default::default(),
            outlines: Default::default(),
            transform: Default::default(),
            cursors: Default::default(),
        });
        Ok(self.clone())
    }

    pub fn set_base_material(&mut self, material: StandardMaterial) -> Result<Self> {
        let cad_mesh = self
            .cad_mesh
            .as_mut()
            .ok_or_else(|| anyhow!("CadMesh is None!"))?;
        cad_mesh.base_material = material;
        Ok(self.clone())
    }

    pub fn set_material_texture_set(
        &mut self,
        material_texture_set: CadMaterialTextureSet<Option<Image>>,
    ) -> Result<Self> {
        let cad_mesh = self
            .cad_mesh
            .as_mut()
            .ok_or_else(|| anyhow!("CadMesh is None!"))?;
        cad_mesh.material_texture_set = material_texture_set;
        Ok(self.clone())
    }

    pub fn set_transform(&mut self, transform: Transform) -> Result<Self> {
        let cad_mesh = self
            .cad_mesh
            .as_mut()
            .ok_or_else(|| anyhow!("CadMesh is None!"))?;
        cad_mesh.transform = transform;
        Ok(self.clone())
    }

    pub fn set_outlines(&mut self, outlines: CadMeshOutlines) -> Result<Self> {
        let cad_mesh = self
            .cad_mesh
            .as_mut()
            .ok_or_else(|| anyhow!("CadMesh is None!"))?;
        cad_mesh.outlines = outlines;
        Ok(self.clone())
    }

    /// Add [`CadFaceCursor`] from [`CadSolid`].
    pub fn add_cursor(
        &mut self,
        cursor_name: String,
        build_fn: fn(&Self, &CadShell) -> Result<CadCursor>,
    ) -> Result<Self> {
        let cursor = build_fn(self, &self.shell).with_context(|| "Failed to build cursor!")?;
        let cad_mesh = self
            .cad_mesh
            .as_mut()
            .ok_or_else(|| anyhow!("CadMesh is None!"))?;
        cad_mesh.cursors.insert(cursor_name.into(), cursor);
        Ok(self.clone())
    }

    pub fn build(&self) -> Result<CadMesh> {
        self.cad_mesh
            .clone()
            .ok_or_else(|| anyhow!("CadMesh is None!"))
    }
}

#[derive(Debug, Clone)]
pub struct CadMesh {
    pub mesh: Mesh,
    pub base_material: StandardMaterial,
    pub material_texture_set: CadMaterialTextureSet<Option<Image>>,
    pub transform: Transform,
    pub outlines: CadMeshOutlines,
    pub cursors: HashMap<CadCursorName, CadCursor>,
}

#[derive(Debug, Clone, Deref, DerefMut, Hash, PartialEq, Eq, Component)]
pub struct CadCursorName(pub String);

impl From<String> for CadCursorName {
    fn from(value: String) -> Self {
        CadCursorName(value)
    }
}

/// Outlines for [`InteractiveCadMesh`]
#[derive(Debug, Clone, Default, Deref, DerefMut)]
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
