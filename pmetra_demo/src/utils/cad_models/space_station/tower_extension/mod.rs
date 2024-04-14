use std::str::FromStr;

use anyhow::{Ok, Result};
use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::prelude::*;
use strum::{Display, EnumString};

use self::{
    beams::{
        build_cross_beam_shell, build_straight_beam_shell, cross_beam_mesh_builder,
        straight_beam_mesh_builder,
    },
    cuboid_enclosure::{
        build_cuboid_enclosure_shell, build_tower_length_slider, cuboid_enclosure_mesh_builder,
    },
};

pub mod beams;
pub mod cuboid_enclosure;

/// Basic Parametric Station Segment.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct TowerExtension {
    #[inspector(min = 0.1)]
    pub tower_length: f64,
    // Straight Beams...
    #[inspector(min = 0.1)]
    pub straight_beam_l_sect_side_len: f64,
    #[inspector(min = 0.1)]
    pub straight_beam_l_sect_thickness: f64,
    // Cross Beams...
    #[inspector(min = 0.1)]
    pub cross_beam_l_sect_side_len: f64,
    #[inspector(min = 0.1)]
    pub cross_beam_l_sect_thickness: f64,
    // Enclosure...
    #[inspector(min = 0.1)]
    pub enclosure_profile_width: f64,
    #[inspector(min = 0.1)]
    pub enclosure_profile_depth: f64,
}

impl Default for TowerExtension {
    fn default() -> Self {
        Self {
            tower_length: 1.0,
            straight_beam_l_sect_side_len: 0.05,
            straight_beam_l_sect_thickness: 0.01,
            cross_beam_l_sect_side_len: 0.05,
            cross_beam_l_sect_thickness: 0.01,
            enclosure_profile_width: 0.5,
            enclosure_profile_depth: 0.5,
        }
    }
}

impl TowerExtension {
    pub fn num_of_cross_segments(&self) -> u32 {
        (self.tower_length / 0.2).floor() as u32
    }

    pub fn cross_segment_length(&self) -> f64 {
        (self.tower_length - self.cross_beam_l_sect_side_len * 2.)
            / self.num_of_cross_segments() as f64
    }

    pub fn enclosure_inner_width(&self) -> f64 {
        self.enclosure_profile_width - self.straight_beam_l_sect_thickness * 2.
    }

    pub fn cross_beam_hypotenuse(&self) -> f64 {
        (self.cross_segment_length().powi(2) + self.enclosure_inner_width().powi(2)).sqrt()
    }

    pub fn cross_beam_length(&self) -> f64 {
        (self.cross_beam_hypotenuse().powi(2) - self.cross_beam_l_sect_side_len.powi(2)).sqrt()
    }

    pub fn cross_beam_hyp_angle_z(&self) -> f64 {
        std::f64::consts::FRAC_PI_2
            - (self.cross_segment_length() / self.enclosure_inner_width()).atan()
    }

    pub fn cross_beam_angle_z(&self) -> f64 {
        self.cross_beam_hyp_angle_z()
            - (self.cross_beam_l_sect_side_len / self.cross_beam_length()).atan()
    }

