use std::str::FromStr;

use anyhow::{anyhow, Context, Error, Result};
use bevy::{math::DVec3, prelude::*, utils::HashMap};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::prelude::*;
use strum::{Display, EnumString};

use self::{
    cabin::{build_cabin_mesh, build_cabin_shell, CabinCursorIds},
    end_walls::{build_end_wall_mesh, build_end_wall_shell, EndWallCursorIds},
};

use super::RoundRectCuboid;

pub mod cabin;
pub mod end_walls;

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
    #[inspector(min = 0.1)]
    pub end_wall_thickness: f64,
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
            end_wall_thickness: 0.2,
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
pub enum CadMaterialIds {
    Base,
    Roof,
}

impl ParametricModelling for RoundCabinSegment {
    fn build_shells(&self) -> Result<CadShells> {
        let cad_shells =
            CadShellsBuilder::new(self.clone())? // builder
                .add_shell(CadShellIds::CabinShell.to_string(), build_cabin_shell)?
                .add_shell(CadShellIds::EndWall.to_string(), build_end_wall_shell)?
                .build()?;

        Ok(cad_shells)
    }
}

impl ParametricCad for RoundCabinSegment {
    fn build_cad_meshes_from_shells(
        &self,
        solids: CadShells,
        textures: CadMaterialTextures<Option<Image>>,
    ) -> Result<CadMeshes> {
        CadMeshesBuilder::new(self.clone(), solids, textures)? // builder
            .add_mesh(
                CadShellIds::CabinShell.to_string(),
                CadMeshIds::CabinShell.to_string(),
                build_cabin_mesh,
            )?
            .add_mesh(
                CadShellIds::EndWall.to_string(),
                CadMeshIds::EndWall.to_string(),
                build_end_wall_mesh,
            )?
            .build()
    }

    fn on_cursor_transform(
        &mut self,
        mesh_name: CadMeshName,
        cursor_name: CadCursorName,
        prev_transform: Transform,
        new_transform: Transform,
    ) {
        match CadMeshIds::from_str(&mesh_name).unwrap() {
            CadMeshIds::CabinShell => match CabinCursorIds::from_str(&cursor_name).unwrap() {
                CabinCursorIds::ExtrudeCursor => {
                    let delta = new_transform.translation - prev_transform.translation;
                    if delta.length() > 0. {
                        let sensitivity = 1.0;
                        let new_value = self.profile_extrude_length + delta.z as f64 * sensitivity;
                        self.profile_extrude_length = new_value.clamp(0.001, std::f64::MAX);
                    }
                }
                CabinCursorIds::CornerRadiusCursor => {
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
                CabinCursorIds::ProfileThicknessCursor => {
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
                CabinCursorIds::ProfileHeightCursor => {
                    let delta = new_transform.translation - prev_transform.translation;
                    if delta.length() > 0. {
                        let sensitivity = 1.0;
                        let new_value = self.profile_height + delta.y as f64 * sensitivity;
                        self.profile_height =
                            new_value.clamp(self.profile_corner_radius * 2. + 0.01, std::f64::MAX);
                    }
                }
                CabinCursorIds::ProfileWidthCursor => {
                    let delta = new_transform.translation - prev_transform.translation;
                    if delta.length() > 0. {
                        let sensitivity = 1.0;
                        let new_value = self.profile_width + delta.x as f64 * sensitivity;
                        self.profile_width = new_value.clamp(
                            (self.profile_corner_radius * 2.).max(self.profile_thickness * 2.)
                                + 0.1,
                            std::f64::MAX,
                        );
                    }
                }
                CabinCursorIds::WindowTranslationCursor => {
                    let delta = new_transform.translation - prev_transform.translation;
                    if delta.length() > 0. {
                        let sensitivity = 1.0;
                        self.window_translation += delta.as_dvec3() * sensitivity;
                    }
                }
            },
            CadMeshIds::EndWall => match EndWallCursorIds::from_str(&cursor_name).unwrap() {
                EndWallCursorIds::ExtrudeCursor => {
                    let delta = new_transform.translation - prev_transform.translation;
                    if delta.length() > 0. {
                        let sensitivity = 0.2;
                        let new_value = self.end_wall_thickness - delta.z as f64 * sensitivity;
                        self.end_wall_thickness = new_value.clamp(0.001, std::f64::MAX);
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
            CadMeshIds::CabinShell => match CabinCursorIds::from_str(&cursor_name).unwrap() {
                CabinCursorIds::ExtrudeCursor => {
                    format!(
                        "profile_extrude_length : {:.3}",
                        self.profile_extrude_length
                    )
                }
                CabinCursorIds::CornerRadiusCursor => {
                    format!("profile_corner_radius : {:.3}", self.profile_corner_radius)
                }
                CabinCursorIds::ProfileThicknessCursor => {
                    format!("profile_thickness : {:.3}", self.profile_thickness)
                }
                CabinCursorIds::ProfileHeightCursor => {
                    format!("profile_height : {:.3}", self.profile_height)
                }
                CabinCursorIds::ProfileWidthCursor => {
                    format!("profile_width : {:.3}", self.profile_width)
                }
                CabinCursorIds::WindowTranslationCursor => {
                    format!(
                        "window_translation : [{:.3}, {:.3}, {:.3}]",
                        self.window_translation.x,
                        self.window_translation.y,
                        self.window_translation.z
                    )
                }
            },
            CadMeshIds::EndWall => match EndWallCursorIds::from_str(&cursor_name).unwrap() {
                EndWallCursorIds::ExtrudeCursor => {
                    format!("end_wall_thickness : {:.3}", self.end_wall_thickness)
                }
            },
        };
        Ok(tooltip)
    }
}
