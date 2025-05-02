use std::f64::consts::FRAC_PI_2;

use bevy::{color::palettes::css, math::DVec3, prelude::*};
use bevy_pmetra::{
    math::get_rotation_from_normals,
    pmetra_core::extensions::face::FaceCadExtension,
    prelude::*,
    re_exports::{
        anyhow::{anyhow, Context, Result},
        truck_modeling::{
            builder, cgmath::AbsDiffEq, Face, ParametricSurface3D, Point3, Rad, Shell, Tolerance,
            Vector3,
        },
    },
};

use crate::utils::cad_models::space_station::common::{
    get_corner_arcs_for_corner_vertices, get_profile_from_corner_arcs, ref_edge_direction_for_wire,
};

use super::{CadShellIds, RoundCabinSegment};

pub fn build_cabin_shell(params: &RoundCabinSegment) -> Result<CadShell> {
    let RoundCabinSegment {
        profile_width,
        profile_height,
        profile_corner_radius,
        profile_thickness,
        profile_extrude_length,
        window,
        window_translation,
        ..
    } = params.clone();

    let mut tagged_elements = CadTaggedElements::default();

    // alignment vertices...
    let a0 = DVec3::ZERO;
    let a1 = a0 + DVec3::X * profile_width;
    let a2 = a1 + DVec3::Y * profile_height;
    let a3 = a2 - DVec3::X * profile_width;

    let (arc0, arc1, arc2, arc3) =
        get_corner_arcs_for_corner_vertices(a0, a1, a2, a3, profile_corner_radius);
    tagged_elements.insert(CadElementTag::new("Arc3"), CadElement::Edge(arc3.clone()));
    // connect all arcs with intermediate wires and create a profile...
    let profile = get_profile_from_corner_arcs(&arc0, &arc1, &arc2, &arc3)?;
    // calc centroid
    let profile_centroid = profile.get_centroid();

    // get inner arcs from outer arcs...
    let arc0_in = builder::translated(
        &arc0,
        DVec3::new(profile_thickness, profile_thickness, 0.)
            .to_array()
            .into(),
    );
    let arc1_in = builder::translated(
        &arc1,
        DVec3::new(-profile_thickness, profile_thickness, 0.)
            .to_array()
            .into(),
    );
    let arc2_in = builder::translated(
        &arc2,
        DVec3::new(-profile_thickness, -profile_thickness, 0.)
            .to_array()
            .into(),
    );
    let arc3_in = builder::translated(
        &arc3,
        DVec3::new(profile_thickness, -profile_thickness, 0.)
            .to_array()
            .into(),
    );
    // Create inner profile/hole profile from main profile...
    let profile_inner = get_profile_from_corner_arcs(&arc0_in, &arc1_in, &arc2_in, &arc3_in)?;
    // calc centroid
    let profile_inner_centroid = profile_inner.get_centroid();
    let profile_inner = builder::translated(
        &profile_inner,
        (profile_centroid - profile_inner_centroid)
            .to_array()
            .into(),
    );

    // Create face from profile with hole profile...
    let mut profile_face =
        builder::try_attach_plane(&[profile.clone()]).with_context(|| "Could not attach plane!")?;
    // add the inner profile (inverted for hole)
    profile_face.add_boundary(profile_inner.inverse());
    // add this as a tagged face
    tagged_elements.insert(
        CadElementTag::new("ProfileFace"),
        CadElement::Face(profile_face.clone()),
    );

    // extrude profile into shell structure...
    let mut profile_shell = builder::tsweep(&profile, Vector3::unit_z() * profile_extrude_length);
    let mut profile_inner_shell =
        builder::tsweep(&profile_inner, Vector3::unit_z() * profile_extrude_length);
    // invert faces of inner profile...
    profile_inner_shell.face_iter_mut().for_each(|f| {
        f.invert();
    });

    // window cutouts...
    let mut window = window.clone();
    window.profile_extrude_length = profile_thickness;
    let window_shell = window.try_build().with_context(|| "window build failed!")?;
    let window_shell = builder::rotated(
        &window_shell,
        DVec3::ZERO.to_array().into(),
        Vector3::unit_y(),
        Rad(FRAC_PI_2),
    );

    // Left window...
    let window_shell = builder::translated(&window_shell, window_translation.to_array().into());
    let window_left_face =
        get_shell_face_from_normal_dir(&window_shell, -Vector3::unit_x(), (0.5, 0.5))?;
    // Tag left window left face for window transform slider later...
    tagged_elements.insert(
        CadElementTag::new("LeftWindowLeftFace"),
        CadElement::Face(window_left_face.clone()),
    );
    let window_right_face =
        get_shell_face_from_normal_dir(&window_shell, Vector3::unit_x(), (0.5, 0.5))?;
    // Get wires...
    let window_left_face_wire = window_left_face.get_last_boundary_wire()?;
    let window_right_face_wire = window_right_face.get_last_boundary_wire()?;
    // Add window wires(left and right) on resp profile shells faces to punch holes...
    let profile_shell_left_face =
        get_shell_face_mut_from_normal_dir(&mut profile_shell, -Vector3::unit_x(), (0.5, 0.5))?;
    profile_shell_left_face.add_boundary(window_left_face_wire.clone().inverse());
    let profile_inner_shell_right_face = get_shell_face_mut_from_normal_dir(
        &mut profile_inner_shell,
        Vector3::unit_x(),
        (0.5, 0.5),
    )?;
    profile_inner_shell_right_face.add_boundary(window_right_face_wire.clone().inverse());
    let left_window_intermediate_shell = builder::tsweep(
        &window_left_face_wire,
        (DVec3::X * profile_thickness).to_array().into(),
    );

    // Right window...
    let window_shell = builder::translated(
        &window_shell,
        (DVec3::X * (profile_width - profile_thickness))
            .to_array()
            .into(),
    );
    let window_left_face =
        get_shell_face_from_normal_dir(&window_shell, -Vector3::unit_x(), (0.5, 0.5))?;
    let window_right_face =
        get_shell_face_from_normal_dir(&window_shell, Vector3::unit_x(), (0.5, 0.5))?;
    // Get wires...
    let window_left_face_wire = window_left_face.get_last_boundary_wire()?;
    let window_right_face_wire = window_right_face.get_last_boundary_wire()?;
    // Add window wires(left and right) on resp profile shells faces to punch holes...
    let profile_shell_right_face =
        get_shell_face_mut_from_normal_dir(&mut profile_shell, Vector3::unit_x(), (0.5, 0.5))?;
    profile_shell_right_face.add_boundary(window_right_face_wire.clone().inverse());
    // Tag right face...
    tagged_elements.insert(
        CadElementTag::new("RightFace"),
        CadElement::Face(profile_shell_right_face.clone()),
    );
    let profile_inner_shell_left_face = get_shell_face_mut_from_normal_dir(
        &mut profile_inner_shell,
        -Vector3::unit_x(),
        (0.5, 0.5),
    )?;
    profile_inner_shell_left_face.add_boundary(window_left_face_wire.clone().inverse());
    let right_window_intermediate_shell = builder::tsweep(
        &window_left_face_wire,
        (DVec3::X * profile_thickness).to_array().into(),
    );

    // create final cabin shell from outer & inner profile shells...
    let mut shell = Shell::new();
    shell.extend(profile_shell.clone());
    shell.extend(profile_inner_shell.clone());
    // Glue outer and inner window cutouts via intermediate shell...
    shell.extend(left_window_intermediate_shell.clone());
    shell.extend(right_window_intermediate_shell.clone());
    // Glue together inner and outer profile shell via profile faces...
    // add profile back face.
    shell.push(profile_face.inverse());
    // add profile front face...
    let profile_front_face = builder::translated(
        &profile_face,
        (DVec3::Z * profile_extrude_length).to_array().into(),
    );
    shell.push(profile_front_face);

    let extruded_profile_face = shell
        .face_iter()
        .find(|f| {
            f.vertex_iter().all(|v| {
                v.point()
                    .z
                    .abs_diff_eq(&profile_extrude_length, Point3::default_epsilon())
            })
        })
        .with_context(|| "Could not find extruded profile face")?;
    tagged_elements.insert(
        CadElementTag::new("ExtrudedProfileFace"),
        CadElement::Face(extruded_profile_face.clone()),
    );

    // Tag top face...
    let top_face_y = profile
        .edge_iter()
        .find_map(|e| {
            let (v0, v1) = e.ends();
            let is_top_edge = v0
                .point()
                .y
                .abs_diff_eq(&v1.point().y, Point3::default_epsilon())
                && v0.point().y > profile_centroid.y;
            if !is_top_edge {
                return None;
            }
            Some(v0.point().y)
        })
        .ok_or_else(|| anyhow!("Could not top face y"))?;
    let top_face = shell
        .face_iter()
        .find(|f| {
            f.vertex_iter().all(|v| {
                v.point()
                    .y
                    .abs_diff_eq(&top_face_y, Point3::default_epsilon())
            })
        })
        .ok_or_else(|| anyhow!("Could not get top face"))?;
    tagged_elements.insert(
        CadElementTag::new("TopFace"),
        CadElement::Face(top_face.clone()),
    );

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

/// Get's the [`Shell`]'s [`Face`] pointing in the given normal direction([`Vector3`]).
///
/// The point on the surface at which normal is measured is given by `normal_uv` (u, v).
fn get_shell_face_from_normal_dir(
    shell: &Shell,
    normal_dir: Vector3,
    normal_uv: (f64, f64),
) -> Result<&Face> {
    let window_left_face = shell
        .face_iter()
        .find(|f| {
            let surface = f.oriented_surface();
            let normal = surface.normal(normal_uv.0, normal_uv.1);
            normal.near(&normal_dir)
        })
        .ok_or_else(|| anyhow!("Could not find shell face from normal direction!"))?;
    Ok(window_left_face)
}

/// Get's the [`Shell`]'s [`Face`] (mut) pointing in the given normal direction([`Vector3`]).
///
/// The point on the surface at which normal is measured is given by `normal_uv` (u, v). default: (0.5, 0.5)
fn get_shell_face_mut_from_normal_dir(
    shell: &mut Shell,
    normal_dir: Vector3,
    normal_uv: (f64, f64),
) -> Result<&mut Face> {
    let window_left_face = shell
        .face_iter_mut()
        .find(|f| {
            let surface = f.oriented_surface();
            let normal = surface.normal(normal_uv.0, normal_uv.1);
            normal.near(&normal_dir)
        })
        .ok_or_else(|| anyhow!("Could not find shell face from normal direction!"))?;
    Ok(window_left_face)
}

pub fn build_cabin_mesh(
    params: &RoundCabinSegment,
    shell_name: CadShellName,
) -> Result<CadMeshBuilder<RoundCabinSegment>> {
    // spawn entity with generated mesh...
    let main_mesh_transform: Transform = Transform::from_translation(Vec3::ZERO);
    // Init cad mesh from mesh stuff...
    let mesh_builder = CadMeshBuilder::new(params.clone(), shell_name)? // builder
        .set_base_material(Color::from(css::RED).into())?
        .set_transform(main_mesh_transform)?;

    Ok(mesh_builder)
}

// Sliders...

pub fn build_extrude_slider(
    _params: &RoundCabinSegment,
    shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let cad_shell = shells_by_name
        .get(&CadShellName(CadShellIds::CabinShell.to_string()))
        .ok_or_else(|| anyhow!("Could not find shell!"))?;

    let Some(CadElement::Face(extruded_profile_face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("ExtrudedProfileFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };
    let extruded_profile_face_normal = extruded_profile_face
        .oriented_surface()
        .normal(0.5, 0.5)
        .as_bevy_vec3();
    let extruded_profile_face_boundaries = extruded_profile_face.boundaries();
    let extruded_profile_face_wire = extruded_profile_face_boundaries
        .last()
        .expect("No wire found!");
    let extruded_profile_face_centroid = extruded_profile_face_wire.get_centroid();
    let ref_edge_direction = ref_edge_direction_for_wire(extruded_profile_face_wire.clone())?;
    let slider_normal = extruded_profile_face_normal
        .cross(ref_edge_direction)
        .normalize();
    let slider_transform = Transform::from_translation(
        extruded_profile_face_centroid.as_vec3() + extruded_profile_face_normal * 0.1,
    )
    .with_rotation(get_rotation_from_normals(Vec3::Z, slider_normal));

    Ok(CadSlider {
        drag_plane_normal: slider_normal,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: extruded_profile_face_normal,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}

pub fn build_corner_radius_slider(
    params: &RoundCabinSegment,
    shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let RoundCabinSegment {
        profile_extrude_length,
        ..
    } = &params;

    let cad_shell = shells_by_name
        .get(&CadShellName(CadShellIds::CabinShell.to_string()))
        .ok_or_else(|| anyhow!("Could not find shell!"))?;

    let Some(CadElement::Face(extruded_profile_face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("ExtrudedProfileFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };
    let extruded_profile_face_normal = extruded_profile_face
        .oriented_surface()
        .normal(0.5, 0.5)
        .as_bevy_vec3();
    let profile_face_boundaries = extruded_profile_face.boundaries();
    let profile_face_wire = profile_face_boundaries.last().expect("No wire found!");
    let ref_edge_direction = ref_edge_direction_for_wire(profile_face_wire.clone())?;
    let Some(CadElement::Edge(arc3)) = cad_shell.get_element_by_tag(CadElementTag::new("Arc3"))
    else {
        return Err(anyhow!("Could not find arc3!"));
    };
    let (arc3_v1, arc3_v2) = arc3.ends();
    let slider_translation = (arc3_v1.point().as_bevy_vec3() + arc3_v2.point().as_bevy_vec3()) / 2.
        + Vec3::Z * *profile_extrude_length as f32;
    let slider_transform =
        Transform::from_translation(slider_translation + extruded_profile_face_normal * 0.01)
            .with_rotation(get_rotation_from_normals(
                Vec3::Z,
                extruded_profile_face_normal,
            ));

    Ok(CadSlider {
        drag_plane_normal: extruded_profile_face_normal,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: ref_edge_direction,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}

pub fn build_profile_width_slider(
    _params: &RoundCabinSegment,
    shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let cad_shell = shells_by_name
        .get(&CadShellName(CadShellIds::CabinShell.to_string()))
        .ok_or_else(|| anyhow!("Could not find shell!"))?;

    let Some(CadElement::Face(right_face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("RightFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };
    let right_face_normal = right_face
        .oriented_surface()
        .normal(0.5, 0.5)
        .as_bevy_vec3();
    let right_face_boundaries = right_face.boundaries();
    let right_face_wire = right_face_boundaries.first().expect("No wire found!");
    let right_face_centroid = right_face_wire.get_centroid();

    let ref_edge_direction = ref_edge_direction_for_wire(right_face_wire.clone())?;
    let slider_translation = right_face_centroid.as_vec3();
    let top_direction = right_face_normal.cross(ref_edge_direction).normalize();
    let slider_normal = right_face_normal.cross(top_direction).normalize();
    let slider_transform =
        Transform::from_translation(slider_translation + right_face_normal * 0.1)
            .with_rotation(get_rotation_from_normals(Vec3::Z, slider_normal));

    Ok(CadSlider {
        drag_plane_normal: slider_normal,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: right_face_normal,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}

pub fn build_profile_height_slider(
    _params: &RoundCabinSegment,
    shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let cad_shell = shells_by_name
        .get(&CadShellName(CadShellIds::CabinShell.to_string()))
        .ok_or_else(|| anyhow!("Could not find shell!"))?;

    let Some(CadElement::Face(extruded_profile_face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("ExtrudedProfileFace"))
    else {
        return Err(anyhow!("Could not find ExtrudedProfileFace!"));
    };
    let profile_face_boundaries = extruded_profile_face.boundaries();
    let profile_face_wire = profile_face_boundaries.first().expect("No wire found!");

    let Some(CadElement::Face(top_face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("TopFace"))
    else {
        return Err(anyhow!("Could not find top TopFace!"));
    };
    let top_face_boundaries = top_face.boundaries();
    let top_face_wire = top_face_boundaries
        .last()
        .ok_or_else(|| anyhow!("Could not get boundary"))?;
    let top_face_normal = top_face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();

    let ref_edge_direction = ref_edge_direction_for_wire(profile_face_wire.clone())?;
    let slider_translation = top_face_wire.get_centroid();
    let slider_normal = ref_edge_direction.cross(top_face_normal).normalize();
    let slider_transform =
        Transform::from_translation(slider_translation.as_vec3() + top_face_normal * 0.1)
            .with_rotation(get_rotation_from_normals(Vec3::Z, slider_normal));

    Ok(CadSlider {
        drag_plane_normal: slider_normal,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: top_face_normal,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}

pub fn build_profile_thickness_slider(
    _params: &RoundCabinSegment,
    shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let cad_shell = shells_by_name
        .get(&CadShellName(CadShellIds::CabinShell.to_string()))
        .ok_or_else(|| anyhow!("Could not find shell!"))?;

    let Some(CadElement::Face(extruded_profile_face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("ExtrudedProfileFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };
    let extruded_profile_face_normal = extruded_profile_face
        .oriented_surface()
        .normal(0.5, 0.5)
        .as_bevy_vec3();
    let profile_face_boundaries = extruded_profile_face.boundaries();
    let profile_face_wire = profile_face_boundaries.last().expect("No wire found!");
    let profile_face_wire_centroid = profile_face_wire.get_centroid();
    let left_edge_center = profile_face_wire
        .edge_iter()
        .find_map(|e| {
            let (v0, v1) = e.ends();
            let is_left_edge = v0
                .point()
                .x
                .abs_diff_eq(&v1.point().x, Point3::default_epsilon())
                && v0.point().x < profile_face_wire_centroid.x;
            if !is_left_edge {
                return None;
            }
            Some((v0.point().as_bevy_vec3() + v1.point().as_bevy_vec3()) / 2.)
        })
        .ok_or_else(|| anyhow!("Could not find right edge with same x"))?;

    let ref_edge_direction = ref_edge_direction_for_wire(profile_face_wire.clone())?;
    let slider_translation = left_edge_center;
    let slider_transform =
        Transform::from_translation(slider_translation + extruded_profile_face_normal * 0.01)
            .with_rotation(get_rotation_from_normals(
                Vec3::Z,
                extruded_profile_face_normal,
            ));

    Ok(CadSlider {
        drag_plane_normal: extruded_profile_face_normal,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: ref_edge_direction,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}

pub fn build_window_translation_slider(
    _params: &RoundCabinSegment,
    shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let cad_shell = shells_by_name
        .get(&CadShellName(CadShellIds::CabinShell.to_string()))
        .ok_or_else(|| anyhow!("Could not find shell!"))?;

    let Some(CadElement::Face(left_window_left_face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("LeftWindowLeftFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };

    let face_normal = left_window_left_face
        .oriented_surface()
        .normal(0.5, 0.5)
        .as_bevy_vec3();
    let boundaries = left_window_left_face.boundaries();
    let face_wire = boundaries.last().expect("No wire found!");
    let wire_centroid = face_wire.get_centroid();

    let slider_translation = wire_centroid;
    let slider_transform =
        Transform::from_translation(slider_translation.as_vec3() + face_normal * 0.01)
            .with_rotation(get_rotation_from_normals(Vec3::Z, face_normal));

    Ok(CadSlider {
        drag_plane_normal: face_normal,
        transform: slider_transform,
        slider_type: CadSliderType::Planer,
        ..default()
    })
}
