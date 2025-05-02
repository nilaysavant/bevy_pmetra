use std::f64::consts::TAU;

use bevy::{color::palettes::css, math::DVec3, prelude::*};
use bevy_pmetra::{
    math::get_rotation_from_normals,
    pmetra_core::extensions::shell::ShellCadExtension,
    prelude::*,
    re_exports::{
        anyhow::{anyhow, Context, Result},
        truck_modeling::{
            builder, control_point::ControlPoint, ParametricSurface3D, Point3, Rad, Shell, Vector3,
            Vertex,
        },
    },
};

use super::{CadShellIds, SimpleCubeAtCylinder};

pub fn build_cylinder_shell(params: &SimpleCubeAtCylinder) -> Result<CadShell> {
    let SimpleCubeAtCylinder {
        cylinder_radius,
        cylinder_height,
        ..
    } = params.clone();

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

pub fn cylinder_mesh_builder(
    params: &SimpleCubeAtCylinder,
    shell_name: CadShellName,
) -> Result<CadMeshBuilder<SimpleCubeAtCylinder>> {
    // spawn entity with generated mesh...
    let transform = Transform::default();

    let mesh_builder = CadMeshBuilder::new(params.clone(), shell_name.clone())? // builder
        .set_transform(transform)?
        .set_base_material(Color::from(css::RED).into())?;

    Ok(mesh_builder)
}

pub fn build_radius_slider(
    params: &SimpleCubeAtCylinder,
    cad_shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let SimpleCubeAtCylinder {
        cylinder_radius, ..
    } = params;

    let cad_shell = cad_shells_by_name
        .get(&CadShellName(CadShellIds::Cylinder.to_string()))
        .ok_or_else(|| anyhow!("Could not get cylinder shell!"))?;

    let Some(CadElement::Vertex(vertex_v0)) =
        cad_shell.get_element_by_tag(CadElementTag::new("VertexV0"))
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
    let right_direction = (vertex_v0.point().as_bevy_vec3() - face_centroid.as_vec3()).normalize();
    let mesh_transform = Transform::default();
    let slider_transform = Transform::from_translation(
        mesh_transform.translation
            + face_centroid.as_vec3()
            + right_direction * (*cylinder_radius as f32 + 0.1),
    )
    .with_rotation(get_rotation_from_normals(Vec3::Z, face_normal));

    Ok(CadSlider {
        drag_plane_normal: face_normal,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: right_direction,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}
