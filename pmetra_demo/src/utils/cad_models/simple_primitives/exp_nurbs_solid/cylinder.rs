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

use super::{CadShellIds, ExpNurbsSolid};

fn cylinder_translation(params: &ExpNurbsSolid) -> Vec3 {
    Vec3::new(
        params.cylinder_translation[0],
        params.cylinder_translation[1],
        params.cylinder_translation[2],
    )
}

pub fn build_cylinder_shell(params: &ExpNurbsSolid) -> Result<CadShell> {
    let ExpNurbsSolid {
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

    let wire = builder::rsweep(&v0, Point3::origin(), Vector3::unit_y(), Rad(TAU + 1.0), 2);
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
    params: &ExpNurbsSolid,
    shell_name: CadShellName,
) -> Result<CadMeshBuilder<ExpNurbsSolid>> {
    // spawn entity with generated mesh...
    let transform = Transform::from_translation(cylinder_translation(params));

    let mesh_builder = CadMeshBuilder::new(params.clone(), shell_name.clone())? // builder
        .set_transform(transform)?
        .set_base_material(Color::from(css::RED).into())?;

    Ok(mesh_builder)
}

pub fn build_radius_slider(
    params: &ExpNurbsSolid,
    cad_shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let ExpNurbsSolid {
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
    let mesh_transform = Transform::from_translation(cylinder_translation(params));
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

pub fn build_height_slider(
    params: &ExpNurbsSolid,
    cad_shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let ExpNurbsSolid {
        cylinder_height, ..
    } = params;

    let cad_shell = cad_shells_by_name
        .get(&CadShellName(CadShellIds::Cylinder.to_string()))
        .ok_or_else(|| anyhow!("Could not get cylinder shell!"))?;

    let Some(CadElement::Face(face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("ProfileFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };

    let face_boundaries = face.boundaries();
    let face_wire = face_boundaries.last().expect("No wire found!");
    let face_centroid = face_wire.get_centroid();
    let mesh_transform = Transform::from_translation(cylinder_translation(params));
    let slider_transform = Transform::from_translation(
        mesh_transform.translation + face_centroid.as_vec3() + Vec3::Y * (*cylinder_height as f32 + 0.1),
    );

    Ok(CadSlider {
        drag_plane_normal: Vec3::X,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: Vec3::Y,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}

pub fn build_move_slider(
    params: &ExpNurbsSolid,
    cad_shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let ExpNurbsSolid {
        cylinder_radius, ..
    } = params;

    let cad_shell = cad_shells_by_name
        .get(&CadShellName(CadShellIds::Cylinder.to_string()))
        .ok_or_else(|| anyhow!("Could not get cylinder shell!"))?;

    let Some(CadElement::Face(face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("ProfileFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };

    let face_boundaries = face.boundaries();
    let face_wire = face_boundaries.last().expect("No wire found!");
    let face_centroid = face_wire.get_centroid();
    let mesh_transform = Transform::from_translation(cylinder_translation(params));
    let slider_transform = Transform::from_translation(
        mesh_transform.translation
            + face_centroid.as_vec3()
            + Vec3::Z * (*cylinder_radius as f32 + 0.1),
    )
    .with_rotation(get_rotation_from_normals(Vec3::Z, Vec3::Y));

    Ok(CadSlider {
        drag_plane_normal: Vec3::Y,
        transform: slider_transform,
        slider_type: CadSliderType::Planer,
        ..default()
    })
}
