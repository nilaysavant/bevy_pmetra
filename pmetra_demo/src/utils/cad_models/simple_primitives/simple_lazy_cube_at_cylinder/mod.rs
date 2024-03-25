use std::{f64::consts::PI, str::FromStr};

use anyhow::{anyhow, Context, Error, Ok, Result};
use bevy::{math::DVec3, prelude::*, utils::HashMap};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::{
    cad_core::lazy_builders::{
        CadMeshLazyBuilder, CadMeshesLazyBuilder, CadMeshesLazyBuildersByCadShell, CadShellName,
        CadShellsByName, CadShellsLazyBuilders, ParametricLazyCad, ParametricLazyModelling,
    },
    prelude::*,
};
use strum::{Display, EnumString};

use self::{
    cube::{build_cube_shell, CubeCursorIds},
    cylinder::{
        build_cylinder_shell, build_radius_cursor, cylinder_mesh_builder, CylinderCursorIds,
    },
};

pub mod cube;
pub mod cylinder;

/// Basic Parametric Station Segment.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct SimpleLazyCubeAtCylinder {
    #[inspector(min = 0.1)]
    pub cylinder_radius: f64,
    #[inspector(min = 0.1)]
    pub cylinder_height: f64,
    #[inspector(min = 0., max = std::f64::consts::TAU)]
    pub cube_attach_angle: f64,
    #[inspector(min = 0.1)]
    pub cube_side_length: f64,
}

impl Default for SimpleLazyCubeAtCylinder {
    fn default() -> Self {
        Self {
            cylinder_radius: 1.2,
            cylinder_height: 0.5,
            cube_side_length: 0.2,
            cube_attach_angle: PI,
        }
    }
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadSolidIds {
    Cylinder,
    Cube,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadMeshIds {
    Cylinder,
    Cube,
}

impl ParametricLazyModelling for SimpleLazyCubeAtCylinder {
    fn shells_builders(
        &self,
    ) -> Result<bevy_pmetra::cad_core::lazy_builders::CadShellsLazyBuilders<Self>> {
        let builders = CadShellsLazyBuilders::new(self.clone())? // builder
            .add_shell_builder(
                CadShellName(CadSolidIds::Cylinder.to_string()),
                build_cylinder_shell,
            )?
            .add_shell_builder(
                CadShellName(CadSolidIds::Cube.to_string()),
                build_cube_shell,
            )?;
        Ok(builders)
    }
}

impl ParametricLazyCad for SimpleLazyCubeAtCylinder {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: CadShellsByName,
    ) -> Result<CadMeshesLazyBuildersByCadShell<Self>> {
        let cad_meshes_lazy_builders_by_cad_shell =
            CadMeshesLazyBuildersByCadShell::new(self.clone(), shells_by_name)?
                .add_mesh_builder(
                    CadShellName(CadSolidIds::Cylinder.to_string()),
                    CadMeshIds::Cylinder.to_string(),
                    cylinder_mesh_builder(self, CadShellName(CadSolidIds::Cylinder.to_string()))?,
                )?
                .add_mesh_builder(
                    CadShellName(CadSolidIds::Cylinder.to_string()),
                    CadMeshIds::Cylinder.to_string(),
                    cylinder_mesh_builder(self, CadShellName(CadSolidIds::Cylinder.to_string()))?,
                )?;

        Ok(cad_meshes_lazy_builders_by_cad_shell)
    }

    fn on_cursor_transform(
        &mut self,
        mesh_name: CadMeshName,
        cursor_name: CadCursorName,
        prev_transform: Transform,
        new_transform: Transform,
    ) {
        match CadMeshIds::from_str(&mesh_name).unwrap() {
            CadMeshIds::Cylinder => match CylinderCursorIds::from_str(&cursor_name).unwrap() {
                CylinderCursorIds::RadiusCursor => {
                    let delta = new_transform.translation - prev_transform.translation;
                    if delta.length() > 0. {
                        let sensitivity = 1.;
                        let new_value = self.cylinder_radius + delta.x as f64 * sensitivity;
                        self.cylinder_radius = new_value.clamp(0.01, std::f64::MAX);
                    }
                }
            },
            CadMeshIds::Cube => match CubeCursorIds::from_str(&cursor_name).unwrap() {
                CubeCursorIds::SideLength => {
                    let delta = new_transform.translation - prev_transform.translation;
                    if delta.length() > 0. {
                        let sensitivity = 1.;
                        let new_value = self.cube_side_length + delta.z as f64 * sensitivity;
                        self.cube_side_length = new_value.clamp(0.01, std::f64::MAX);
                    }
                }
            },
        }
    }

    fn on_cursor_tooltip(
        &self,
        mesh_name: CadMeshName,
        cursor_name: CadCursorName,
    ) -> Result<String> {
        let tooltip = match CadMeshIds::from_str(&mesh_name).unwrap() {
            CadMeshIds::Cylinder => match CylinderCursorIds::from_str(&cursor_name).unwrap() {
                CylinderCursorIds::RadiusCursor => {
                    format!("cylinder_radius : {:.3}", self.cylinder_radius)
                }
            },
            CadMeshIds::Cube => match CubeCursorIds::from_str(&cursor_name).unwrap() {
                CubeCursorIds::SideLength => {
                    format!("cube_side_length : {:.3}", self.cube_side_length)
                }
            },
        };
        Ok(tooltip)
    }
}
