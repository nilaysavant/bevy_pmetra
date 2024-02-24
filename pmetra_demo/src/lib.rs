#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::let_and_return)]
#![allow(clippy::field_reassign_with_default)]
pub mod components;
pub mod plugin;
pub mod plugins;
pub mod resources;
pub mod systems;
pub mod utils;

pub use plugin::PmetraDemoPlugin;