    pub fn cross_beam_y_offset(&self) -> f64 {
        let cross_beam_bot_angle =
            std::f64::consts::PI - self.cross_beam_angle_z() - std::f64::consts::FRAC_PI_2;
        self.cross_beam_l_sect_side_len * cross_beam_bot_angle.cos()
    }
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadShellIds {
    CuboidEnclosure,
    StraightBeam,
    CrossBeam,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadMeshIds {
    CuboidEnclosure,
    StraightBeam,
    CrossBeam,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadSliderIds {
    TowerLengthSlider,
}

impl PmetraCad for TowerExtension {
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
        let builders = CadShellsBuilders::new(self.clone())? // builder
            .add_shell_builder(
                CadShellName(CadShellIds::CuboidEnclosure.to_string()),
                build_cuboid_enclosure_shell,
            )?
            .add_shell_builder(
                CadShellName(CadShellIds::StraightBeam.to_string()),
                build_straight_beam_shell,
            )?
            .add_shell_builder(
                CadShellName(CadShellIds::CrossBeam.to_string()),
                build_cross_beam_shell,
            )?;

        Ok(builders)
    }
}

impl PmetraModelling for TowerExtension {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesBuildersByCadShell<Self>> {
        // Create enclosure...
        let mut cad_meshes_builders_by_cad_shell =
            CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                .add_mesh_builder(
                    CadShellName(CadShellIds::CuboidEnclosure.to_string()),
                    CadMeshIds::CuboidEnclosure.to_string(),
                    cuboid_enclosure_mesh_builder(
                        self,
                        CadShellName(CadShellIds::CuboidEnclosure.to_string()),
                    )?,
                )?;
        // Create straight beams...
        let straight_beam_transforms = [
            // back-left
            Transform::from_translation(Vec3::new(
                -self.enclosure_profile_width as f32 / 2.,
                0.,
                -self.enclosure_profile_depth as f32 / 2.,
            ))
            .with_rotation(Quat::from_rotation_y(0.)),
            // back-right
            Transform::from_translation(Vec3::new(
                self.enclosure_profile_width as f32 / 2.,
                0.,
                -self.enclosure_profile_depth as f32 / 2.,
            ))
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
            // front-right
            Transform::from_translation(Vec3::new(
                self.enclosure_profile_width as f32 / 2.,
                0.,
                self.enclosure_profile_depth as f32 / 2.,
            ))
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::PI)),
            // front-left
            Transform::from_translation(Vec3::new(
                -self.enclosure_profile_width as f32 / 2.,
                0.,
                self.enclosure_profile_depth as f32 / 2.,
            ))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        ];
        for (idx, transform) in straight_beam_transforms.iter().enumerate() {
            cad_meshes_builders_by_cad_shell.add_mesh_builder(
                CadShellName(CadShellIds::StraightBeam.to_string()),
                CadMeshIds::StraightBeam.to_string() + &idx.to_string(),
                straight_beam_mesh_builder(
                    self,
                    CadShellName(CadShellIds::StraightBeam.to_string()),
                    *transform,
                )?,
            )?;
        }
        // Create cross beams...
        let base_transforms = [
            Transform::default(), // front
            Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)), // right
            Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI)), // back
            Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)), // left
        ];
        for (idx, base_transform) in base_transforms.iter().enumerate() {
            let org_transform = Transform::from_translation(Vec3::new(
                -self.enclosure_profile_width as f32 / 2.
                    + self.straight_beam_l_sect_thickness as f32,
                self.cross_beam_y_offset() as f32,
                self.enclosure_profile_depth as f32 / 2.
                    - self.straight_beam_l_sect_thickness as f32,
            ))
            .with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0.,
                std::f32::consts::FRAC_PI_2,
                0.,
            ));
            let cross_beam_angle_z = self.cross_beam_angle_z();
            let num_of_cross_segments = self.num_of_cross_segments();
            let cross_segment_length = self.cross_segment_length();
            for jdx in 0..num_of_cross_segments {
                //  Add idx: adjustment to flip cross beams pos so that they appear connected between sides....
                let jdx = jdx + idx as u32;
                let mut transform = org_transform;
                transform
                    .rotate_y(std::f32::consts::FRAC_PI_2 * if jdx % 2 == 0 { 0. } else { 1. });
                transform.rotate_z(cross_beam_angle_z as f32 * if jdx % 2 == 0 { -1. } else { 1. });
                transform.translation.x += if jdx % 2 == 0 {
                    0.
                } else {
                    self.enclosure_inner_width() as f32
                };
                //  Sub idx: adjustment to flip cross beams pos so that they appear connected between sides....
                transform.translation.y +=
                    (jdx - idx as u32) as f32 * (cross_segment_length as f32);

                cad_meshes_builders_by_cad_shell.add_mesh_builder(
                    CadShellName(CadShellIds::CrossBeam.to_string()),
                    CadMeshIds::CrossBeam.to_string()
                        + "-"
                        + &idx.to_string()
                        + "-"
                        + &jdx.to_string(),
                    cross_beam_mesh_builder(
                        self,
                        CadShellName(CadShellIds::CrossBeam.to_string()),
                        *base_transform * transform,
                    )?,
                )?;
            }
        }

        Ok(cad_meshes_builders_by_cad_shell)
    }

    fn sliders(&self, shells_by_name: &CadShellsByName) -> Result<CadSliders> {
        let sliders = CadSliders::default().add_slider(
            CadSliderIds::TowerLengthSlider.to_string().into(),
            build_tower_length_slider(self, shells_by_name)?,
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
            CadSliderIds::TowerLengthSlider => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.;
                    let new_value = self.tower_length + delta.y as f64 * sensitivity;
                    self.tower_length = new_value.clamp(0.01, std::f64::MAX);
                }
            }
        }
    }

    fn on_slider_tooltip(&self, name: CadSliderName) -> Result<Option<String>> {
        let tooltip = match CadSliderIds::from_str(&name.0).unwrap() {
            CadSliderIds::TowerLengthSlider => {
                Some(format!("tower_length : {:.3}", self.tower_length))
            }
        };

        Ok(tooltip)
    }
}
