use std::str::FromStr;

use anyhow::{Ok, Result};
use bevy::{math::DVec3, prelude::*, transform};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::{
    cad_core::lazy_builders::{
        CadMeshesLazyBuildersByCadShell, CadShellName, CadShellsByName, CadShellsLazyBuilders,
        ParametricLazyCad, ParametricLazyModelling,
    },
    prelude::*,
};
use strum::{Display, EnumString};

use self::cabin::{build_cabin_mesh, build_cabin_shell, build_extrude_cursor};

use super::RoundRectCuboid;

pub mod cabin;
pub mod end_walls;

/// Basic Parametric Station Segment.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct LazyRoundCabinSegment {
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

impl Default for LazyRoundCabinSegment {
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

impl ParametricLazyModelling for LazyRoundCabinSegment {
    fn shells_builders(&self) -> Result<CadShellsLazyBuilders<Self>> {
        let builders = CadShellsLazyBuilders::new(self.clone())? // builder
            .add_shell_builder(
                CadShellName(CadShellIds::CabinShell.to_string()),
                build_cabin_shell,
            )?;

        Ok(builders)
    }
}

impl ParametricLazyCad for LazyRoundCabinSegment {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesLazyBuildersByCadShell<Self>> {
        let cad_meshes_lazy_builders_by_cad_shell =
            CadMeshesLazyBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
                .add_mesh_builder(
                    CadShellName(CadShellIds::CabinShell.to_string()),
                    CadMeshIds::CabinShell.to_string(),
                    build_cabin_mesh(self, CadShellName(CadShellIds::CabinShell.to_string()))?,
                )?;

        Ok(cad_meshes_lazy_builders_by_cad_shell)
    }

    fn cursors(&self, shells_by_name: &CadShellsByName) -> Result<CadCursors> {
        let cursors = CadCursors::default() // builder
        .add_cursor(
            CadCursorIds::ExtrudeCursor.to_string().into(),
            build_extrude_cursor(self, shells_by_name)?,
        )? // todo
        ;

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
            _ => {}
        }
    }

    fn on_cursor_tooltip(&self, cursor_name: CadCursorName) -> Result<Option<String>> {
        let tooltip = match CadCursorIds::from_str(&cursor_name).unwrap() {
            CadCursorIds::ExtrudeCursor => Some(format!(
                "profile_extrude_length : {:.3}",
                self.profile_extrude_length
            )),
            // CadCursorIds::CornerRadiusCursor => {
            //     format!("profile_corner_radius : {:.3}", self.profile_corner_radius)
            // }
            // CadCursorIds::ProfileThicknessCursor => {
            //     format!("profile_thickness : {:.3}", self.profile_thickness)
            // }
            // CadCursorIds::ProfileHeightCursor => {
            //     format!("profile_height : {:.3}", self.profile_height)
            // }
            // CadCursorIds::ProfileWidthCursor => {
            //     format!("profile_width : {:.3}", self.profile_width)
            // }
            // CadCursorIds::WindowTranslationCursor => {
            //     format!(
            //         "window_translation : [{:.3}, {:.3}, {:.3}]",
            //         self.window_translation.x, self.window_translation.y, self.window_translation.z
            //     )
            // }
            _ => None,
        };

        Ok(tooltip)
    }
}
