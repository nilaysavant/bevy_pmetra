use std::str::FromStr;

use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::{prelude::*, re_exports::anyhow::Result};
use strum::{Display, EnumString};

use self::{
    cylinder::{
        build_cylinder_shell, build_height_slider, build_move_slider, build_radius_slider,
        cylinder_mesh_builder,
    },
    nurbs_surface::{
        build_control_point_slider, build_nurbs_surface_shell, build_surface_length_slider,
        nurbs_surface_mesh_builder,
    },
};

pub mod cylinder;
pub mod nurbs_surface;

/// Basic Parametric Station Segment.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct ExpNurbsSolid {
    #[inspector(min = 0.1)]
    pub cylinder_radius: f64,
    #[inspector(min = 0.1)]
    pub cylinder_height: f64,
    #[inspector(min = -10., max = 10.)]
    pub cylinder_translation: [f32; 3],
    #[inspector(min = 0.1)]
    pub control_point_spacing: f32,
    #[inspector(min = 0.1)]
    pub surface_length: f32,
    #[inspector(min = -10., max = 10.)]
    pub control_points: [[f32; 3]; 8],
}

impl Default for ExpNurbsSolid {
    fn default() -> Self {
        Self {
            cylinder_radius: 1.2,
            cylinder_height: 0.5,
            cylinder_translation: [0.0, 0.0, 0.0],
            control_point_spacing: 1.0,
            surface_length: 1.0,
            control_points: default_control_points(1.0, 1.0),
        }
    }
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadShellIds {
    Cylinder,
    NurbsSurface,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadMeshIds {
    Cylinder,
    NurbsSurface,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadSliderIds {
    CylinderRadius,
    CylinderHeight,
    CylinderMove,
    SurfaceLength,
    ControlPoint0,
    ControlPoint1,
    ControlPoint2,
    ControlPoint3,
    ControlPoint4,
    ControlPoint5,
    ControlPoint6,
    ControlPoint7,
}

impl PmetraCad for ExpNurbsSolid {
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
        let builders = CadShellsBuilders::new(self.clone())? // builder
            .add_shell_builder(
                CadShellName(CadShellIds::Cylinder.to_string()),
                build_cylinder_shell,
            )?
            .add_shell_builder(
                CadShellName(CadShellIds::NurbsSurface.to_string()),
                build_nurbs_surface_shell,
            )?
            ;
        Ok(builders)
    }
}

impl PmetraModelling for ExpNurbsSolid {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesBuildersByCadShell<Self>> {
        let cad_meshes_lazy_builders_by_cad_shell =
            CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                .add_mesh_builder_with_outlines(
                    CadShellName(CadShellIds::Cylinder.to_string()),
                    CadMeshIds::Cylinder.to_string(),
                    cylinder_mesh_builder(self, CadShellName(CadShellIds::Cylinder.to_string()))?,
                )?
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

impl PmetraInteractions for ExpNurbsSolid {
    fn sliders(&self, shells_by_name: &CadShellsByName) -> Result<CadSliders> {
        let sliders = CadSliders::default()
            .add_slider(
                CadSliderIds::CylinderRadius.to_string().into(),
                build_radius_slider(self, shells_by_name)?,
            )?
            .add_slider(
                CadSliderIds::CylinderHeight.to_string().into(),
                build_height_slider(self, shells_by_name)?,
            )?
            .add_slider(
                CadSliderIds::CylinderMove.to_string().into(),
                build_move_slider(self, shells_by_name)?,
            )?
            .add_slider(
                CadSliderIds::SurfaceLength.to_string().into(),
                build_surface_length_slider(self)?,
            )?;

        let mut sliders = sliders;
        for index in 0..self.control_points.len() {
            let slider_id = control_point_slider_id(index);
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
        match CadSliderIds::from_str(&name.0).unwrap() {
            CadSliderIds::CylinderRadius => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.;
                    let new_value = self.cylinder_radius + delta.x as f64 * sensitivity;
                    self.cylinder_radius = new_value.clamp(0.01, f64::MAX);
                }
            }
            CadSliderIds::CylinderHeight => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length_squared() > 0. {
                    let sensitivity = 1.0;
                    let new_value = self.cylinder_height + delta.y as f64 * sensitivity;
                    self.cylinder_height = new_value.clamp(0.01, f64::MAX);
                }
            }
            CadSliderIds::CylinderMove => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length_squared() > 0. {
                    self.cylinder_translation[0] += delta.x;
                    self.cylinder_translation[2] += delta.z;
                }
            }
            CadSliderIds::SurfaceLength => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length_squared() > 0. {
                    let sensitivity = 1.0;
                    let new_value = self.surface_length + delta.z * sensitivity;
                    self.surface_length = new_value.clamp(0.1, f32::MAX);
                    apply_surface_length_to_control_points(&mut self.control_points, self.surface_length);
                }
            }
            CadSliderIds::ControlPoint0
            | CadSliderIds::ControlPoint1
            | CadSliderIds::ControlPoint2
            | CadSliderIds::ControlPoint3
            | CadSliderIds::ControlPoint4
            | CadSliderIds::ControlPoint5
            | CadSliderIds::ControlPoint6
            | CadSliderIds::ControlPoint7 => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length_squared() > 0. {
                    let index = control_point_index_from_slider(&name);
                    let point = &mut self.control_points[index];
                    point[0] += delta.x;
                    point[1] += delta.y;
                    point[2] = control_point_z_for_index(index, self.surface_length);
                }
            }
        }
    }

    fn on_slider_tooltip(&self, name: CadSliderName) -> Result<Option<String>> {
        let tooltip = match CadSliderIds::from_str(&name.0).unwrap() {
            CadSliderIds::CylinderRadius => {
                Some(format!("cylinder_radius : {:.3}", self.cylinder_radius))
            }
            CadSliderIds::CylinderHeight => {
                Some(format!("cylinder_height : {:.3}", self.cylinder_height))
            }
            CadSliderIds::CylinderMove => Some(format!(
                "cylinder_translation : ({:.3}, {:.3}, {:.3})",
                self.cylinder_translation[0],
                self.cylinder_translation[1],
                self.cylinder_translation[2]
            )),
            CadSliderIds::SurfaceLength => {
                Some(format!("surface_length : {:.3}", self.surface_length))
            }
            CadSliderIds::ControlPoint0
            | CadSliderIds::ControlPoint1
            | CadSliderIds::ControlPoint2
            | CadSliderIds::ControlPoint3
            | CadSliderIds::ControlPoint4
            | CadSliderIds::ControlPoint5
            | CadSliderIds::ControlPoint6
            | CadSliderIds::ControlPoint7 => {
                let index = control_point_index_from_slider(&name);
                let point = self.control_points[index];
                Some(format!(
                    "control_point_{} : ({:.3}, {:.3}, {:.3})",
                    index, point[0], point[1], point[2]
                ))
            }
        };

        Ok(tooltip)
    }
}

