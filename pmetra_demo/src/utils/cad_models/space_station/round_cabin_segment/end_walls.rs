use std::f64::consts::FRAC_PI_2;

use anyhow::{anyhow, Context, Error, Result};
use bevy::{math::DVec3, prelude::*};
use bevy_pmetra::{
    cad_core::extensions::shell::ShellCadExtension,
    math::get_rotation_from_normals,
    prelude::*,
    re_exports::truck_modeling::{
        builder, cgmath::AbsDiffEq, ParametricSurface3D, Point3, Rad, Shell, Vector3, Wire,
    },
};
use strum::{Display, EnumString};

use crate::utils::cad_models::space_station::{
    common::ref_edge_direction_for_wire, RoundRectCuboid,
};

use super::RoundCabinSegment;

pub fn build_end_wall_shell(builder: &CadShellsBuilder<RoundCabinSegment>) -> Result<CadShell> {
    let RoundCabinSegment {
        profile_width,
        profile_height,
        profile_corner_radius,
        profile_thickness,
        profile_extrude_length,
        end_wall_thickness,
        window,
        window_translation,
        ..
    } = builder.params.clone();

    let mut tagged_elements = CadTaggedElements::default();

    let round_rect_cuboid = RoundRectCuboid {
        profile_width: profile_width - profile_thickness * 2.,
        profile_height: profile_height - profile_thickness * 2.,
        profile_corner_radius,
        profile_extrude_length: end_wall_thickness,
    };

    let shell = round_rect_cuboid
        .try_build()
        .with_context(|| "Could not build RoundRectCuboid")?;

    let profile_face = shell
        .face_iter()
        .find(|face| {
            let face_vertex_z = face.vertex_iter().last().unwrap().point().z;
            let is_z_same = face.vertex_iter().all(|v| {
                v.point()
                    .z
                    .abs_diff_eq(&face_vertex_z, Point3::default_epsilon())
            });
            is_z_same && face_vertex_z.abs_diff_eq(&0., Point3::default_epsilon())
        })
        .ok_or_else(|| anyhow!("Could not find same z face!"))?;
    tagged_elements.insert(
        CadElementTag("ProfileFace".into()),
        CadElement::Face(profile_face.clone()),
    );

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

pub fn build_end_wall_mesh(
    builder: &CadMeshesBuilder<RoundCabinSegment>,
    cad_shell: &CadShell,
    textures: &CadMaterialTextures<Option<Image>>,
) -> Result<CadMesh> {
    let RoundCabinSegment {
        profile_width,
        profile_height,
        profile_corner_radius,
        profile_thickness,
        profile_extrude_length,
        end_wall_thickness,
        window,
        window_translation,
    } = &builder.params;
    // convert result into mesh...
    let main_mesh = cad_shell.build_polygon()?.build_mesh();
    // spawn entity with generated mesh...
    let main_mesh_transform = Transform::from_translation(Vec3::new(
        *profile_thickness as f32,
        *profile_thickness as f32,
        (profile_extrude_length - end_wall_thickness) as f32,
    ));
    let cad_mesh = CadMeshBuilder::new(builder.params.clone(), cad_shell.clone())? // builder
        .set_bevy_mesh(main_mesh)?
        .set_base_material(StandardMaterial {
            base_color: Color::WHITE.with_a(0.2),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })?
        .set_outlines(cad_shell.shell.build_outlines())?
        .set_transform(main_mesh_transform)?
        .add_cursor(
            EndWallCursorIds::ExtrudeCursor.to_string(),
            build_extrude_cursor,
        )?
        .build()?;

    Ok(cad_mesh)
}

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum EndWallCursorIds {
    ExtrudeCursor,
}

pub fn build_extrude_cursor(
    builder: &CadMeshBuilder<RoundCabinSegment>,
    cad_shell: &CadShell,
) -> Result<CadCursor> {
    let RoundCabinSegment {
        profile_width,
        profile_height,
        profile_corner_radius,
        profile_thickness,
        profile_extrude_length,
        end_wall_thickness,
        window,
        window_translation,
    } = &builder.params;

    let Some(CadElement::Face(face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("ProfileFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };
    let face_normal = face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();
    let face_boundaries = face.boundaries();
    let face_wire = face_boundaries.last().expect("No wire found!");
    let face_centroid = face_wire.get_centroid();
    let ref_edge_direction = ref_edge_direction_for_wire(face_wire.clone())?;
    let cursor_normal = face_normal.cross(ref_edge_direction).normalize();
    let right_direction = cursor_normal.cross(face_normal).normalize();
    let main_mesh_transform = Transform::from_translation(Vec3::new(
        *profile_thickness as f32,
        *profile_thickness as f32,
        (profile_extrude_length - end_wall_thickness) as f32,
    ));
    let cursor_transform = Transform::from_translation(
        main_mesh_transform.translation
            + face_centroid.as_vec3()
            + right_direction * (*profile_width as f32 / 2. + 0.1),
    )
    .with_rotation(get_rotation_from_normals(Vec3::Z, cursor_normal));

    Ok(CadCursor {
        normal: cursor_normal,
        transform: cursor_transform,
        cursor_type: CadCursorType::Linear {
            direction: face_normal,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}
