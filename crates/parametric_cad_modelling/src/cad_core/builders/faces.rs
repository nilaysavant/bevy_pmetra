use anyhow::{anyhow, Context, Ok, Result};
use bevy::{prelude::*, utils::HashMap};
use truck_meshalgo::tessellation::{MeshableShape, MeshedShape};
use truck_modeling::{builder, Face, Shell};

use crate::{constants::CUSTOM_TRUCK_TOLERANCE_1, cad_core::dimensions::AsBevyVec3};

use super::{CadCursor, CadShell};

/// Builder for building a [`InteractiveCadFace`].
#[derive(Debug, Clone, Default)]
pub struct InteractiveCadFaceBuilder<P: Default + Clone> {
    pub params: P,
    pub shell: CadShell,
    pub cad_face: Option<InteractiveCadFace>,
}

impl<P: Default + Clone> InteractiveCadFaceBuilder<P> {
    pub fn new(params: P, shell: CadShell) -> Result<Self> {
        let builder = Self {
            params,
            shell,
            ..default()
        };
        Ok(builder)
    }

    pub fn set_bevy_mesh(&mut self, mesh: Mesh) -> Result<Self> {
        self.cad_face = Some(InteractiveCadFace {
            mesh,
            normal: Default::default(),
            transform: Default::default(),
            outlines: Default::default(),
            cursors: Default::default(),
        });
        Ok(self.clone())
    }

    pub fn set_transform(&mut self, transform: Transform) -> Result<Self> {
        let cad_face = self
            .cad_face
            .as_mut()
            .ok_or_else(|| anyhow!("InteractiveCadFace is None!"))?;
        cad_face.transform = transform;
        Ok(self.clone())
    }

    pub fn set_normal(&mut self, normal: Vec3) -> Result<Self> {
        let cad_face = self
            .cad_face
            .as_mut()
            .ok_or_else(|| anyhow!("InteractiveCadFace is None!"))?;
        cad_face.normal = normal;
        Ok(self.clone())
    }

    pub fn set_outlines(&mut self, outlines: InteractiveCadFaceOutlines) -> Result<Self> {
        let cad_face = self
            .cad_face
            .as_mut()
            .ok_or_else(|| anyhow!("InteractiveCadFace is None!"))?;
        cad_face.outlines = outlines;
        Ok(self.clone())
    }

    /// Add [`InteractiveCadFace`] from [`CadSolid`].
    pub fn add_cursor(
        &mut self,
        cursor_name: String,
        build_fn: fn(&Self, &CadShell) -> Result<CadCursor>,
    ) -> Result<Self> {
        let cursor = build_fn(self, &self.shell).with_context(|| "Failed to build cursor!")?;

        let cad_face = self
            .cad_face
            .as_mut()
            .ok_or_else(|| anyhow!("InteractiveCadFace is None!"))?;
        cad_face.cursors.insert(cursor_name, cursor);

        Ok(self.clone())
    }

    pub fn build(&self) -> Result<InteractiveCadFace> {
        self.cad_face
            .clone()
            .ok_or_else(|| anyhow!("InteractiveCadFace is None!"))
    }
}

#[derive(Debug, Clone)]
pub struct InteractiveCadFace {
    pub mesh: Mesh,
    pub normal: Vec3,
    pub transform: Transform,
    pub outlines: InteractiveCadFaceOutlines,
    pub cursors: HashMap<String, CadCursor>,
}

/// Outlines for [`InteractiveCadFace`]
#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct InteractiveCadFaceOutlines(pub Vec<Vec<Vec3>>);

/// Trait which allows building [`InteractiveCadFaceOutlines`] for given truck primitive elem.
pub trait BuildCadFaceOutlines {
    /// Build [`InteractiveCadFaceOutlines`] for given struct.
    fn build_outlines(&self) -> InteractiveCadFaceOutlines;
}

impl BuildCadFaceOutlines for Face {
    fn build_outlines(&self) -> InteractiveCadFaceOutlines {
        let outlines_inner = self
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

        InteractiveCadFaceOutlines(outlines_inner)
    }
}
