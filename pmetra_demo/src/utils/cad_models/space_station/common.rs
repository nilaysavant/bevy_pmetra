use std::f64::consts::FRAC_PI_2;

use anyhow::{anyhow, Context, Result};
use bevy::{math::DVec3, prelude::*};
use bevy_pmetra::{
    prelude::*,
    re_exports::truck_modeling::{builder, cgmath::AbsDiffEq, Edge, Point3, Rad, Vector3, Wire},
};

/// Get the rounded corner arcs for rectangular profile corner points given.
pub fn get_corner_arcs_for_corner_vertices(
    a0: DVec3,
    a1: DVec3,
    a2: DVec3,
    a3: DVec3,
    profile_corner_radius: f64,
) -> (Edge, Edge, Edge, Edge) {
    // profile arcs...
    let a0_start = builder::vertex((a0 + DVec3::Y * profile_corner_radius).to_array().into());
    let a0_center = a0 + DVec3::X * profile_corner_radius + DVec3::Y * profile_corner_radius;
    let a0_end = builder::rotated(
        &a0_start,
        a0_center.to_array().into(),
        Vector3::unit_z(),
        Rad(FRAC_PI_2),
    );
    let a0_transit = builder::rotated(
        &a0_start,
        a0_center.to_array().into(),
        Vector3::unit_z(),
        Rad(FRAC_PI_2 / 2.),
    );
    let arc0 = builder::circle_arc(&a0_start, &a0_end, a0_transit.point());
    // create rot arcs based on arc0
    let arc1 = builder::rotated(
        &arc0,
        a0_center.to_array().into(),
        Vector3::unit_z(),
        Rad(FRAC_PI_2),
    );
    let arc2 = builder::rotated(
        &arc1,
        a0_center.to_array().into(),
        Vector3::unit_z(),
        Rad(FRAC_PI_2),
    );
    let arc3 = builder::rotated(
        &arc2,
        a0_center.to_array().into(),
        Vector3::unit_z(),
        Rad(FRAC_PI_2),
    );
    // translate arcs to correct pos...
    let arc1 = builder::translated(
        &arc1,
        (a1 - DVec3::X * 2. * profile_corner_radius)
            .to_array()
            .into(),
    );
    let arc2 = builder::translated(
        &arc2,
        (a2 - DVec3::Y * 2. * profile_corner_radius - DVec3::X * 2. * profile_corner_radius)
            .to_array()
            .into(),
    );
    let arc3 = builder::translated(
        &arc3,
        (a3 - DVec3::Y * 2. * profile_corner_radius)
            .to_array()
            .into(),
    );
    (arc0, arc1, arc2, arc3)
}

/// Create profile wire from the 4 corner arcs.
pub fn get_profile_from_corner_arcs(
    arc0: &Edge,
    arc1: &Edge,
    arc2: &Edge,
    arc3: &Edge,
) -> Result<Wire> {
    let mut profile = Wire::new();
    profile.push_back(arc0.clone());
    profile.push_back(builder::line(arc0.back(), arc1.front()));
    profile.push_back(arc1.clone());
    profile.push_back(builder::line(arc1.back(), arc2.front()));
    profile.push_back(arc2.clone());
    profile.push_back(builder::line(arc2.back(), arc3.front()));
    profile.push_back(arc3.clone());
    profile.push_back(builder::line(arc3.back(), arc0.front()));

    if !profile.is_simple() {
        return Err(anyhow!("Profile is not simple!"));
    }

    Ok(profile)
}

/// Used to get reference [`Edge`] (with end vertices having same y) direction [`Vec3`].
pub fn ref_edge_direction_for_wire(profile_face_wire: Wire) -> Result<Vec3> {
    let direction_edge_w_same_y = profile_face_wire
        .edge_iter()
        .find_map(|e| {
            let (v0, v1) = e.ends();
            let is_same_y = v0
                .point()
                .y
                .abs_diff_eq(&v1.point().y, Point3::default_epsilon());
            if !is_same_y {
                return None;
            };
            Some((v1.point().as_bevy_vec3() - v0.point().as_bevy_vec3()).normalize())
        })
        .with_context(|| "Could not find edge with same y!")?;
    Ok(direction_edge_w_same_y)
}
