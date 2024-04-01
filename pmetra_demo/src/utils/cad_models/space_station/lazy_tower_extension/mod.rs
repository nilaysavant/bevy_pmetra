use std::str::FromStr;

use anyhow::{Ok, Result};
use bevy::{prelude::*, transform};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::{
    cad_core::lazy_builders::{
        CadMeshesLazyBuildersByCadShell, CadShellName, CadShellsByName, CadShellsLazyBuilders,
        ParametricLazyCad, ParametricLazyModelling,
    },
    prelude::*,
};
use strum::{Display, EnumString};

use self::{
    beams::{
        build_cross_beam_shell, build_straight_beam_shell, cross_beam_mesh_builder,
        straight_beam_mesh_builder,
    },
    cuboid_enclosure::{
        build_cuboid_enclosure_shell, build_tower_length_cursor, cuboid_enclosure_mesh_builder,
    },
};

pub mod beams;
pub mod cuboid_enclosure;

/// Basic Parametric Station Segment.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct LazyTowerExtension {
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

impl Default for LazyTowerExtension {
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

impl LazyTowerExtension {
    pub fn num_of_cross_segments(&self) -> u32 {
        (self.tower_length / 0.5).floor() as u32
    }
    pub fn cross_segment_length(&self) -> f64 {
        (self.tower_length - self.cross_beam_l_sect_side_len * 2.)
            / self.num_of_cross_segments() as f64
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
pub enum CadCursorIds {
    TowerLengthCursor,
}

impl ParametricLazyModelling for LazyTowerExtension {
    fn shells_builders(&self) -> Result<CadShellsLazyBuilders<Self>> {
        let builders = CadShellsLazyBuilders::new(self.clone())? // builder
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

impl ParametricLazyCad for LazyTowerExtension {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesLazyBuildersByCadShell<Self>> {
        // Create enclosure...
        let mut cad_meshes_lazy_builders_by_cad_shell =
            CadMeshesLazyBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
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
            Transform::from_translation(Vec3::new(
                -self.enclosure_profile_width as f32 / 2.,
                0.,
                -self.enclosure_profile_depth as f32 / 2.,
            ))
            .with_rotation(Quat::from_rotation_y(0.)),
            Transform::from_translation(Vec3::new(
                self.enclosure_profile_width as f32 / 2.,
                0.,
                -self.enclosure_profile_depth as f32 / 2.,
            ))
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
            Transform::from_translation(Vec3::new(
                self.enclosure_profile_width as f32 / 2.,
                0.,
                self.enclosure_profile_depth as f32 / 2.,
            ))
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::PI)),
            Transform::from_translation(Vec3::new(
                -self.enclosure_profile_width as f32 / 2.,
                0.,
                self.enclosure_profile_depth as f32 / 2.,
            ))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        ];
        for (idx, transform) in straight_beam_transforms.iter().enumerate() {
            cad_meshes_lazy_builders_by_cad_shell.add_mesh_builder(
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
        let cross_beams_proj_width = self.enclosure_profile_width
            - self.straight_beam_l_sect_side_len
            - self.straight_beam_l_sect_thickness * 2.
            - 0.02;
        let cross_beam_angle_z = std::f64::consts::FRAC_PI_2
            - (self.cross_segment_length() / (cross_beams_proj_width)).atan();
        let org_transform = Transform::from_translation(Vec3::new(
            -self.enclosure_profile_width as f32 / 2. + self.straight_beam_l_sect_thickness as f32,
            self.cross_beam_l_sect_side_len as f32 / 2. + 0.01,
            self.enclosure_profile_depth as f32 / 2. - self.straight_beam_l_sect_thickness as f32,
        ))
        .with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.,
            std::f32::consts::FRAC_PI_2,
            0.,
        ));
        for idx in 0..self.num_of_cross_segments() {
            let mut transform = org_transform;
            transform.rotate_y(std::f32::consts::FRAC_PI_2 * if idx % 2 == 0 { 0. } else { 1. });
            transform.rotate_z(cross_beam_angle_z as f32 * if idx % 2 == 0 { -1. } else { 1. });
            transform.translation.x += if idx % 2 == 0 {
                0.
            } else {
                cross_beams_proj_width as f32
                    + self.straight_beam_l_sect_side_len as f32
                    + self.straight_beam_l_sect_thickness as f32
                    + 0.01
            };
            transform.translation.y += idx as f32
                * (self.cross_segment_length() as f32
                    + self.straight_beam_l_sect_thickness as f32 * 2.);

            cad_meshes_lazy_builders_by_cad_shell.add_mesh_builder(
                CadShellName(CadShellIds::CrossBeam.to_string()),
                CadMeshIds::CrossBeam.to_string() + &idx.to_string(),
                cross_beam_mesh_builder(
                    self,
                    CadShellName(CadShellIds::CrossBeam.to_string()),
                    transform,
                )?,
            )?;
        }

        Ok(cad_meshes_lazy_builders_by_cad_shell)
    }

    fn cursors(&self, shells_by_name: &CadShellsByName) -> Result<CadCursors> {
        let cursors = CadCursors::default().add_cursor(
            CadCursorIds::TowerLengthCursor.to_string().into(),
            build_tower_length_cursor(self, shells_by_name)?,
        )?;

        Ok(cursors)
    }

    fn on_cursor_transform(
        &mut self,
        cursor_name: CadCursorName,
        prev_transform: Transform,
        new_transform: Transform,
    ) {
        match CadCursorIds::from_str(&cursor_name.0).unwrap() {
            CadCursorIds::TowerLengthCursor => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.;
                    let new_value = self.tower_length + delta.y as f64 * sensitivity;
                    self.tower_length = new_value.clamp(0.01, std::f64::MAX);
                }
            }
        }
    }

    fn on_cursor_tooltip(&self, cursor_name: CadCursorName) -> Result<String> {
        let tooltip = match CadCursorIds::from_str(&cursor_name.0).unwrap() {
            CadCursorIds::TowerLengthCursor => {
                format!("tower_length : {:.3}", self.tower_length)
            }
        };

        Ok(tooltip)
    }
}
