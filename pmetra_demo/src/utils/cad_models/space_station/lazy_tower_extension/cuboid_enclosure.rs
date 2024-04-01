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
/// ad = bc = enclosure_profile_width
/// ab = dc = enclosure_profile_depth
/// o : Located at origin(0,0,0)
///
///  a --------d
///  |         |           
///  |    o    | enclosure_profile_depth
///  |         |
///  b --------c
///     enclosure_profile_width
/// ```
///
///
///
///
pub fn build_cuboid_enclosure_shell(params: &LazyTowerExtension) -> Result<CadShell> {
    let LazyTowerExtension {
        tower_length,
        enclosure_profile_width,
        enclosure_profile_depth,
        ..
    } = params.clone();

    let mut tagged_elements = CadTaggedElements::default();

    // Create the L-Shaped Cross section of beam...

    // Create points...
    let o = DVec3::ZERO;
    let a = DVec3::new(
        -enclosure_profile_width / 2.,
        0.,
        -enclosure_profile_depth / 2.,
    );
    let b = a + DVec3::new(0., 0., enclosure_profile_depth);
    let c = b + DVec3::new(enclosure_profile_width, 0., 0.);
    let d = c + DVec3::new(0., 0., -enclosure_profile_depth);

    // Create wire...
    let points = [a, b, c, d];
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

pub fn cuboid_enclosure_mesh_builder(
    params: &LazyTowerExtension,
    shell_name: CadShellName,
) -> Result<CadMeshLazyBuilder<LazyTowerExtension>> {
    let LazyTowerExtension {
        tower_length,
        enclosure_profile_depth,
        enclosure_profile_width,
        ..
    } = &params;
    // spawn entity with generated mesh...
    let transform = Transform::default();

    let mesh_builder = CadMeshLazyBuilder::new(params.clone(), shell_name.clone())? // builder
        .set_transform(transform)?
        .set_base_material(Color::WHITE.with_a(0.2).into())?;

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
