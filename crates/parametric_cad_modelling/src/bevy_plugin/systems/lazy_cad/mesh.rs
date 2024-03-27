use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::{
    bevy_plugin::components::cad::{
        BelongsToCadGeneratedMesh, CadGeneratedCursor, CadGeneratedMesh,
        CadGeneratedMeshOutlinesState,
    },
    prelude::BelongsToCadGeneratedRoot,
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
    mut cad_cursors: Query<(&BelongsToCadGeneratedRoot, &mut Visibility), With<CadGeneratedCursor>>,
) {
    for (cad_mesh_ent, selection, mut outlines_state, &BelongsToCadGeneratedRoot(root_ent)) in
        cad_meshes.iter_mut()
    {
        if selection.is_selected {
            *outlines_state = CadGeneratedMeshOutlinesState::Visible;
        } else {
            *outlines_state = CadGeneratedMeshOutlinesState::default();
        }
        for (&BelongsToCadGeneratedRoot(cur_root_ent), mut visibility) in cad_cursors.iter_mut() {
            if cur_root_ent != root_ent {
                continue;
            }
            // if belongs to same root as mesh set to visible else hide cursors...
            if selection.is_selected {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
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