fn control_point_index_from_slider(name: &CadSliderName) -> usize {
    match CadSliderIds::from_str(&name.0).unwrap() {
        CadSliderIds::ControlPoint0 => 0,
        CadSliderIds::ControlPoint1 => 1,
        CadSliderIds::ControlPoint2 => 2,
        CadSliderIds::ControlPoint3 => 3,
        CadSliderIds::ControlPoint4 => 4,
        CadSliderIds::ControlPoint5 => 5,
        CadSliderIds::ControlPoint6 => 6,
        CadSliderIds::ControlPoint7 => 7,
        CadSliderIds::CylinderRadius
        | CadSliderIds::CylinderHeight
        | CadSliderIds::CylinderMove
        | CadSliderIds::SurfaceLength => 0,
    }
}

fn control_point_slider_id(index: usize) -> CadSliderIds {
    match index {
        0 => CadSliderIds::ControlPoint0,
        1 => CadSliderIds::ControlPoint1,
        2 => CadSliderIds::ControlPoint2,
        3 => CadSliderIds::ControlPoint3,
        4 => CadSliderIds::ControlPoint4,
        5 => CadSliderIds::ControlPoint5,
        6 => CadSliderIds::ControlPoint6,
        _ => CadSliderIds::ControlPoint7,
    }
}

fn default_control_points(spacing: f32, surface_length: f32) -> [[f32; 3]; 8] {
    let x0 = 0.0;
    let x1 = spacing;
    let x2 = spacing * 2.0;
    let x3 = spacing * 3.0;
    let z0 = 0.0;
    let z1 = surface_length;

    [
        [x0, 0.0, z0],
        [x1, 0.0, z0],
        [x2, 0.0, z0],
        [x3, 0.0, z0],
        [x0, 0.0, z1],
        [x1, 0.0, z1],
        [x2, 0.0, z1],
        [x3, 0.0, z1],
    ]
}

fn apply_surface_length_to_control_points(points: &mut [[f32; 3]; 8], surface_length: f32) {
    for (index, point) in points.iter_mut().enumerate() {
        point[2] = control_point_z_for_index(index, surface_length);
    }
}

fn control_point_z_for_index(index: usize, surface_length: f32) -> f32 {
    if index < 4 {
        0.0
    } else {
        surface_length
    }
}
