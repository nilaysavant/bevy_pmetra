#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::let_and_return)]

/// [`bevy`] meshing utils.
pub mod bevy_mesh;
/// Plugin for integrating with [`bevy`].
pub mod bevy_plugin;
/// Core Data Structures/Traits/Extensions/Types.
pub mod pmetra_core;
/// Constants.
pub mod constants;
/// Math utilities.
pub mod math;

/// Commonly imported/prelude modules.
pub mod prelude {
    use super::*;

    pub use {
        bevy_mesh::BevyMeshBuilder,
        bevy_plugin::{
            components::cad::*, components::camera::*, components::wire_frame::*, events::cad::*,
            plugins::*,
        },
        pmetra_core::{builders::*, centroid::CadCentroid, dimensions::*, meshing::*},
        constants::*,
    };
}

/// Re-exported library modules. (incl truck modules).
pub mod re_exports {
    pub use {bevy_mod_picking, truck_meshalgo, truck_modeling, truck_shapeops, truck_topology};
}
