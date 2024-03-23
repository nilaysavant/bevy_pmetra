use std::f64::consts::{FRAC_PI_2, PI, TAU};

use anyhow::{anyhow, Context, Error, Result};
use bevy::{math::DVec3, prelude::*};
use bevy_pmetra::{
    cad_core::extensions::shell::ShellCadExtension, math::get_rotation_from_normals, prelude::*, re_exports::truck_modeling::{
        builder, cgmath::AbsDiffEq, control_point::ControlPoint, ParametricSurface3D, Point3, Rad, Shell, Vector3, Vertex, Wire
    }
};
use strum::{Display, EnumString};

use super::SimpleCubeAtCylinder;

pub fn build_cylinder_shell(builder: &CadShellsBuilder<SimpleCubeAtCylinder>) -> Result<CadShell> {
    let SimpleCubeAtCylinder {
        cylinder_radius,
        cylinder_height,
        cube_attach_angle,
        cube_side_length,
    } = builder.params.clone();

    let mut tagged_elements = CadTaggedElements::default();

    let v0 = Vertex::new((DVec3::X * cylinder_radius).to_array().into());
    tagged_elements.insert(
        CadElementTag("VertexV0".into()),
        CadElement::Vertex(v0.clone()),
    );

    let wire = builder::rsweep(&v0, Point3::origin(), Vector3::unit_y(), Rad(TAU + 1.0));
    let face =
        builder::try_attach_plane(&[wire]).with_context(|| "Could not attach plane to wire")?;
    tagged_elements.insert(
        CadElementTag("ProfileFace".into()),
        CadElement::Face(face.clone()),
    );
    let solid = builder::tsweep(&face, (DVec3::Y * cylinder_height).to_array().into());

    let shell = Shell::try_from_solid(&solid)?;

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

pub fn build_cylinder_mesh(
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
    let transform = Transform::default();
    let cad_mesh = CadMeshBuilder::new(builder.params.clone(), cad_shell.clone())? // builder
        .set_bevy_mesh(mesh)?
        .set_base_material(Color::RED.into())?
        .set_outlines(cad_shell.shell.build_outlines())?
        .set_transform(transform)?
        .add_cursor(
            CylinderCursorIds::RadiusCursor.to_string(),
            build_radius_cursor,
        )?
        .build()?;

    Ok(cad_mesh)
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum CylinderCursorIds {
    RadiusCursor,
}

pub fn build_radius_cursor(
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
    let Some(CadElement::Face(face)) = cad_shell.get_element_by_tag(CadElementTag::new("ProfileFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };
    let face_normal = face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();
    let face_boundaries = face.boundaries();
    let face_wire = face_boundaries.last().expect("No wire found!");
    let face_centroid = face_wire.get_centroid();
    let right_direction = (vertex_v0.point().as_bevy_vec3() - face_centroid.as_vec3()).normalize();
    let mesh_transform = Transform::default();
    let cursor_transform = Transform::from_translation(
        mesh_transform.translation
            + face_centroid.as_vec3()
            + right_direction * (*cylinder_radius as f32 + 0.1),
    )
    .with_rotation(get_rotation_from_normals(Vec3::Z, face_normal));

    Ok(CadCursor {
        normal: face_normal,
        transform: cursor_transform,
        cursor_type: CadCursorType::Linear {
            direction: right_direction,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}
