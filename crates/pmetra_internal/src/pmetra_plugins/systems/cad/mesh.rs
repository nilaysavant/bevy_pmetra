use bevy::{color::palettes::css, prelude::*};
use bevy_mod_picking::prelude::*;

use crate::pmetra_plugins::components::cad::CadGeneratedMesh;

pub fn show_mesh_local_debug_axis(
    cad_meshes: Query<(&PickSelection, &Transform), With<CadGeneratedMesh>>,
    mut gizmos: Gizmos,
) {
    for (selection, transform) in cad_meshes.iter() {
        if !selection.is_selected {
            continue;
        }
        // x
        gizmos.arrow(
            transform.translation,
            transform.translation + *transform.local_x(),
            css::RED,
        );
        // y
        gizmos.arrow(
            transform.translation,
            transform.translation + *transform.local_y(),
            css::GREEN,
        );
        // z
        gizmos.arrow(
            transform.translation,
            transform.translation + *transform.local_z(),
            css::BLUE,
        );
    }
}
