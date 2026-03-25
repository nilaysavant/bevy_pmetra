use bevy::{color::palettes::css, prelude::*};
use bevy_pmetra::{
    math::get_rotation_from_normals,
    pmetra_core::extensions::shell::ShellCadExtension,
    prelude::*,
    re_exports::{
        anyhow::{anyhow, Result},
        truck_modeling::{
            builder, BSplineSurface, Curve, Face, KnotVec, NurbsSurface, Point3, Shell, Solid,
            Surface, Vector4, Wire,
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
    let bottom_wire = build_bottom_wire_from_control_points(&params.control_points);

    let knot_u = KnotVec::bezier_knot(3);
    let knot_v = KnotVec::bezier_knot(1);
    let control_points = build_surface_control_points(&params.control_points);
    let surface = NurbsSurface::new(BSplineSurface::new((knot_u, knot_v), control_points));
    let top_face = Face::new(vec![wire.clone()], Surface::NurbsSurface(surface));
    let mut bottom_face = builder::try_attach_plane::<Curve, Surface>(vec![bottom_wire.clone()])?;
    bottom_face.invert();

    let mut side_shell = builder::try_wire_homotopy::<Curve, Surface>(&bottom_wire, &wire)?;
    side_shell.push(top_face);
    side_shell.push(bottom_face);

    let solid = Solid::try_new(vec![side_shell])?;
    let shell = Shell::try_from_solid(&solid)?;

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

    let mut position = vec3_from_control_point(params.control_points[index]);
    let z_offset = 0.05;
    if index < U_COUNT {
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

pub fn build_surface_length_slider(params: &ExpNurbsSolid) -> Result<CadSlider> {
    let p4 = vec3_from_control_point(params.control_points[4]);
    let p7 = vec3_from_control_point(params.control_points[7]);
    let mut midpoint = (p4 + p7) * 0.5;
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

fn build_bottom_wire_from_control_points(points: &[[f32; 3]; CONTROL_POINTS_COUNT]) -> Wire {
    let (mut min_x, mut max_x) = (points[0][0], points[0][0]);
    let (mut min_z, mut max_z) = (points[0][2], points[0][2]);

    for point in points.iter().skip(1) {
        min_x = min_x.min(point[0]);
        max_x = max_x.max(point[0]);
        min_z = min_z.min(point[2]);
        max_z = max_z.max(point[2]);
    }

    let b00 = builder::vertex(Point3::new(min_x as f64, 0.0, min_z as f64));
    let b30 = builder::vertex(Point3::new(max_x as f64, 0.0, min_z as f64));
    let b01 = builder::vertex(Point3::new(min_x as f64, 0.0, max_z as f64));
    let b31 = builder::vertex(Point3::new(max_x as f64, 0.0, max_z as f64));

    let edge_bottom = builder::line::<Curve>(&b00, &b30);
    let edge_right = builder::line::<Curve>(&b30, &b31);
    let edge_top = builder::line::<Curve>(&b01, &b31);
    let edge_left = builder::line::<Curve>(&b01, &b00);

    Wire::from(vec![edge_bottom, edge_right, edge_top.inverse(), edge_left]).inverse()
}
