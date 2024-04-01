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
    re_exports::truck_modeling::{
        builder, control_point::ControlPoint, Curve, Edge, ParametricSurface3D, Point3, Rad, Shell,
        Vector3, Vertex, Wire,
    },
};
use itertools::Itertools;
use strum::{Display, EnumString};

use super::{CadShellIds, LazyTowerExtension};

/// Straight Beam Shell Builder.
///
/// The beam has a L-shaped cross section.
///
/// ```
/// O -> x
/// |
/// z
///
/// ao = ob = straight_beam_l_sect_side_len
/// bc = ae = straight_beam_l_sect_thickness
/// o : Located at origin(0,0)
///
///  o ---------- b  ^
///  | *--------- c  V
///  | | d
///  | |
///  | |
///  a e
///
/// ```
///
///
///
///
pub fn build_straight_beam_shell(params: &LazyTowerExtension) -> Result<CadShell> {
    let LazyTowerExtension {
        tower_length,
        straight_beam_l_sect_side_len,
        straight_beam_l_sect_thickness,
    } = params.clone();

    let mut tagged_elements = CadTaggedElements::default();

    // Create the L-Shaped Cross section of beam...

    // Create points...
    let o = DVec3::ZERO;
    let b = DVec3::new(straight_beam_l_sect_side_len, 0., 0.);
    let c = b + DVec3::new(0., 0., straight_beam_l_sect_thickness);
    let d = DVec3::new(
        straight_beam_l_sect_thickness,
        0.,
        straight_beam_l_sect_thickness,
    );
    let a = DVec3::new(0., 0., straight_beam_l_sect_thickness);
    let e = a + DVec3::new(straight_beam_l_sect_thickness, 0., 0.);

    // Create wire...
    let points = [o, b, c, d, e, a, o];
    let vertices = points
        .iter()
        .map(|p| Vertex::new(Point3::from(p.to_array())))
        .collect::<Vec<_>>();
    let mut wire = Wire::new();
    for (v0, v1) in vertices.iter().circular_tuple_windows() {
        let edge = builder::line(v0, v1);
        wire.push_back(edge);
    }
    // Checks for wire...
    debug_assert!(wire.is_closed());
    debug_assert!(wire.vertex_iter().count() == 7);

    // Extrude wire and create shell...
    let face =
        builder::try_attach_plane(&[wire]).with_context(|| "Could not attach plane to wire")?;
    let solid = builder::tsweep(&face, Vector3::from((DVec3::Y * tower_length).to_array()));
    let shell = Shell::try_from_solid(&solid)?;

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

pub fn straight_beam_mesh_builder(
    params: &LazyTowerExtension,
    shell_name: CadShellName,
) -> Result<CadMeshLazyBuilder<LazyTowerExtension>> {
    let LazyTowerExtension {
        tower_length,
        straight_beam_l_sect_side_len,
        straight_beam_l_sect_thickness,
    } = &params;
    // spawn entity with generated mesh...
    let transform = Transform::default();

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
            tower_length: 10.,
            straight_beam_l_sect_side_len: 1.,
            straight_beam_l_sect_thickness: 0.1,
        };

        let shell = build_straight_beam_shell(&params).unwrap();
    }
}
