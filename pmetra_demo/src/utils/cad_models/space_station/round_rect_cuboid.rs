use bevy::{math::DVec3, prelude::*};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_pmetra::{
    pmetra_core::extensions::shell::ShellCadExtension,
    re_exports::{
        anyhow::{Context, Error, Result},
        truck_modeling::{builder, Shell, Vector3},
    },
};

use super::common::{get_corner_arcs_for_corner_vertices, get_profile_from_corner_arcs};

/// Rounded Rectangle Cuboid.
#[derive(Debug, Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct RoundRectCuboid {
    #[inspector(min = 0.2, speed = 0.1)]
    pub profile_width: f64,
    #[inspector(min = 0.2, speed = 0.1)]
    pub profile_height: f64,
    #[inspector(min = 0.01)]
    pub profile_corner_radius: f64,
    #[inspector(min = 0.2)]
    pub profile_extrude_length: f64,
}

impl RoundRectCuboid {
    /// Try Build a [`Shell`] for [`RoundRectCuboid`]
    pub fn try_build(self) -> Result<Shell> {
        Shell::try_from(self)
    }
}

impl Default for RoundRectCuboid {
    fn default() -> Self {
        Self {
            profile_width: 1.3,
            profile_height: 1.,
            profile_corner_radius: 0.1,
            profile_extrude_length: 1.2,
        }
    }
}

impl TryFrom<RoundRectCuboid> for Shell {
    type Error = Error;

    fn try_from(value: RoundRectCuboid) -> Result<Self> {
        let RoundRectCuboid {
            profile_width,
            profile_height,
            profile_corner_radius,
            profile_extrude_length,
        } = value;

        // Corner alignment vertices...
        let c0 = DVec3::ZERO;
        let c1 = c0 + DVec3::X * profile_width;
        let c2 = c1 + DVec3::Y * profile_height;
        let c3 = c2 - DVec3::X * profile_width;
        let (window_arc0, window_arc1, window_arc2, window_arc3) =
            get_corner_arcs_for_corner_vertices(c0, c1, c2, c3, profile_corner_radius);
        // Create profile...
        let profile =
            get_profile_from_corner_arcs(&window_arc0, &window_arc1, &window_arc2, &window_arc3)?;

        // Create face from profile with hole profile...
        let profile_face =
            builder::try_attach_plane(&[profile]).with_context(|| "Could not attach plane!")?;
        // extrude profile into solid/cuboid structure...
        let cuboid = builder::tsweep(&profile_face, Vector3::unit_z() * profile_extrude_length);

        let shell = Shell::try_from_solid(&cuboid)?;

        Ok(shell)
    }
}
