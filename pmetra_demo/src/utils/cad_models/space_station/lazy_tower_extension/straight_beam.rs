use std::f64::consts::{FRAC_PI_2, PI, TAU};

use anyhow::{anyhow, Context, Error, Result};
use bevy::{math::DVec3, prelude::*};
use bevy_pmetra::{
    cad_core::{
        extensions::shell::ShellCadExtension,
        lazy_builders::{CadMeshLazyBuilder, CadMeshesLazyBuilder, CadShellName, CadShellsByName},
    },
    math::get_rotation_from_normals,
    prelude::*,
    re_exports::{
        truck_modeling::{
            builder, control_point::ControlPoint, Curve, Edge, ParametricSurface3D, Point3, Rad,
            Shell, Vector3, Vertex, Wire,
        },
        truck_topology::{VertexDisplayFormat, WireDisplayFormat},
    },
};
use itertools::Itertools;
use strum::{Display, EnumString};

use super::{common::l_beam_shell, CadShellIds, LazyTowerExtension};

/// Straight Beam Shell Builder.
pub fn build_straight_beam_shell(params: &LazyTowerExtension) -> Result<CadShell> {
    let LazyTowerExtension {
        tower_length,
        straight_beam_l_sect_side_len,
        straight_beam_l_sect_thickness,
        ..
    } = params.clone();

    let mut tagged_elements = CadTaggedElements::default();

    let shell = l_beam_shell(
        straight_beam_l_sect_side_len,
        straight_beam_l_sect_thickness,
        tower_length,
    )?;

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

pub fn straight_beam_mesh_builder(
    params: &LazyTowerExtension,
    shell_name: CadShellName,
    transform: Transform,
) -> Result<CadMeshLazyBuilder<LazyTowerExtension>> {
    let LazyTowerExtension {
        tower_length,
        straight_beam_l_sect_side_len,
        straight_beam_l_sect_thickness,
        ..
    } = &params;
    // spawn entity with generated mesh...

    let mesh_builder = CadMeshLazyBuilder::new(params.clone(), shell_name.clone())? // builder
        .set_transform(transform)?
        .set_base_material(Color::RED.into())?;

    Ok(mesh_builder)
}

// pub fn build_radius_cursor(
//     params: &SimpleLazyCubeAtCylinder,
//     cad_shells_by_name: &CadShellsByName,
// ) -> Result<CadCursor> {
//     let SimpleLazyCubeAtCylinder {
//         cylinder_radius,
//         cylinder_height,
//         cube_attach_angle,
//         cube_side_length,
//     } = params;

//     let cad_shell = cad_shells_by_name
//         .get(&CadShellName(CadShellIds::Cylinder.to_string()))
//         .ok_or_else(|| anyhow!("Could not get cylinder shell!"))?;

//     let Some(CadElement::Vertex(vertex_v0)) =
//         cad_shell.get_element_by_tag(CadElementTag::new("VertexV0"))
//     else {
//         return Err(anyhow!("Could not find vertex!"));
//     };
//     let Some(CadElement::Face(face)) =
//         cad_shell.get_element_by_tag(CadElementTag::new("ProfileFace"))
//     else {
//         return Err(anyhow!("Could not find face!"));
//     };
//     let face_normal = face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();
//     let face_boundaries = face.boundaries();
//     let face_wire = face_boundaries.last().expect("No wire found!");
//     let face_centroid = face_wire.get_centroid();
//     let right_direction = (vertex_v0.point().as_bevy_vec3() - face_centroid.as_vec3()).normalize();
//     let mesh_transform = Transform::default();
//     let cursor_transform = Transform::from_translation(
//         mesh_transform.translation
//             + face_centroid.as_vec3()
//             + right_direction * (*cylinder_radius as f32 + 0.1),
//     )
//     .with_rotation(get_rotation_from_normals(Vec3::Z, face_normal));

//     Ok(CadCursor {
//         normal: face_normal,
//         transform: cursor_transform,
//         cursor_type: CadCursorType::Linear {
//             direction: right_direction,
//             limit_min: None,
//             limit_max: None,
//         },
//         ..default()
//     })
// }

mod tests {
    use super::*;

    #[test]
    pub fn test_shell_builder() {
        let params = LazyTowerExtension {
            tower_length: 1.0,
            straight_beam_l_sect_side_len: 0.25,
            straight_beam_l_sect_thickness: 0.05,
            ..Default::default()
        };

        let shell = build_straight_beam_shell(&params).unwrap();
    }
}
