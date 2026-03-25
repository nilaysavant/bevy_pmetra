use bevy::{color::palettes::css, prelude::*};
use bevy_pmetra::{
    math::get_rotation_from_normals,
    prelude::*,
    re_exports::{
        anyhow::{anyhow, Result},
        truck_modeling::{
            builder, BSplineSurface, Curve, Face, KnotVec, NurbsSurface, Point3, Shell, Surface,
            Vector4, Wire,
        },
    },
};

use super::ExpNurbsSolid;

const CONTROL_POINTS_COUNT: usize = 8;
const U_COUNT: usize = 4;
const V_COUNT: usize = 2;

pub fn build_nurbs_surface_shell(params: &ExpNurbsSolid) -> Result<CadShell> {
    let tagged_elements = CadTaggedElements::default();

    let control_points = params.control_points;

    let (p00, p10, p20, p30, p01, p11, p21, p31) = (
        to_point3(control_points[0]),
        to_point3(control_points[1]),
        to_point3(control_points[2]),
        to_point3(control_points[3]),
        to_point3(control_points[4]),
        to_point3(control_points[5]),
        to_point3(control_points[6]),
        to_point3(control_points[7]),
    );

    let v00 = builder::vertex(p00);
    let v30 = builder::vertex(p30);
    let v01 = builder::vertex(p01);
    let v31 = builder::vertex(p31);

    let edge_bottom = builder::bezier::<Curve>(&v00, &v30, vec![p10, p20]);
    let edge_right = builder::line::<Curve>(&v30, &v31);
    let edge_top = builder::bezier::<Curve>(&v01, &v31, vec![p11, p21]);
    let edge_left = builder::line::<Curve>(&v01, &v00);

    let wire = Wire::from(vec![edge_bottom, edge_right, edge_top.inverse(), edge_left]).inverse();

    let knot_u = KnotVec::bezier_knot(3);
    let knot_v = KnotVec::bezier_knot(1);
    let control_points = build_surface_control_points(&params.control_points);
    let surface = NurbsSurface::new(BSplineSurface::new((knot_u, knot_v), control_points));
    let face = Face::new(vec![wire], Surface::NurbsSurface(surface));

    let shell: Shell = vec![face].into_iter().collect();

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

pub fn nurbs_surface_mesh_builder(
    params: &ExpNurbsSolid,
    shell_name: CadShellName,
) -> Result<CadMeshBuilder<ExpNurbsSolid>> {
    let transform = Transform::default();

    let mesh_builder = CadMeshBuilder::new(params.clone(), shell_name.clone())?
        .set_transform(transform)?
        .set_base_material(Color::from(css::ORANGE).into())?;

    Ok(mesh_builder)
}

pub fn build_control_point_slider(params: &ExpNurbsSolid, index: usize) -> Result<CadSlider> {
    if index >= CONTROL_POINTS_COUNT {
        return Err(anyhow!("Control point index out of bounds"));
    }

    let position = vec3_from_control_point(params.control_points[index]);
    let slider_transform = Transform::from_translation(position);

    Ok(CadSlider {
        drag_plane_normal: Vec3::Z,
        transform: slider_transform,
        slider_type: CadSliderType::Planer,
        ..default()
    })
}

pub fn build_surface_length_slider(params: &ExpNurbsSolid) -> Result<CadSlider> {
    let p0 = vec3_from_control_point(params.control_points[0]);
    let p1 = vec3_from_control_point(params.control_points[4]);
    let midpoint = (p0 + p1) * 0.5;
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

fn to_point3(point: [f32; 3]) -> Point3 {
    Point3::new(point[0] as f64, point[1] as f64, point[2] as f64)
}

fn build_surface_control_points(points: &[[f32; 3]; CONTROL_POINTS_COUNT]) -> Vec<Vec<Vector4>> {
    let mut control_points = vec![vec![Vector4::new(0.0, 0.0, 0.0, 1.0); V_COUNT]; U_COUNT];

    for (u, row) in control_points.iter_mut().enumerate() {
        for (v, cell) in row.iter_mut().enumerate() {
            let index = (V_COUNT - 1 - v) * U_COUNT + u;
            let point = points[index];
            *cell = Vector4::new(point[0] as f64, point[1] as f64, point[2] as f64, 1.0);
        }
    }

    control_points
}

fn vec3_from_control_point(point: [f32; 3]) -> Vec3 {
    Vec3::new(point[0], point[1], point[2])
}
