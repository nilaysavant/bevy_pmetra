use bevy_pmetra::re_exports::{
    truck_modeling::{builder, Curve, EuclideanSpace, Leader, Point3, Rad, Solid, Vector3},
    truck_shapeops,
};

/// `Cube` minus `Cylinder`.
///
/// Truck shape-ops example.
///
/// Ref: https://github.com/ricosjp/truck/blob/db958f90adf39bfaf8a7d758672f57f99948b2a3/truck-shapeops/examples/punched-cube-shapeops.rs
pub fn basic_shape_ops_eg() -> Solid {
    let v = builder::vertex(Point3::origin());
    let e = builder::tsweep(&v, Vector3::unit_x());
    let f = builder::tsweep(&e, Vector3::unit_y());
    let cube = builder::tsweep(&f, Vector3::unit_z());

    let v = builder::vertex(Point3::new(0.5, 0.25, -0.5));
    let w = builder::rsweep(&v, Point3::new(0.5, 0.5, 0.0), Vector3::unit_z(), Rad(7.0));
    let f = builder::try_attach_plane(&[w]).unwrap();
    let mut cylinder = builder::tsweep(&f, Vector3::unit_z() * 2.0);
    cylinder.not();
    let and = truck_shapeops::and(&cube, &cylinder, 0.05).unwrap();
    and.edge_iter().for_each(|edge| {
        let mut curve = edge.curve();
        if let Curve::IntersectionCurve(inter) = &curve {
            if matches! { inter.leader(), Leader::Polyline(_) } {
                let flag = curve.to_bspline_leader(0.01, 0.1, 20);
                println!("{flag}");
            }
        }
        edge.set_curve(curve);
    });

    and
}
