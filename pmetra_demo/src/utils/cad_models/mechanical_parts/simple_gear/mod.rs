use std::{f64::consts::PI, str::FromStr};

use anyhow::Result;
use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::prelude::*;
use bevy_rapier3d::na::ComplexField;
use strum::{Display, EnumString};

use self::gear::{build_main_gear_mesh, build_main_gear_shell, GearCursorIds};

/// Gear CAD Model.
pub mod gear;
/// Math primitives used in gear construction.
pub mod math;

/// Basic Parametric Station Segment.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct SimpleGear {
    pub num_of_teeth: u32,
    pub pitch_circle_diameter: f64,
    pub face_width: f64,
}

impl Default for SimpleGear {
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

impl ParametricModelling for SimpleGear {
    fn build_shells(&self) -> Result<CadShells> {
        let cad_shells =
            CadShellsBuilder::new(self.clone())? // builder
                .add_shell(CadSolidIds::MainGear.to_string(), build_main_gear_shell)?
                .build()?;

        Ok(cad_shells)
    }
}

impl ParametricCad for SimpleGear {
    fn build_cad_meshes_from_shells(
        &self,
        solids: CadShells,
    ) -> Result<CadMeshes> {
        CadMeshesBuilder::new(self.clone(), solids)? // builder
            .add_mesh(
                CadSolidIds::MainGear.to_string(),
                CadMeshIds::MainGear.to_string(),
                build_main_gear_mesh,
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
            CadMeshIds::MainGear => match GearCursorIds::from_str(&cursor_name).unwrap() {
                GearCursorIds::PitchCircleDiaCursor => {
                    let delta = new_transform.translation - prev_transform.translation;
                    if delta.length() > 0. {
                        let sensitivity = 1.;
                        let new_value = self.pitch_circle_diameter + delta.x as f64 * sensitivity;
                        self.pitch_circle_diameter = new_value.clamp(0.002, std::f64::MAX);
                    }
                }
                GearCursorIds::FaceWidthCursor => {
                    let delta = new_transform.translation - prev_transform.translation;
                    if delta.length() > 0. {
                        let sensitivity = 1.;
                        let new_value = self.face_width + delta.y as f64 * sensitivity;
                        self.face_width = new_value.clamp(0.01, std::f64::MAX);
                    }
                }
                GearCursorIds::NumTeethCursor => {
                    let delta = new_transform.translation - prev_transform.translation;
                    if delta.length() > 0. {
                        let sensitivity = 0.1;
                        let new_value = (self.num_of_teeth as i32
                            + ((delta.z.abs() * sensitivity).ceil() * delta.z.signum()) as i32)
                            as u32;
                        self.num_of_teeth = new_value.clamp(2, std::u32::MAX);
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
            CadMeshIds::MainGear => match GearCursorIds::from_str(&cursor_name).unwrap() {
                GearCursorIds::PitchCircleDiaCursor => {
                    format!("pitch_circle_diameter : {:.3}", self.pitch_circle_diameter)
                }
                GearCursorIds::FaceWidthCursor => {
                    format!("face_width : {:.3}", self.face_width)
                }
                GearCursorIds::NumTeethCursor => {
                    format!("num_of_teeth : {:.3}", self.num_of_teeth)
                }
            },
        };
        Ok(tooltip)
    }
}
