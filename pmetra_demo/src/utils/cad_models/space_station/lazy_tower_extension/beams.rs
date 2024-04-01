use anyhow::{anyhow, Context, Error, Result};
use bevy::{math::DVec3, prelude::*};
use bevy_pmetra::{
    cad_core::{
        extensions::shell::ShellCadExtension,
        lazy_builders::{CadMeshLazyBuilder, CadShellName},
    },
    prelude::*,
    re_exports::truck_modeling::{builder, Point3, Shell, Vector3, Vertex, Wire},
};
use itertools::Itertools;

use super::LazyTowerExtension;

/// Straight Beam Shell Builder.
pub fn build_straight_beam_shell(params: &LazyTowerExtension) -> Result<CadShell> {
    let LazyTowerExtension {
        tower_length,
        straight_beam_l_sect_side_len,
        straight_beam_l_sect_thickness,
        ..
    } = params.clone();

    let mut tagged_elements = CadTaggedElements::default();

    let shell = l_beam_shell(
        straight_beam_l_sect_side_len,
        straight_beam_l_sect_thickness,
        tower_length,
    )?;

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

/// Cross Beam Shell Builder.
pub fn build_cross_beam_shell(params: &LazyTowerExtension) -> Result<CadShell> {
    let LazyTowerExtension {
        cross_beam_length,
        cross_beam_l_sect_side_len,
        cross_beam_l_sect_thickness,
        ..
    } = params.clone();

    let mut tagged_elements = CadTaggedElements::default();

    let shell = l_beam_shell(
        cross_beam_l_sect_side_len,
        cross_beam_l_sect_thickness,
        cross_beam_length,
    )?;

    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

/// Get L-shaped beam shell.
///
/// # Args
/// - `length`: Length of the beam.
///
/// ## Profile
///
/// The beam has a L-shaped cross section:
/// ```ignore
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

pub fn straight_beam_mesh_builder(
    params: &LazyTowerExtension,
    shell_name: CadShellName,
    transform: Transform,
) -> Result<CadMeshLazyBuilder<LazyTowerExtension>> {
    let LazyTowerExtension {
        tower_length,
        straight_beam_l_sect_side_len,
        straight_beam_l_sect_thickness,
        ..
    } = &params;
    // spawn entity with generated mesh...

    let mesh_builder = CadMeshLazyBuilder::new(params.clone(), shell_name.clone())? // builder
        .set_transform(transform)?
        .set_base_material(Color::RED.into())?;

    Ok(mesh_builder)
}

pub fn cross_beam_mesh_builder(
    params: &LazyTowerExtension,
    shell_name: CadShellName,
    transform: Transform,
) -> Result<CadMeshLazyBuilder<LazyTowerExtension>> {
    let LazyTowerExtension {
        cross_beam_length,
        cross_beam_l_sect_side_len,
        cross_beam_l_sect_thickness,
        ..
    } = &params;
    // spawn entity with generated mesh...

    let mesh_builder = CadMeshLazyBuilder::new(params.clone(), shell_name.clone())? // builder
        .set_transform(transform)?
        .set_base_material(Color::YELLOW.into())?;

    Ok(mesh_builder)
}

mod tests {
    use super::*;

    #[test]
    pub fn test_shell_builder() {
        let params = LazyTowerExtension {
            tower_length: 1.0,
            straight_beam_l_sect_side_len: 0.25,
            straight_beam_l_sect_thickness: 0.05,
            ..Default::default()
        };

        let shell = build_straight_beam_shell(&params).unwrap();
    }
}
