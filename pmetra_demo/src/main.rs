use bevy::prelude::*;
use pmetra_demo::PmetraDemoPlugin;

fn main() {
    App::new() // App
        .add_plugins(PmetraDemoPlugin)
        .run();
}
