use std::str::FromStr;

use anyhow::{Ok, Result};
use bevy::{math::DVec3, prelude::*, transform};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::prelude::*;
use strum::{Display, EnumString};

use self::cabin::{
    build_cabin_mesh, build_cabin_shell, build_corner_radius_slider, build_extrude_slider,
    build_profile_height_slider, build_profile_thickness_slider, build_profile_width_slider,
    build_window_translation_slider,
};

use super::RoundRectCuboid;

pub mod cabin;

/// Basic Parametric Station Segment.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct RoundCabinSegment {
    #[inspector(min = 0.1)]
    pub profile_width: f64,
    #[inspector(min = 0.1)]
    pub profile_height: f64,
    #[inspector(min = 0.01)]
    pub profile_corner_radius: f64,
    #[inspector(min = 0.01)]
    pub profile_thickness: f64,
    #[inspector(min = 0.1)]
    pub profile_extrude_length: f64,
    /// Params for windows.
    pub window: RoundRectCuboid,
    window_translation: DVec3,
}

impl Default for RoundCabinSegment {
    fn default() -> Self {
        Self {
            profile_width: 1.3,
            profile_height: 1.,
            profile_corner_radius: 0.1,
            profile_thickness: 0.03,
            profile_extrude_length: 1.2,
            window: RoundRectCuboid {
                profile_width: 0.5,
                profile_height: 0.4,
                profile_corner_radius: 0.04,
                ..default()
            },
            window_translation: DVec3::new(0., 0.3, 0.84),
        }
    }
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadShellIds {
    CabinShell,
    EndWall,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadMeshIds {
    CabinShell,
    EndWall,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadSliderIds {
    ExtrudeSlider,
    CornerRadiusSlider,
    ProfileThicknessSlider,
    ProfileHeightSlider,
    ProfileWidthSlider,
    WindowTranslationSlider,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadMaterialIds {
    Base,
    Roof,
}

impl PmetraCad for RoundCabinSegment {
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
        let builders = CadShellsBuilders::new(self.clone())? // builder
            .add_shell_builder(
                CadShellName(CadShellIds::CabinShell.to_string()),
                build_cabin_shell,
            )?;

        Ok(builders)
    }
}

impl PmetraModelling for RoundCabinSegment {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesBuildersByCadShell<Self>> {
        let cad_meshes_builders_by_cad_shell =
            CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                .add_mesh_builder_with_outlines(
                    CadShellName(CadShellIds::CabinShell.to_string()),
                    CadMeshIds::CabinShell.to_string(),
                    build_cabin_mesh(self, CadShellName(CadShellIds::CabinShell.to_string()))?,
                )?;

        Ok(cad_meshes_builders_by_cad_shell)
    }
}

impl PmetraInteractions for RoundCabinSegment {
    fn sliders(&self, shells_by_name: &CadShellsByName) -> Result<CadSliders> {
        let sliders = CadSliders::default() // builder
            .add_slider(
                CadSliderIds::ExtrudeSlider.to_string().into(),
                build_extrude_slider(self, shells_by_name)?,
            )?
            .add_slider(
                CadSliderIds::CornerRadiusSlider.to_string().into(),
                build_corner_radius_slider(self, shells_by_name)?,
            )?
            .add_slider(
                CadSliderIds::ProfileWidthSlider.to_string().into(),
                build_profile_width_slider(self, shells_by_name)?,
            )?
            .add_slider(
                CadSliderIds::ProfileHeightSlider.to_string().into(),
                build_profile_height_slider(self, shells_by_name)?,
            )?
            .add_slider(
                CadSliderIds::ProfileThicknessSlider.to_string().into(),
                build_profile_thickness_slider(self, shells_by_name)?,
            )?
            .add_slider(
                CadSliderIds::WindowTranslationSlider.to_string().into(),
                build_window_translation_slider(self, shells_by_name)?,
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
            CadSliderIds::ExtrudeSlider => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.0;
                    let new_value = self.profile_extrude_length + delta.z as f64 * sensitivity;
                    self.profile_extrude_length = new_value.clamp(0.001, std::f64::MAX);
                }
            }
            CadSliderIds::CornerRadiusSlider => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.0;
                    let new_value = self.profile_corner_radius + delta.x as f64 * sensitivity;
                    self.profile_corner_radius = new_value.clamp(
                        0.001,
                        (self.profile_height / 2.).min(self.profile_width / 2.),
                    );
                }
            }
            CadSliderIds::ProfileWidthSlider => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.0;
                    let new_value = self.profile_width + delta.x as f64 * sensitivity;
                    self.profile_width = new_value.clamp(
                        (self.profile_corner_radius * 2.).max(self.profile_thickness * 2.) + 0.1,
                        std::f64::MAX,
                    );
                }
            }
            CadSliderIds::ProfileThicknessSlider => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.0;
                    let new_value = self.profile_thickness + delta.x as f64 * sensitivity;
                    self.profile_thickness = new_value.clamp(
                        0.02,
                        ((self.profile_width / 2.).min(self.profile_height / 2.)
                            - self.profile_corner_radius)
                            .max(0.02),
                    );
                }
            }
            CadSliderIds::ProfileHeightSlider => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.0;
                    let new_value = self.profile_height + delta.y as f64 * sensitivity;
                    self.profile_height =
                        new_value.clamp(self.profile_corner_radius * 2. + 0.01, std::f64::MAX);
                }
            }
            CadSliderIds::WindowTranslationSlider => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.0;
                    self.window_translation += delta.as_dvec3() * sensitivity;
                }
            }
            _ => {}
        }
    }

    fn on_slider_tooltip(&self, name: CadSliderName) -> Result<Option<String>> {
        let tooltip = match CadSliderIds::from_str(&name).unwrap() {
            CadSliderIds::ExtrudeSlider => Some(format!(
                "profile_extrude_length : {:.3}",
                self.profile_extrude_length
            )),
            CadSliderIds::CornerRadiusSlider => Some(format!(
                "profile_corner_radius : {:.3}",
                self.profile_corner_radius
            )),
            CadSliderIds::ProfileWidthSlider => {
                Some(format!("profile_width : {:.3}", self.profile_width))
            }
            CadSliderIds::ProfileThicknessSlider => {
                Some(format!("profile_thickness : {:.3}", self.profile_thickness))
            }
            CadSliderIds::ProfileHeightSlider => {
                Some(format!("profile_height : {:.3}", self.profile_height))
            }
            CadSliderIds::WindowTranslationSlider => Some(format!(
                "window_translation : [{:.3}, {:.3}, {:.3}]",
                self.window_translation.x, self.window_translation.y, self.window_translation.z
            )),
            _ => None,
        };

        Ok(tooltip)
    }
}
