use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::pmetra_plugins::components::cad::{
    BelongsToCadGeneratedRoot, CadGeneratedMesh, CadGeneratedRoot, CadGeneratedRootSelectionState,
};

pub fn mesh_pointer_move(
    pointer_event: Listener<Pointer<Move>>,
    mut cad_generated: Query<&mut CadGeneratedRootSelectionState, With<CadGeneratedRoot>>,
    mut cad_meshes: Query<&BelongsToCadGeneratedRoot, With<CadGeneratedMesh>>,
) {
    let cad_mesh_ent = pointer_event.listener();
    let Ok(&BelongsToCadGeneratedRoot(root_ent)) = cad_meshes.get_mut(cad_mesh_ent) else {
        return;
    };
    let Ok(mut root_selection_state) = cad_generated.get_mut(root_ent) else {
        return;
    };
    if let CadGeneratedRootSelectionState::None = *root_selection_state {
        *root_selection_state = CadGeneratedRootSelectionState::Hovered;
    }
}

pub fn mesh_pointer_out(
    pointer_event: Listener<Pointer<Out>>,
    mut cad_generated: Query<&mut CadGeneratedRootSelectionState, With<CadGeneratedRoot>>,
    cad_meshes: Query<&BelongsToCadGeneratedRoot, With<CadGeneratedMesh>>,
) {
    let cad_mesh_ent = pointer_event.listener();
    let Ok(&BelongsToCadGeneratedRoot(root_ent)) = cad_meshes.get(cad_mesh_ent) else {
        return;
    };
    let Ok(mut root_selection_state) = cad_generated.get_mut(root_ent) else {
        return;
    };
    if let CadGeneratedRootSelectionState::Hovered = *root_selection_state {
        *root_selection_state = CadGeneratedRootSelectionState::None;
    }
}

pub fn update_root_selection_based_on_mesh_selection(
    mut cad_generated: Query<(Entity, &mut CadGeneratedRootSelectionState), With<CadGeneratedRoot>>,
    cad_meshes: Query<(Entity, &PickSelection, &BelongsToCadGeneratedRoot), With<CadGeneratedMesh>>,
) {
    for (root_ent, mut root_selection_state) in cad_generated.iter_mut() {
        let any_mesh_selected =
            cad_meshes
                .iter()
                .any(|(_, selection, &BelongsToCadGeneratedRoot(cur_root_ent))| {
                    cur_root_ent == root_ent && selection.is_selected
                });
        if any_mesh_selected {
            *root_selection_state = CadGeneratedRootSelectionState::Selected;
        } else if !matches!(*root_selection_state, CadGeneratedRootSelectionState::Hovered) {
            *root_selection_state = CadGeneratedRootSelectionState::None;
        }
    }
}

pub fn show_mesh_local_debug_axis(
    cad_meshes: Query<(Entity, &PickSelection, &Transform), With<CadGeneratedMesh>>,
    mut gizmos: Gizmos,
) {
    for (entity, selection, transform) in cad_meshes.iter() {
        if !selection.is_selected {
            continue;
        }
        // x
        gizmos.line(
            transform.translation,
            transform.translation + *transform.local_x(),
            Color::RED,
        );
        // y
        gizmos.line(
            transform.translation,
            transform.translation + *transform.local_y(),
            Color::GREEN,
        );
        // z
        gizmos.line(
            transform.translation,
            transform.translation + *transform.local_z(),
            Color::BLUE,
        );
    }
}
