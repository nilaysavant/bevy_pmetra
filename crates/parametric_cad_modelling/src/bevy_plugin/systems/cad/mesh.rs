use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::{
    bevy_plugin::components::cad::{
        CadGeneratedMesh,
        CadGeneratedMeshOutlinesState,
    },
    prelude::{BelongsToCadGeneratedRoot, CadGeneratedRoot, CadGeneratedRootSelectionState},
};

pub fn mesh_pointer_move(
    pointer_event: Listener<Pointer<Move>>,
    mut cad_meshes: Query<
        (Entity, &PickSelection, &mut CadGeneratedMeshOutlinesState),
        With<CadGeneratedMesh>,
    >,
) {
    let cad_mesh_ent = pointer_event.listener();
    let Ok((_, mesh_selection, mut outlines_state)) = cad_meshes.get_mut(cad_mesh_ent) else {
        return;
    };
    if mesh_selection.is_selected {
        *outlines_state = CadGeneratedMeshOutlinesState::Visible;
    } else {
        *outlines_state = CadGeneratedMeshOutlinesState::SlightlyVisible;
    }
}

pub fn mesh_pointer_out(
    pointer_event: Listener<Pointer<Out>>,
    mut cad_meshes: Query<
        (Entity, &PickSelection, &mut CadGeneratedMeshOutlinesState),
        With<CadGeneratedMesh>,
    >,
) {
    let cad_mesh_ent = pointer_event.listener();
    let Ok((_, mesh_selection, mut outlines_state)) = cad_meshes.get_mut(cad_mesh_ent) else {
        return;
    };
    if mesh_selection.is_selected {
        *outlines_state = CadGeneratedMeshOutlinesState::Visible;
    } else {
        *outlines_state = CadGeneratedMeshOutlinesState::default();
    }
}

pub fn handle_mesh_selection(
    mut cad_meshes: Query<
        (
            Entity,
            &PickSelection,
            &mut CadGeneratedMeshOutlinesState,
            &BelongsToCadGeneratedRoot,
        ),
        (With<CadGeneratedMesh>, Changed<PickSelection>),
    >,
) {
    for (cad_mesh_ent, selection, mut outlines_state, &BelongsToCadGeneratedRoot(root_ent)) in
        cad_meshes.iter_mut()
    {
        if selection.is_selected {
            *outlines_state = CadGeneratedMeshOutlinesState::Visible;
        } else {
            *outlines_state = CadGeneratedMeshOutlinesState::default();
        }
    }
}

pub fn update_root_selection_based_on_mesh_selection(
    mut cad_generated: Query<(Entity, &mut CadGeneratedRootSelectionState), With<CadGeneratedRoot>>,
    cad_meshes: Query<(Entity, &PickSelection, &BelongsToCadGeneratedRoot), With<CadGeneratedMesh>>,
) {
    for (root_ent, mut root_selection) in cad_generated.iter_mut() {
        let any_mesh_selected =
            cad_meshes
                .iter()
                .any(|(_, selection, &BelongsToCadGeneratedRoot(cur_root_ent))| {
                    cur_root_ent == root_ent && selection.is_selected
                });
        if any_mesh_selected {
            *root_selection = CadGeneratedRootSelectionState::Selected;
        } else {
            *root_selection = CadGeneratedRootSelectionState::None;
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
