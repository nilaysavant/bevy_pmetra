use std::str::FromStr;

use anyhow::{Ok, Result};
use bevy::{math::DVec3, prelude::*, transform};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::prelude::*;
use strum::{Display, EnumString};

use self::cabin::{
    build_cabin_mesh, build_cabin_shell, build_corner_radius_cursor, build_extrude_cursor,
    build_profile_height_cursor, build_profile_thickness_cursor, build_profile_width_cursor,
    build_window_translation_cursor,
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
pub enum CadCursorIds {
    ExtrudeCursor,
    CornerRadiusCursor,
    ProfileThicknessCursor,
    ProfileHeightCursor,
    ProfileWidthCursor,
    WindowTranslationCursor,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadMaterialIds {
    Base,
    Roof,
}

impl ParametricModelling for RoundCabinSegment {
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
        let builders = CadShellsBuilders::new(self.clone())? // builder
            .add_shell_builder(
                CadShellName(CadShellIds::CabinShell.to_string()),
                build_cabin_shell,
            )?;

        Ok(builders)
    }
}

impl ParametricCad for RoundCabinSegment {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesBuildersByCadShell<Self>> {
        let cad_meshes_builders_by_cad_shell =
            CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                .add_mesh_builder(
                    CadShellName(CadShellIds::CabinShell.to_string()),
                    CadMeshIds::CabinShell.to_string(),
                    build_cabin_mesh(self, CadShellName(CadShellIds::CabinShell.to_string()))?,
                )?;

        Ok(cad_meshes_builders_by_cad_shell)
    }

    fn cursors(&self, shells_by_name: &CadShellsByName) -> Result<CadCursors> {
        let cursors = CadCursors::default() // builder
            .add_cursor(
                CadCursorIds::ExtrudeCursor.to_string().into(),
                build_extrude_cursor(self, shells_by_name)?,
            )?
            .add_cursor(
                CadCursorIds::CornerRadiusCursor.to_string().into(),
                build_corner_radius_cursor(self, shells_by_name)?,
            )?
            .add_cursor(
                CadCursorIds::ProfileWidthCursor.to_string().into(),
                build_profile_width_cursor(self, shells_by_name)?,
            )?
            .add_cursor(
                CadCursorIds::ProfileHeightCursor.to_string().into(),
                build_profile_height_cursor(self, shells_by_name)?,
            )?
            .add_cursor(
                CadCursorIds::ProfileThicknessCursor.to_string().into(),
                build_profile_thickness_cursor(self, shells_by_name)?,
            )?
            .add_cursor(
                CadCursorIds::WindowTranslationCursor.to_string().into(),
                build_window_translation_cursor(self, shells_by_name)?,
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
            CadCursorIds::ExtrudeCursor => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.0;
                    let new_value = self.profile_extrude_length + delta.z as f64 * sensitivity;
                    self.profile_extrude_length = new_value.clamp(0.001, std::f64::MAX);
                }
            }
            CadCursorIds::CornerRadiusCursor => {
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
            CadCursorIds::ProfileWidthCursor => {
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
            CadCursorIds::ProfileThicknessCursor => {
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
            CadCursorIds::ProfileHeightCursor => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.0;
                    let new_value = self.profile_height + delta.y as f64 * sensitivity;
                    self.profile_height =
                        new_value.clamp(self.profile_corner_radius * 2. + 0.01, std::f64::MAX);
                }
            }
            CadCursorIds::WindowTranslationCursor => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.0;
                    self.window_translation += delta.as_dvec3() * sensitivity;
                }
            }
            _ => {}
        }
    }

    fn on_cursor_tooltip(&self, cursor_name: CadCursorName) -> Result<Option<String>> {
        let tooltip = match CadCursorIds::from_str(&cursor_name).unwrap() {
            CadCursorIds::ExtrudeCursor => Some(format!(
                "profile_extrude_length : {:.3}",
                self.profile_extrude_length
            )),
            CadCursorIds::CornerRadiusCursor => Some(format!(
                "profile_corner_radius : {:.3}",
                self.profile_corner_radius
            )),
            CadCursorIds::ProfileWidthCursor => {
                Some(format!("profile_width : {:.3}", self.profile_width))
            }
            CadCursorIds::ProfileThicknessCursor => {
                Some(format!("profile_thickness : {:.3}", self.profile_thickness))
            }
            CadCursorIds::ProfileHeightCursor => {
                Some(format!("profile_height : {:.3}", self.profile_height))
            }
            CadCursorIds::WindowTranslationCursor => Some(format!(
                "window_translation : [{:.3}, {:.3}, {:.3}]",
                self.window_translation.x, self.window_translation.y, self.window_translation.z
            )),
            _ => None,
        };

        Ok(tooltip)
    }
}
