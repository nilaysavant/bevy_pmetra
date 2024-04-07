use std::{f64::consts::PI, str::FromStr};

use anyhow::Result;
use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::{
    cad_core::lazy_builders::{
        CadMeshesLazyBuildersByCadShell, CadShellName, CadShellsByName, CadShellsLazyBuilders,
        ParametricLazyCad, ParametricLazyModelling,
    },
    prelude::*,
};
use bevy_rapier3d::na::ComplexField;
use strum::{Display, EnumString};

use self::gear::{
    build_face_width_cursor, build_main_gear_mesh, build_main_gear_shell, build_radius_cursor,
};

/// Gear CAD Model.
pub mod gear;
/// Math primitives used in gear construction.
pub mod math;

/// Basic Parametric Station Segment.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct LazySimpleGear {
    pub num_of_teeth: u32,
    pub pitch_circle_diameter: f64,
    pub face_width: f64,
}

impl Default for LazySimpleGear {
    fn default() -> Self {
        Self {
            num_of_teeth: 20,
            pitch_circle_diameter: 0.1,
            face_width: 0.02,
        }
    }
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadSolidIds {
    MainGear,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadMeshIds {
    MainGear,
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CadCursorIds {
    PitchCircleDiaCursor,
    FaceWidthCursor,
    NumTeethCursor,
}

impl ParametricLazyModelling for LazySimpleGear {
    fn shells_builders(&self) -> Result<CadShellsLazyBuilders<Self>> {
        let mut builders = CadShellsLazyBuilders::new(self.clone())?.add_shell_builder(
            CadShellName(CadSolidIds::MainGear.to_string()),
            build_main_gear_shell,
        )?;

        Ok(builders)
    }
}

impl ParametricLazyCad for LazySimpleGear {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesLazyBuildersByCadShell<Self>> {
        CadMeshesLazyBuildersByCadShell::new(self.clone(), shells_by_name.clone())?
            .add_mesh_builder(
                CadShellName(CadSolidIds::MainGear.to_string()),
                CadMeshIds::MainGear.to_string(),
                build_main_gear_mesh(
                    self,
                    CadShellName(CadSolidIds::MainGear.to_string()),
                    shells_by_name,
                )?,
            )
    }

    fn cursors(&self, shells_by_name: &CadShellsByName) -> Result<CadCursors> {
        let cursors = CadCursors::default()
            .add_cursor(
                CadCursorIds::PitchCircleDiaCursor.to_string().into(),
                build_radius_cursor(self, shells_by_name)?,
            )?
            .add_cursor(
                CadCursorIds::FaceWidthCursor.to_string().into(),
                build_face_width_cursor(self, shells_by_name)?,
            )?
            .add_cursor(
                CadCursorIds::NumTeethCursor.to_string().into(),
                build_face_width_cursor(self, shells_by_name)?,
            )?;

        Ok(cursors)
    }

    fn on_cursor_transform(
        &mut self,
        cursor_name: CadCursorName,
        prev_transform: Transform,
        new_transform: Transform,
    ) {
        match CadCursorIds::from_str(&cursor_name).unwrap() {
            CadCursorIds::PitchCircleDiaCursor => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.;
                    let new_value = self.pitch_circle_diameter + delta.x as f64 * sensitivity;
                    self.pitch_circle_diameter = new_value.clamp(0.002, std::f64::MAX);
                }
            }
            CadCursorIds::FaceWidthCursor => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 1.;
                    let new_value = self.face_width + delta.y as f64 * sensitivity;
                    self.face_width = new_value.clamp(0.01, std::f64::MAX);
                }
            }
            CadCursorIds::NumTeethCursor => {
                let delta = new_transform.translation - prev_transform.translation;
                if delta.length() > 0. {
                    let sensitivity = 0.1;
                    let new_value = (self.num_of_teeth as i32
                        + ((delta.z.abs() * sensitivity).ceil() * delta.z.signum()) as i32)
                        as u32;
                    self.num_of_teeth = new_value.clamp(2, std::u32::MAX);
                }
            }
        }
    }

    fn on_cursor_tooltip(&self, cursor_name: CadCursorName) -> Result<Option<String>> {
        let tooltip = match CadCursorIds::from_str(&cursor_name).unwrap() {
            CadCursorIds::PitchCircleDiaCursor => Some(format!(
                "pitch_circle_diameter : {:.3}",
                self.pitch_circle_diameter
            )),
            CadCursorIds::FaceWidthCursor => Some(format!("face_width : {:.3}", self.face_width)),
            CadCursorIds::NumTeethCursor => {
                Some(format!("num_of_teeth : {:.3}", self.num_of_teeth))
            }
        };

        Ok(tooltip)
    }
}
