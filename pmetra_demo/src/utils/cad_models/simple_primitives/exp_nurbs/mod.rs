use std::str::FromStr;

use bevy::{math::DVec3, prelude::*};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::{prelude::*, re_exports::anyhow::Result};
use strum::{Display, EnumString};

use self::nurbs_surface::{
    build_control_point_slider, build_nurbs_surface_shell, build_surface_length_slider,
    nurbs_surface_mesh_builder,
};

pub mod nurbs_surface;

/// Experimental NURBS Surface Solid.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct ExpNurbs {
    #[inspector(min = 0.1)]
    pub control_point_spacing: f64,
    #[inspector(min = 0.1)]
    pub surface_length: f64,
    #[inspector(min = 0.1)]
    pub surface_thickness: f64,
    pub control_points: [DVec3; Self::CONTROL_POINTS_COUNT],
}

impl ExpNurbs {
    pub const CONTROL_POINTS_COUNT: usize = 8;
    pub const U_COUNT: usize = 4;
    pub const V_COUNT: usize = 2;

    pub fn new(control_point_spacing: f64, surface_length: f64, surface_thickness: f64) -> Self {
        let control_points =
            Self::default_control_points(control_point_spacing, surface_length, surface_thickness);
        Self {
            control_point_spacing,
            surface_length,
            surface_thickness,
            control_points,
        }
    }

    pub fn default_control_points(
        control_point_spacing: f64,
        surface_length: f64,
        surface_thickness: f64,
    ) -> [DVec3; Self::CONTROL_POINTS_COUNT] {
        let x0 = 0.0;
        let x1 = control_point_spacing;
        let x2 = control_point_spacing * 2.0;
        let x3 = control_point_spacing * 3.0;
        let z0 = 0.0;
        let z1 = surface_length;
        let y = surface_thickness;

        let control_points = [
            DVec3::new(x0, y, z0),
            DVec3::new(x1, y, z0),
            DVec3::new(x2, y, z0),
            DVec3::new(x3, y, z0),
            DVec3::new(x0, y, z1),
            DVec3::new(x1, y, z1),
            DVec3::new(x2, y, z1),
            DVec3::new(x3, y, z1),
        ];

        control_points
    }

    pub fn apply_surface_length_to_control_points(&mut self) {
        for (index, point) in self.control_points.iter_mut().enumerate() {
            point.z = Self::control_point_z_for_index(index, self.surface_length);
        }
    }

    pub fn control_point_z_for_index(index: usize, surface_length: f64) -> f64 {
        if index < 4 {
            0.0
        } else {
            surface_length
        }
    }
}

impl Default for ExpNurbs {
    fn default() -> Self {
        let control_point_spacing = 1.0;
        let surface_length = 1.0;
        let surface_thickness = 1.0;

        Self::new(control_point_spacing, surface_length, surface_thickness)
    }
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadShellIds {
    NurbsSurface,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadMeshIds {
    NurbsSurface,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadSliderIds {
    SurfaceLength,
    /// Control point slider ID with index 0-7.
    #[strum(to_string = "ControlPoint-{0}")]
    ControlPoint(u8),
}

impl From<CadSliderName> for CadSliderIds {
    fn from(value: CadSliderName) -> Self {
        if let Ok(slider_id) = CadSliderIds::from_str(&value.0) {
            return slider_id;
        }
        let parts: Vec<&str> = value.0.split('-').collect();
        if parts.len() == 2 && parts[0] == "ControlPoint" {
            if let Ok(index) = parts[1].parse::<u8>() {
                return CadSliderIds::ControlPoint(index);
            }
        }
        panic!("Invalid ControlPoint slider name format");
    }
}

impl PmetraCad for ExpNurbs {
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
        let builders = CadShellsBuilders::new(self.clone())? // builder
            .add_shell_builder(
                CadShellName(CadShellIds::NurbsSurface.to_string()),
                build_nurbs_surface_shell,
            )?;
        Ok(builders)
    }
}

impl PmetraModelling for ExpNurbs {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesBuildersByCadShell<Self>> {
        let cad_meshes_lazy_builders_by_cad_shell =
            CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                .add_mesh_builder_with_outlines(
                    CadShellName(CadShellIds::NurbsSurface.to_string()),
                    CadMeshIds::NurbsSurface.to_string(),
                    nurbs_surface_mesh_builder(
                        self,
                        CadShellName(CadShellIds::NurbsSurface.to_string()),
                    )?,
                )?;

        Ok(cad_meshes_lazy_builders_by_cad_shell)
    }
}

impl PmetraInteractions for ExpNurbs {
    fn sliders(&self, _shells_by_name: &CadShellsByName) -> Result<CadSliders> {
        let mut sliders = CadSliders::default();
        sliders = sliders.add_slider(
            CadSliderIds::SurfaceLength.to_string().into(),
            build_surface_length_slider(self)?,
        )?;
        for index in 0..self.control_points.len() {
            let slider_id = CadSliderIds::ControlPoint(index as u8);
            sliders = sliders.add_slider(
                slider_id.to_string().into(),
                build_control_point_slider(self, index)?,
            )?;
        }

        Ok(sliders)
    }

    fn on_slider_transform(
        &mut self,
        name: CadSliderName,
        prev_transform: Transform,
        new_transform: Transform,
    ) {
        match CadSliderIds::from(name) {
            CadSliderIds::SurfaceLength => {
                let delta = (new_transform.translation - prev_transform.translation).as_dvec3();
                if delta.length_squared() > 0. {
                    let sensitivity = 1.0;
                    let new_value = self.surface_length + delta.z * sensitivity;
                    self.surface_length = new_value.clamp(0.1, f64::MAX);
                    self.apply_surface_length_to_control_points();
                }
            }
            CadSliderIds::ControlPoint(index) => {
                let delta = (new_transform.translation - prev_transform.translation).as_dvec3();
                if delta.length_squared() > 0. {
                    if let Some(point) = self.control_points.get_mut(index as usize) {
                        point.x += delta.x;
                        point.y += delta.y;
                        point.z =
                            Self::control_point_z_for_index(index.into(), self.surface_length);
                    }
                }
            }
        }
    }

    fn on_slider_tooltip(&self, name: CadSliderName) -> Result<Option<String>> {
        let tooltip = match CadSliderIds::from(name) {
            CadSliderIds::SurfaceLength => {
                Some(format!("surface_length : {:.3}", self.surface_length))
            }
            CadSliderIds::ControlPoint(index) => {
                self.control_points.get(index as usize).map(|point| {
                    format!(
                        "control_point_{} : ({:.3}, {:.3}, {:.3})",
                        index, point.x, point.y, point.z
                    )
                })
            }
        };

        Ok(tooltip)
    }
}
