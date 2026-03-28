use bevy::{color::palettes::css, math::DVec3, prelude::*};
use bevy_pmetra::{
    math::get_rotation_from_normals,
    pmetra_core::extensions::shell::ShellCadExtension,
    prelude::*,
    re_exports::{
        anyhow::{anyhow, Result},
        truck_modeling::{
            builder, BSplineSurface, Curve, Face, KnotVec, NurbsSurface, Point3, Shell, Solid,
            Surface, Vector3, Vector4, Wire,
        },
    },
};

use super::ExpNurbs;

pub fn build_nurbs_surface_shell(params: &ExpNurbs) -> Result<CadShell> {
    let tagged_elements = CadTaggedElements::default();

    let control_points = params.control_points;

    let (p00, p10, p20, p30, p01, p11, p21, p31) = (
        Point3::from_bevy_dvec3(control_points[0]),
        Point3::from_bevy_dvec3(control_points[1]),
        Point3::from_bevy_dvec3(control_points[2]),
        Point3::from_bevy_dvec3(control_points[3]),
        Point3::from_bevy_dvec3(control_points[4]),
        Point3::from_bevy_dvec3(control_points[5]),
        Point3::from_bevy_dvec3(control_points[6]),
        Point3::from_bevy_dvec3(control_points[7]),
    );

    let v00 = builder::vertex(p00);
    let v30 = builder::vertex(p30);
    let v01 = builder::vertex(p01);
    let v31 = builder::vertex(p31);

    let edge_back = builder::bezier::<Curve>(&v00, &v30, vec![p10, p20]);
    let edge_right = builder::line::<Curve>(&v30, &v31);
    let edge_front = builder::bezier::<Curve>(&v01, &v31, vec![p11, p21]);
    let edge_left = builder::line::<Curve>(&v01, &v00);

    let wire = Wire::from(vec![edge_back, edge_right, edge_front.inverse(), edge_left]).inverse();

    let knot_u = KnotVec::bezier_knot(ExpNurbs::U_COUNT - 1);
    let knot_v = KnotVec::bezier_knot(ExpNurbs::V_COUNT - 1);
    let control_points = build_surface_control_points(&params.control_points);
    let surface = NurbsSurface::new(BSplineSurface::new((knot_u, knot_v), control_points));
    let top_face = Face::new(vec![wire.clone()], Surface::NurbsSurface(surface));
    let solid = builder::tsweep(
        &top_face.inverse(),
        Vector3::unit_y() * -params.surface_thickness,
    );

    let shell = Shell::try_from_solid(&solid)?;

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

fn build_surface_control_points(
    points: &[DVec3; ExpNurbs::CONTROL_POINTS_COUNT],
) -> Vec<Vec<Vector4>> {
    let mut control_points =
        vec![vec![Vector4::new(0.0, 0.0, 0.0, 1.0); ExpNurbs::V_COUNT]; ExpNurbs::U_COUNT];

    for (u, row) in control_points.iter_mut().enumerate() {
        for (v, cell) in row.iter_mut().enumerate() {
            let index = (ExpNurbs::V_COUNT - 1 - v) * ExpNurbs::U_COUNT + u;
            let point = points[index];
            *cell = Vector4::new(point.x, point.y, point.z, 1.0);
        }
    }
    control_points
}

pub fn nurbs_surface_mesh_builder(
    params: &ExpNurbs,
    shell_name: CadShellName,
) -> Result<CadMeshBuilder<ExpNurbs>> {
    let transform = Transform::default();

    let mesh_builder = CadMeshBuilder::new(params.clone(), shell_name.clone())?
        .set_transform(transform)?
        .set_base_material(Color::from(css::ORANGE).into())?;

    Ok(mesh_builder)
}

pub fn build_control_point_slider(params: &ExpNurbs, index: usize) -> Result<CadSlider> {
    if index >= ExpNurbs::CONTROL_POINTS_COUNT {
        return Err(anyhow!("Control point index out of bounds"));
    }

    let mut position = params.control_points[index].as_vec3();
    let z_offset = 0.05;
    if index < ExpNurbs::U_COUNT {
        position.z -= z_offset;
    } else {
        position.z += z_offset;
    }
    let slider_transform = Transform::from_translation(position);

    Ok(CadSlider {
        drag_plane_normal: Vec3::Z,
        transform: slider_transform,
        slider_type: CadSliderType::Planer,
        ..default()
    })
}

pub fn build_surface_length_slider(params: &ExpNurbs) -> Result<CadSlider> {
    let p4 = params.control_points[4];
    let p7 = params.control_points[7];
    let mut midpoint = ((p4 + p7) * 0.5).as_vec3();
    midpoint.y = 0.0;
    midpoint += Vec3::Z * 0.15;
    let slider_transform = Transform::from_translation(midpoint)
        .with_rotation(get_rotation_from_normals(Vec3::Z, Vec3::Y));

    Ok(CadSlider {
        drag_plane_normal: Vec3::Y,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: Vec3::Z,
            limit_min: None,
            limit_max: None,
        },
        ..default()
    })
}
