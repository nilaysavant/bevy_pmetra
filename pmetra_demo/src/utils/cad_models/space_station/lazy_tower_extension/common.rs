use anyhow::{Context, Result};
use bevy::{math::DVec3, prelude::*};
use bevy_pmetra::{
    cad_core::extensions::shell::ShellCadExtension,
    re_exports::truck_modeling::{builder, Point3, Shell, Vector3, Vertex, Wire},
};
use itertools::Itertools;

/// Get L-shaped beam shell.
///
/// # Args
/// - `length`: Length of the beam.
///
/// ## Profile
///
/// The beam has a L-shaped cross section:
/// ```
/// O -> x
/// |
/// z
///
/// ao = ob = l_side_length
/// bc = ae = thickness
/// o : Located at origin(0,0, 0)
///
///  o ---------- b  ^
///  | *--------- c  V
///  | | d
///  | |
///  | |
///  a e
///
/// ```
///
pub fn l_beam_shell(l_side_length: f64, thickness: f64, length: f64) -> Result<Shell> {
    // Create points for L-shaped cross section...
    let o = DVec3::ZERO;
    let b = DVec3::new(l_side_length, 0., 0.);
    let c = b + DVec3::new(0., 0., thickness);
    let d = DVec3::new(thickness, 0., thickness);
    let a = DVec3::new(0., 0., l_side_length);
    let e = a + DVec3::new(thickness, 0., 0.);
    let points = [o, b, c, d, e, a];
    let vertices = points
        .iter()
        .map(|p| Vertex::new(Point3::from(p.to_array())))
        .collect::<Vec<_>>();
    let mut wire = Wire::new();
    for (v0, v1) in vertices.iter().circular_tuple_windows() {
        let edge = builder::line(v0, v1);
        wire.push_back(edge);
    }
    wire.invert();
    debug_assert!(wire.is_closed());
    // Attach plane and extrude into shell...
    let face =
        builder::try_attach_plane(&[wire]).with_context(|| "Could not attach plane to wire")?;
    let solid = builder::tsweep(&face, Vector3::from((DVec3::Y * length).to_array()));
    let shell = Shell::try_from_solid(&solid)?;

    Ok(shell)
}
