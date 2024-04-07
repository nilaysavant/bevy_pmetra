use std::f64::consts::{PI, TAU};

use anyhow::{anyhow, Result};
use bevy::{
    log::error,
    math::{DQuat, DVec3, EulerRot},
    prelude::*,
};
use bevy_pmetra::{
    cad_core::{
        extensions::{shell::ShellCadExtension, wire::WireCadExtension},
        lazy_builders::{CadMeshLazyBuilder, CadShellName, CadShellsByName},
    },
    math::get_rotation_from_normals,
    prelude::*,
    re_exports::{
        truck_modeling::{
            builder, cgmath::AbsDiffEq, EuclideanSpace, InnerSpace, ParametricSurface3D, Point3,
            Rad, Shell, Vector3, Wire,
        },
        truck_topology::{EdgeDisplayFormat, VertexDisplayFormat, WireDisplayFormat},
    },
};
use itertools::Itertools;
use strum::{Display, EnumString};

use super::{
    math::{Line, Sphere},
    CadCursorIds, CadSolidIds, LazySimpleGear,
};

/// Build Main Gear Solid.
///
/// ## Nomenclature
///
/// Main dimensions:
/// - `z`   : num of teeth.
/// - `d`   : pitch circle diameter.
/// - `m`   : module.
/// - `de`  : outside circle diameter.
/// - `df`  : root circle diameter.
///
/// Tooth dimensions:
/// - `h`   : tooth depth.
/// - `Pc`  : circular pitch.
/// - `B`   : tooth thickness.
///
pub fn build_main_gear_shell(params: &LazySimpleGear) -> Result<CadShell> {
    let LazySimpleGear {
        num_of_teeth,
        pitch_circle_diameter,
        face_width,
    } = *params;

    let mut tagged_elements = CadTaggedElements::default();

    // Calc all dependent params...
    // m = d / z
    let module = pitch_circle_diameter / num_of_teeth as f64;
    // de = d + 2m
    let outside_circle_diameter = pitch_circle_diameter + 2. * module;
    // df = m (z – 2.5)
    let root_circle_diameter = module * (num_of_teeth as f64 - 2.5);
    // h = 2.25 * m
    let tooth_depth = 2.25 * module;
    // Pc = PI * m
    let circular_pitch = PI * module;
    // B = Pc / 2
    let tooth_thickness = circular_pitch / 2.;
    // Pa = 20 deg
    let pressure_angle = 20f64.to_radians();

    // Create gear profile...
    let center = DVec3::ZERO;
    let root_sphere = Sphere::new(center, root_circle_diameter / 2.);
    let pitch_sphere = Sphere::new(center, pitch_circle_diameter / 2.);
    let outside_sphere = Sphere::new(center, outside_circle_diameter / 2.);

    let tooth_thickness_angle = (tooth_thickness / pitch_circle_diameter) * 2.;
    let tooth_half_thickness_quat = DQuat::from_rotation_y(-tooth_thickness_angle / 2.); // neg because anticlockwise around y is -ve.

    let tooth_thickness_top_pt =
        tooth_half_thickness_quat * (DVec3::X * pitch_circle_diameter / 2.);
    let tooth_thickness_bot_pt =
        tooth_half_thickness_quat.inverse() * (DVec3::X * pitch_circle_diameter / 2.);

    // println!("tooth_thickness_angle: {:?}", tooth_thickness_angle);
    // println!("tooth_thickness_bot_pt: {:?}", tooth_thickness_bot_pt);
    // println!("tooth_thickness_top_pt: {:?}", tooth_thickness_top_pt);

    // Get pressure angle direction vec for getting intersection pts with outer and root circle/sphere.
    let pressure_angle_quat = DQuat::from_rotation_y(-pressure_angle); // neg because anticlockwise around y is -ve.
    let pressure_angle_bot_dir = (pressure_angle_quat * DVec3::X).normalize();
    let slant_line_bot = Line::from_point_direction(tooth_thickness_bot_pt, pressure_angle_bot_dir);
    let pressure_angle_top_dir = (pressure_angle_quat.inverse() * DVec3::X).normalize();
    let slant_line_top = Line::from_point_direction(tooth_thickness_top_pt, pressure_angle_top_dir);

    // println!(
    //     "pressure_angle_quat euler: {:?}",
    //     pressure_angle_quat.to_euler(EulerRot::XYZ)
    // );
    // println!("pressure_angle_bot_dir: {:?}", pressure_angle_bot_dir);
    // println!("pressure_angle_top_dir: {:?}", pressure_angle_top_dir);

    // Get intersection pts...
    // With bottom slant line...
    let Some(slant_line_bot_intersects_root_sphere) =
        root_sphere.get_intersection_with_line(&slant_line_bot)
    else {
        return Err(anyhow!(
            "No intersection found: slant_line_bot & root sphere!"
        ));
    };
    let slant_line_bot_root_pt =
        get_point_with_max_x_from_2_tuple(slant_line_bot_intersects_root_sphere);
    let Some(slant_line_bot_intersects_outside_sphere) =
        outside_sphere.get_intersection_with_line(&slant_line_bot)
    else {
        return Err(anyhow!(
            "No intersection found: slant_line_bot & outside sphere!"
        ));
    };
    let slant_line_bot_outside_pt =
        get_point_with_max_x_from_2_tuple(slant_line_bot_intersects_outside_sphere);
    // With top slant line...
    let Some(slant_line_top_intersects_root_sphere) =
        root_sphere.get_intersection_with_line(&slant_line_top)
    else {
        return Err(anyhow!(
            "No intersection found: slant_line_top & root sphere!"
        ));
    };
    let slant_line_top_root_pt =
        get_point_with_max_x_from_2_tuple(slant_line_top_intersects_root_sphere);
    let Some(slant_line_top_intersects_outside_sphere) =
        outside_sphere.get_intersection_with_line(&slant_line_top)
    else {
        return Err(anyhow!(
            "No intersection found: slant_line_top & outside sphere!"
        ));
    };
    let slant_line_top_outside_pt =
        get_point_with_max_x_from_2_tuple(slant_line_top_intersects_outside_sphere);

    // println!("slant_line_bot_root_pt: {:?}", slant_line_bot_root_pt);
    // println!("slant_line_top_root_pt: {:?}", slant_line_top_root_pt);
    // println!("slant_line_bot_outside_pt: {:?}", slant_line_bot_outside_pt);
    // println!("slant_line_top_outside_pt: {:?}", slant_line_top_outside_pt);

    // Construct teeth (currently simple triangular)...
    // TODO - Impl proper teeth with arcs later.
    let v_bot_root = builder::vertex(slant_line_bot_root_pt.to_array().into());
    let v_bot_out = builder::vertex(slant_line_bot_outside_pt.to_array().into());
    let v_top_out = builder::vertex(slant_line_top_outside_pt.to_array().into());
    let v_top_root = builder::vertex(slant_line_top_root_pt.to_array().into());

    let e_bot = builder::line(&v_bot_root, &v_bot_out);
    let e_int_out = builder::line(&v_bot_out, &v_top_out);
    let e_top = builder::line(&v_top_out, &v_top_root);
    // Create wire from edges...
    let mut tooth_wire = Wire::new();
    tooth_wire.push_back(e_bot);
    tooth_wire.push_back(e_int_out);
    tooth_wire.push_back(e_top);

    let mut all_teeth = vec![];
    let mut angle = 0.;
    while angle < TAU {
        let rot_wire =
            builder::rotated(&tooth_wire, Point3::origin(), Vector3::unit_y(), Rad(angle));
        // println!(
        //     "rot_wire: {:#?}",
        //     rot_wire.display(WireDisplayFormat::EdgesListTuple {
        //         edge_format: EdgeDisplayFormat::VerticesTuple {
        //             vertex_format: VertexDisplayFormat::AsPoint
        //         }
        //     })
        // );
        all_teeth.push(rot_wire);

        angle += tooth_thickness_angle * 2.;
    }
    // println!("all_teeth len: {:?}", all_teeth.len());

    // Connect teeth with intermediate arcs...
    let mut profile_wire = Wire::new();
    // use circular tup windows iter for getting back the last pair from end to start
    // to allow last teeth with connecting edge to be added.
    for (t_prev, t_next) in all_teeth.iter().circular_tuple_windows::<(_, _)>() {
        let Some(prev_vert) = t_prev.back_vertex() else {
            error!("Could not get back vertex!");
            continue;
        };
        let Some(next_vert) = t_next.front_vertex() else {
            error!("Could not get front vertex!");
            continue;
        };
        let transit_vertex = builder::rotated(
            prev_vert,
            Point3::origin(),
            Vector3::unit_y(),
            Rad(-tooth_thickness_angle / 2.),
        );
        let arc = builder::circle_arc(prev_vert, next_vert, transit_vertex.point());
        // Add to profile...
        t_prev.iter().for_each(|e| {
            profile_wire.push_back(e.clone());
        });
        profile_wire.push_back(arc);
    }

    // println!("profile_wire.is_closed: {:?}", profile_wire.is_closed());
    // println!("profile_wire.is_simple: {:?}", profile_wire.is_simple());

    let Ok(face) = builder::try_attach_plane(&[profile_wire.inverse()]) else {
        return Err(anyhow!("Could not attach plane to profile wire!"));
    };

    let solid = builder::tsweep(&face, Vector3::unit_y() * face_width);

    let top_face = solid
        .face_iter()
        .find(|face| {
            face.vertex_iter().all(|v| {
                v.point()
                    .y
                    .abs_diff_eq(&face_width, Point3::default_epsilon())
            })
        })
        .ok_or_else(|| anyhow!("Could not find top face!"))?;
    tagged_elements.insert(
        CadElementTag::new("TopFace"),
        CadElement::Face(top_face.clone()),
    );

    let right_most_face = solid
        .face_iter()
        .find(|face| {
            face.vertex_iter().all(|v| {
                v.point()
                    .x
                    .abs_diff_eq(&v_top_out.point().x, Point3::default_epsilon())
            })
        })
        .ok_or_else(|| anyhow!("Could not find right most face!"))?;
    tagged_elements.insert(
        CadElementTag::new("RightMostFace"),
        CadElement::Face(right_most_face.clone()),
    );

    let shell = Shell::try_from_solid(&solid)?;

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

fn get_point_with_max_x_from_2_tuple(points_tuple: (DVec3, DVec3)) -> DVec3 {
    if points_tuple.0.x > points_tuple.1.x {
        points_tuple.0
    } else {
        points_tuple.1
    }
}

pub fn build_main_gear_mesh(
    params: &LazySimpleGear,
    shell_name: CadShellName,
    shells_by_name: &CadShellsByName,
) -> Result<CadMeshLazyBuilder<LazySimpleGear>> {
    let cad_shell = shells_by_name
        .get(&shell_name)
        .ok_or_else(|| anyhow!("Could not get shell by name!"))?;

    let poly_mesh = cad_shell.build_polygon_with_tol(FAST_TRIANGULATION_TOL_1 * 1.5)?;
    let mesh = poly_mesh.build_mesh();
    // spawn entity with generated mesh...
    let transform = Transform::default();
    let mesh_builder = CadMeshLazyBuilder::new(params.clone(), shell_name.clone())? // builder
        .set_base_material(Color::RED.into())?
        .set_outlines(cad_shell.shell.build_outlines())?
        .set_transform(transform)?;

    Ok(mesh_builder)
}

pub fn build_radius_cursor(
    params: &LazySimpleGear,
    shells_by_name: &CadShellsByName,
) -> Result<CadCursor> {
    let LazySimpleGear {
        num_of_teeth,
        pitch_circle_diameter,
        face_width,
    } = &params;

    let cad_shell = shells_by_name
        .get(&CadShellName(CadSolidIds::MainGear.to_string()))
        .ok_or_else(|| anyhow!("Could not get main gear shell!"))?;

    let Some(CadElement::Face(face)) = cad_shell.get_element_by_tag(CadElementTag::new("TopFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };

    let face_normal = face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();
    let face_boundaries = face.boundaries();
    let face_wire = face_boundaries.last().expect("No wire found!");
    let face_centroid = face_wire.get_centroid();
    let right_direction = Vec3::X;
    let mesh_transform = Transform::default();
    let cursor_transform = Transform::from_translation(
        mesh_transform.translation
            + face_centroid.as_vec3()
            + right_direction * (*pitch_circle_diameter as f32 / 2.)
            + face_normal * 0.001,
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

pub fn build_face_width_cursor(
    params: &LazySimpleGear,
    shells_by_name: &CadShellsByName,
) -> Result<CadCursor> {
    let cad_shell = shells_by_name
        .get(&CadShellName(CadSolidIds::MainGear.to_string()))
        .ok_or_else(|| anyhow!("Could not get main gear shell!"))?;

    let Some(CadElement::Face(face)) = cad_shell.get_element_by_tag(CadElementTag::new("TopFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };

    let face_normal = face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();
    let face_boundaries = face.boundaries();
    let face_wire = face_boundaries.last().expect("No wire found!");
    let face_centroid = face_wire.get_centroid();
    let mesh_transform = Transform::default();
    let cursor_normal = Vec3::X.cross(face_normal).normalize();
    let cursor_transform = Transform::from_translation(
        mesh_transform.translation + face_centroid.as_vec3() + face_normal * 0.02,
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

pub fn build_num_teeth_cursor(
    params: &LazySimpleGear,
    shells_by_name: &CadShellsByName,
) -> Result<CadCursor> {
    let LazySimpleGear {
        num_of_teeth,
        pitch_circle_diameter,
        face_width,
    } = &params;

    let cad_shell = shells_by_name
        .get(&CadShellName(CadSolidIds::MainGear.to_string()))
        .ok_or_else(|| anyhow!("Could not get main gear shell!"))?;

    let Some(CadElement::Face(top_face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("TopFace"))
    else {
        return Err(anyhow!("Could not find top face!"));
    };
    let Some(CadElement::Face(face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("RightMostFace"))
    else {
        return Err(anyhow!("Could not find right most face!"));
    };

    let top_face_normal = top_face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();
    let face_normal = face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();
    let face_boundaries = face.boundaries();
    let face_wire = face_boundaries.last().expect("No wire found!");
    let face_centroid = face_wire.get_centroid();
    let mesh_transform = Transform::default();
    let cursor_transform = Transform::from_translation(
        mesh_transform.translation + face_centroid.as_vec3() + face_normal * 0.005,
    )
    .with_rotation(get_rotation_from_normals(Vec3::Z, face_normal));
    let direction = top_face_normal.cross(face_normal).normalize();

    Ok(CadCursor {
        normal: face_normal,
        transform: cursor_transform,
        cursor_type: CadCursorType::Linear {
            direction,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}

mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    pub fn zip_test() {
        let v = vec![0, 1, 2, 3, 4];
        for e in v.iter().circular_tuple_windows::<(_, _)>() {
            println!("e: {:?}", e);
        }
    }
}
