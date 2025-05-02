use std::{f64::consts::PI, str::FromStr};

use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::{prelude::*, re_exports::anyhow::Result};
use strum::{Display, EnumString};

use self::{
    cube::{build_cube_shell, build_side_length_slider, cube_mesh_builder},
    cylinder::{build_cylinder_shell, build_radius_slider, cylinder_mesh_builder},
};

pub mod cube;
pub mod cylinder;

/// Basic Parametric Station Segment.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct SimpleCubeAtCylinder {
    #[inspector(min = 0.1)]
    pub cylinder_radius: f64,
    #[inspector(min = 0.1)]
    pub cylinder_height: f64,
    #[inspector(min = 0., max = std::f64::consts::TAU)]
    pub cube_attach_angle: f64,
    #[inspector(min = 0.1)]
    pub cube_side_length: f64,
}

impl Default for SimpleCubeAtCylinder {
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
pub enum CadShellIds {
    Cylinder,
    Cube,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadMeshIds {
    Cylinder,
    Cube,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadSliderIds {
    CylinderRadius,
    CubeSideLength,
}

impl PmetraCad for SimpleCubeAtCylinder {
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
        let builders = CadShellsBuilders::new(self.clone())? // builder
            .add_shell_builder(
                CadShellName(CadShellIds::Cylinder.to_string()),
                build_cylinder_shell,
            )?
            .add_shell_builder(
                CadShellName(CadShellIds::Cube.to_string()),
                build_cube_shell,
            )?;
        Ok(builders)
    }
}

impl PmetraModelling for SimpleCubeAtCylinder {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesBuildersByCadShell<Self>> {
        let mut cad_meshes_lazy_builders_by_cad_shell =
            CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                .add_mesh_builder_with_outlines(
                    CadShellName(CadShellIds::Cylinder.to_string()),
                    CadMeshIds::Cylinder.to_string(),
                    cylinder_mesh_builder(self, CadShellName(CadShellIds::Cylinder.to_string()))?,
                )?;

        let cubes_count = (self.cylinder_radius * 4.).floor() as i32;
        for i in 0..cubes_count {
            cad_meshes_lazy_builders_by_cad_shell.add_mesh_builder_with_outlines(
                CadShellName(CadShellIds::Cube.to_string()),
                CadMeshIds::Cube.to_string() + &i.to_string(),
                cube_mesh_builder(
                    self,
                    CadShellName(CadShellIds::Cube.to_string()),
                    shells_by_name,
                    -(i as f32 * std::f32::consts::TAU / cubes_count as f32
                        + std::f32::consts::FRAC_PI_8),
                )?,
            )?;
        }

        Ok(cad_meshes_lazy_builders_by_cad_shell)
    }
}

impl PmetraInteractions for SimpleCubeAtCylinder {
    fn sliders(&self, shells_by_name: &CadShellsByName) -> Result<CadSliders> {
        let sliders = CadSliders::default()
            .add_slider(
                CadSliderIds::CylinderRadius.to_string().into(),
                build_radius_slider(self, shells_by_name)?,
            )?
            .add_slider(
                CadSliderIds::CubeSideLength.to_string().into(),
                build_side_length_slider(self, shells_by_name)?,
            )?;

        Ok(sliders)
    }

    fn on_slider_transform(
        &mut self,
        name: CadSliderName,
        prev_transform: Transform,
        new_transform: Transform,
    ) {
        match CadSliderIds::from_str(&name.0).unwrap() {
            CadSliderIds::CylinderRadius => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.;
                    let new_value = self.cylinder_radius + delta.x as f64 * sensitivity;
                    self.cylinder_radius = new_value.clamp(0.01, f64::MAX);
                }
            }
            CadSliderIds::CubeSideLength => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.;
                    let new_value = self.cube_side_length + delta.y as f64 * sensitivity;
                    self.cube_side_length = new_value.clamp(0.01, f64::MAX);
                }
            }
        }
    }

    fn on_slider_tooltip(&self, name: CadSliderName) -> Result<Option<String>> {
        let tooltip = match CadSliderIds::from_str(&name.0).unwrap() {
            CadSliderIds::CylinderRadius => {
                Some(format!("cylinder_radius : {:.3}", self.cylinder_radius))
            }
            CadSliderIds::CubeSideLength => {
                Some(format!("cube_side_length : {:.3}", self.cube_side_length))
            }
        };

        Ok(tooltip)
    }
}
