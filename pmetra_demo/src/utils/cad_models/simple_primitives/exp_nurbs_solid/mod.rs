use std::str::FromStr;

use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::{prelude::*, re_exports::anyhow::Result};
use strum::{Display, EnumString};

use self::cylinder::{build_cylinder_shell, build_radius_slider, cylinder_mesh_builder};

pub mod cylinder;

/// Basic Parametric Station Segment.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct ExpNurbsSolid {
    #[inspector(min = 0.1)]
    pub cylinder_radius: f64,
    #[inspector(min = 0.1)]
    pub cylinder_height: f64,
}

impl Default for ExpNurbsSolid {
    fn default() -> Self {
        Self {
            cylinder_radius: 1.2,
            cylinder_height: 0.5,
        }
    }
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadShellIds {
    Cylinder,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadMeshIds {
    Cylinder,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadSliderIds {
    CylinderRadius,
}

impl PmetraCad for ExpNurbsSolid {
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
        let builders = CadShellsBuilders::new(self.clone())? // builder
            .add_shell_builder(
                CadShellName(CadShellIds::Cylinder.to_string()),
                build_cylinder_shell,
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
        let mut cad_meshes_lazy_builders_by_cad_shell =
            CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                .add_mesh_builder_with_outlines(
                    CadShellName(CadShellIds::Cylinder.to_string()),
                    CadMeshIds::Cylinder.to_string(),
                    cylinder_mesh_builder(self, CadShellName(CadShellIds::Cylinder.to_string()))?,
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
        }
    }

    fn on_slider_tooltip(&self, name: CadSliderName) -> Result<Option<String>> {
        let tooltip = match CadSliderIds::from_str(&name.0).unwrap() {
            CadSliderIds::CylinderRadius => {
                Some(format!("cylinder_radius : {:.3}", self.cylinder_radius))
            }
        };

        Ok(tooltip)
    }
}
