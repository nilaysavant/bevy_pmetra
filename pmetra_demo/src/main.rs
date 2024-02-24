use bevy::{asset::AssetMetaCheck, prelude::*};
use pmetra_demo::PmetraDemoPlugin;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(PmetraDemoPlugin)
        .run();
}
