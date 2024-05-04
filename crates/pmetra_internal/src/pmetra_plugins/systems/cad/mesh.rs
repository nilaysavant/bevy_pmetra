use bevy::{prelude::*, utils::HashSet};
use bevy_mod_picking::{pointer::InputPress, prelude::*};

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
    cad_meshes: Query<&BelongsToCadGeneratedRoot, With<CadGeneratedMesh>>,
    mut pointer_down: EventReader<Pointer<Down>>,
    mut presses: EventReader<InputPress>,
    mut selections: EventReader<Pointer<Select>>,
) {
    // Check for selections, if a mesh is selected, set the root to selected.
    for selection_event in selections.read() {
        if let Ok(&BelongsToCadGeneratedRoot(root_ent)) = cad_meshes.get(selection_event.target) {
            let Ok((_, mut root_selection_state)) = cad_generated.get_mut(root_ent) else {
                continue;
            };
            *root_selection_state = CadGeneratedRootSelectionState::Selected;
        }
    }

    // Following is borrowed from `bevy_mod_picking`: https://github.com/aevyrie/bevy_mod_picking/blob/0af5d0c80cd027c74373e74bbfe143119f791c06/crates/bevy_picking_selection/src/lib.rs#L155-L214
    // Used to de-select all root entities if a pointer has clicked on nothing...

    // Pointers that have clicked on something.
    let mut pointer_down_list = HashSet::new();

    for Pointer {
        pointer_id,
        pointer_location,
        target,
        event: _,
    } in pointer_down
        .read()
        .filter(|pointer| pointer.event.button == PointerButton::Primary)
    {
        pointer_down_list.insert(pointer_id);
    }
    // If a pointer has pressed, but did not press on anything, this means it clicked on nothing. If
    // so, and the setting is enabled, deselect everything.
    for press in presses
        .read()
        .filter(|p| p.is_just_down(PointerButton::Primary))
    {
        let id = press.pointer_id;
        if !pointer_down_list.contains(&id) {
            for (root_ent, mut root_selection_state) in cad_generated.iter_mut() {
                *root_selection_state = CadGeneratedRootSelectionState::None;
            }
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
