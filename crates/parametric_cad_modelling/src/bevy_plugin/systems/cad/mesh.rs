use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::bevy_plugin::components::cad::{
    BelongsToCadGeneratedMesh, CadGeneratedCursor, CadGeneratedMesh, CadGeneratedMeshOutlinesState,
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
        (Entity, &PickSelection, &mut CadGeneratedMeshOutlinesState),
        (With<CadGeneratedMesh>, Changed<PickSelection>),
    >,
    mut cad_cursors: Query<(&BelongsToCadGeneratedMesh, &mut Visibility), With<CadGeneratedCursor>>,
) {
    for (cad_mesh_ent, selection, mut outlines_state) in cad_meshes.iter_mut() {
        for (BelongsToCadGeneratedMesh(cad_mesh_ent_cur), mut visibility) in cad_cursors.iter_mut()
        {
            if *cad_mesh_ent_cur != cad_mesh_ent {
                continue;
            }
            // if belongs to selected mesh set to visible else hide cursors...
            if selection.is_selected {
                *visibility = Visibility::Visible;
                *outlines_state = CadGeneratedMeshOutlinesState::Visible;
            } else {
                *visibility = Visibility::Hidden;
                *outlines_state = CadGeneratedMeshOutlinesState::default();
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
            transform.translation + transform.local_x(),
            Color::RED,
        );
        // y
        gizmos.line(
            transform.translation,
            transform.translation + transform.local_y(),
            Color::GREEN,
        );
        // z
        gizmos.line(
            transform.translation,
            transform.translation + transform.local_z(),
            Color::BLUE,
        );
    }
}
