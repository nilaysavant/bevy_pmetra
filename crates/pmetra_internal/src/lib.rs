#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::let_and_return)]

/// [`bevy`] meshing utils.
pub mod bevy_mesh;
/// Constants.
pub mod constants;
/// Math utilities.
pub mod math;
/// Core Data Structures/Traits/Extensions/Types.
pub mod pmetra_core;
/// Plugin(s) for integrating with [`bevy`].
pub mod pmetra_plugins;

/// Commonly imported/prelude modules.
pub mod prelude {
    use super::*;

    pub use {
        bevy_mesh::BevyMeshBuilder,
        constants::*,
        pmetra_core::{builders::*, centroid::CadCentroid, dimensions::*, meshing::*},
        pmetra_plugins::{
            components::cad::*, components::camera::*, components::wire_frame::*, events::cad::*,
            plugins::*, resources::PmetraGlobalSettings,
        },
    };
}

/// Re-exported library modules. (incl truck modules).
pub mod re_exports {
    pub use {bevy_mod_picking, truck_meshalgo, truck_modeling, truck_shapeops, truck_topology};
}
