use std::f64::consts::{FRAC_PI_2, PI, TAU};

use anyhow::{anyhow, Context, Error, Result};
use bevy::{math::DVec3, prelude::*};
use bevy_pmetra::{
    cad_core::extensions::shell::ShellCadExtension,
    math::get_rotation_from_normals,
    prelude::*,
    re_exports::truck_modeling::{
        builder, cgmath::AbsDiffEq, control_point::ControlPoint, Homogeneous, ParametricSurface3D,
        Point3, Rad, Shell, Vector3, Vertex, Wire,
    },
};
use strum::{Display, EnumString};

use super::{CadSolidIds, SimpleCubeAtCylinder};

pub fn build_cube_shell(builder: &CadShellsBuilder<SimpleCubeAtCylinder>) -> Result<CadShell> {
    let SimpleCubeAtCylinder {
        cylinder_radius,
        cylinder_height,
        cube_attach_angle,
        cube_side_length,
    } = builder.params.clone();

    let mut tagged_elements = CadTaggedElements::default();

    let v0 = Vertex::new(
        (DVec3::new(-cube_side_length / 2., 0., cube_side_length / 2.))
            .to_array()
            .into(),
    );
    let v1 = Vertex::new(
        (DVec3::new(cube_side_length / 2., 0., cube_side_length / 2.))
            .to_array()
            .into(),
    );
    tagged_elements.insert(
        CadElementTag("VertexV0".into()),
        CadElement::Vertex(v0.clone()),
    );
    tagged_elements.insert(
        CadElementTag("VertexV1".into()),
        CadElement::Vertex(v1.clone()),
    );

    let edge = builder::tsweep(&v0, v1.point().to_vec() - v0.point().to_vec());
    let face = builder::tsweep(&edge, -Vector3::unit_z() * cube_side_length);
    tagged_elements.insert(
        CadElementTag("ProfileFace".into()),
        CadElement::Face(face.clone()),
    );
    let solid = builder::tsweep(&face, (DVec3::Y * cube_side_length).to_array().into());

    let shell = Shell::try_from_solid(&solid)?;

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

pub fn build_cube_mesh(
    builder: &CadMeshesBuilder<SimpleCubeAtCylinder>,
    cad_shell: &CadShell,
) -> Result<CadMesh> {
    let SimpleCubeAtCylinder {
        cylinder_radius,
        cylinder_height,
        cube_attach_angle,
        cube_side_length,
    } = &builder.params;
    // convert result into mesh...
    let mesh = cad_shell.build_polygon()?.build_mesh();
    // spawn entity with generated mesh...

    let Some(cylinder_solid) = builder.shells.get(&CadSolidIds::Cylinder.to_string()) else {
        return Err(anyhow!("Could not get cylinder_solid!"));
    };
    let Some(CadElement::Vertex(cylinder_v0)) = cylinder_solid
        .tagged_elements
        .get(&CadElementTag("VertexV0".into()))
    else {
        return Err(anyhow!("Could not get cylinder VertexV0!"));
    };

    let rotation = get_rotation_from_normals(Vec3::Y, Vec3::X);
    let mut transform = Transform::from_rotation(rotation).with_translation(
        cylinder_v0.point().as_bevy_vec3() + Vec3::Y * (*cylinder_height as f32 / 2.),
    );
    transform.rotate_around(
        Vec3::ZERO,
        Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4),
    );

    let cad_mesh = CadMeshBuilder::new(builder.params.clone(), cad_shell.clone())? // builder
        .set_bevy_mesh(mesh)?
        .set_base_material(Color::BLUE.into())?
        .set_outlines(cad_shell.shell.build_outlines())?
        .set_transform(transform)?
        .add_cursor(
            CubeCursorIds::SideLength.to_string(),
            build_side_length_cursor,
        )?
        .build()?;

    Ok(cad_mesh)
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CubeCursorIds {
    SideLength,
}

pub fn build_side_length_cursor(
    builder: &CadMeshBuilder<SimpleCubeAtCylinder>,
    cad_shell: &CadShell,
) -> Result<CadCursor> {
    let SimpleCubeAtCylinder {
        cylinder_radius,
        cylinder_height,
        cube_attach_angle,
        cube_side_length,
    } = &builder.params;

    let Some(CadElement::Vertex(vertex_v0)) =
        cad_shell.get_element_by_tag(CadElementTag::new("VertexV0"))
    else {
        return Err(anyhow!("Could not find vertex!"));
    };
    let Some(CadElement::Vertex(vertex_v1)) =
        cad_shell.get_element_by_tag(CadElementTag::new("VertexV1"))
    else {
        return Err(anyhow!("Could not find vertex!"));
    };
    let Some(CadElement::Face(face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("ProfileFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };
    let face_normal = face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();
    let face_boundaries = face.boundaries();
    let face_wire = face_boundaries.last().expect("No wire found!");
    let face_centroid = face_wire.get_centroid();
    let local_right_direction =
        (vertex_v1.point().as_bevy_vec3() - vertex_v0.point().as_bevy_vec3()).normalize();
    let Some(CadMesh {
        transform: mesh_transform,
        ..
    }) = builder.cad_mesh
    else {
        return Err(anyhow!("could not get CadMesh!"));
    };
    let local_cursor_pos =
        face_centroid.as_vec3() + Vec3::Z * (*cube_side_length as f32 / 2. + 0.1);
    let cursor_pos = mesh_transform.transform_point(local_cursor_pos);
    let mut cursor_transform =
        Transform::from_translation(cursor_pos).with_rotation(mesh_transform.rotation);
    cursor_transform.rotate_y(std::f32::consts::FRAC_PI_2);

    Ok(CadCursor {
        normal: mesh_transform.up(),
        transform: cursor_transform,
        cursor_type: CadCursorType::Linear {
            direction: mesh_transform.local_z(),
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}
