use bevy::{picking::pointer::PointerInput, platform::collections::HashSet, prelude::*};

use crate::pmetra_plugins::components::cad::{CadGeneratedRoot, CadGeneratedRootSelectionState};

pub fn root_pointer_move(
    pointer_event: Trigger<Pointer<Move>>,
    mut cad_generated: Query<&mut CadGeneratedRootSelectionState, With<CadGeneratedRoot>>,
) {
    let root_ent = pointer_event.target();
    let Ok(mut root_selection_state) = cad_generated.get_mut(root_ent) else {
        return;
    };
    if let CadGeneratedRootSelectionState::None = *root_selection_state {
        *root_selection_state = CadGeneratedRootSelectionState::Hovered;
    }
}

pub fn root_pointer_out(
    pointer_event: Trigger<Pointer<Out>>,
    mut cad_generated: Query<&mut CadGeneratedRootSelectionState, With<CadGeneratedRoot>>,
) {
    let root_ent = pointer_event.target();
    let Ok(mut root_selection_state) = cad_generated.get_mut(root_ent) else {
        return;
    };
    if let CadGeneratedRootSelectionState::Hovered = *root_selection_state {
        *root_selection_state = CadGeneratedRootSelectionState::None;
    }
}

pub fn root_on_click(
    click_event: Trigger<Pointer<Click>>,
    mut cad_generated: Query<(Entity, &mut CadGeneratedRootSelectionState), With<CadGeneratedRoot>>,
) {
    if click_event.button != PointerButton::Primary {
        return;
    }
    let selected_root_ent = click_event.target();
    for (root_ent, mut root_selection_state) in cad_generated.iter_mut() {
        if root_ent == selected_root_ent {
            *root_selection_state = CadGeneratedRootSelectionState::Selected;
        } else if !matches!(
            *root_selection_state,
            CadGeneratedRootSelectionState::Hovered
        ) {
            *root_selection_state = CadGeneratedRootSelectionState::None;
        }
    }
}

pub fn deselect_all_root_if_clicked_outside(
    mut cad_generated: Query<(Entity, &mut CadGeneratedRootSelectionState), With<CadGeneratedRoot>>,
    mut pointer_down: EventReader<Pointer<Pressed>>,
    mut presses: EventReader<PointerInput>,
) {
    // Following is borrowed from `bevy_mod_picking`: https://github.com/aevyrie/bevy_mod_picking/blob/0af5d0c80cd027c74373e74bbfe143119f791c06/crates/bevy_picking_selection/src/lib.rs#L155-L214
    // Used to de-select all root entities if a pointer has clicked on nothing...

    // Pointers that have clicked on something.
    let mut pointer_down_list = HashSet::new();

    for Pointer { pointer_id, .. } in pointer_down
        .read()
        .filter(|pointer| pointer.event.button == PointerButton::Primary)
    {
        pointer_down_list.insert(pointer_id);
    }
    // If a pointer has pressed, but did not press on anything, this means it clicked on nothing. If
    // so, and the setting is enabled, deselect everything.
    for press in presses
        .read()
        .filter(|p| p.button_just_pressed(PointerButton::Primary))
    {
        let id = press.pointer_id;
        if !pointer_down_list.contains(&id) {
            for (_root_ent, mut root_selection_state) in cad_generated.iter_mut() {
                *root_selection_state = CadGeneratedRootSelectionState::None;
            }
        }
    }
}
