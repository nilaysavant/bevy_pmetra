use bevy::{platform::collections::HashSet, prelude::*};

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

/// Used to de-select all root entities if a pointer has clicked on nothing.
///
/// Checks if the pointer is down on a window, and if so, de-selects all root entities.
/// Since this means that pointer did not click on any entity.
pub fn deselect_all_root_if_clicked_outside(
    mut cad_generated: Query<(Entity, &mut CadGeneratedRootSelectionState), With<CadGeneratedRoot>>,
    mut pointer_down: EventReader<Pointer<Pressed>>,
    windows: Query<Entity, With<Window>>,
) {
    // Pointers that have clicked on something.
    let mut pointer_down_targets = HashSet::new();

    for Pointer { target, .. } in pointer_down
        .read()
        .filter(|pointer| pointer.event.button == PointerButton::Primary)
    {
        pointer_down_targets.insert(target);
    }

    for window in windows {
        if !pointer_down_targets.contains(&window) {
            // If the pointer is not down on a window, continue since it is down on a valid entity.
            continue;
        }
        // If the pointer is down on a window, then deselect all root entities.
        for (_root_ent, mut root_selection_state) in cad_generated.iter_mut() {
            *root_selection_state = CadGeneratedRootSelectionState::None;
        }
    }
}
