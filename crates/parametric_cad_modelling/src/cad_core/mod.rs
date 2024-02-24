/// Wrapper structs/extension used by builders.
pub mod builders;
/// Allows centroid calc for [`truck`] primitives.
pub mod centroid;
/// Dimension extensions/conversion.
pub mod dimensions;
/// Extensions of truck primitives.
pub mod extensions;
/// Traits for meshing primitives into [`bevy::prelude::Mesh`] via [`PolygonMesh`].
pub mod meshing;
/// Custom Tessellation adapted from [`truck_meshalgo::tessellation`].
pub mod tessellation;
