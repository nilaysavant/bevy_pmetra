use bevy::prelude::*;

use crate::{
    pmetra_plugins::{
        components::cad::{
            CadGeneratedMesh, CadGeneratedMeshOutlines, CadGeneratedRoot,
            CadGeneratedRootSelectionState,
        },
        systems::gizmos::PmetraMeshOutlineGizmos,
    },
    prelude::BelongsToCadGeneratedRoot,
};

pub fn render_mesh_outlines(
    cad_generated: Query<(Entity, &CadGeneratedRootSelectionState), With<CadGeneratedRoot>>,
    cad_meshes: Query<
        (
            Entity,
            &BelongsToCadGeneratedRoot,
            &GlobalTransform,
            &CadGeneratedMeshOutlines,
        ),
        With<CadGeneratedMesh>,
    >,
    mut gizmos: Gizmos<PmetraMeshOutlineGizmos>,
) {
    for (cur_root_ent, root_selection_state) in cad_generated.iter() {
        for (
            _cad_mesh_ent,
            &BelongsToCadGeneratedRoot(root_ent),
            glob_transform,
            CadGeneratedMeshOutlines(line_strip_positions),
        ) in cad_meshes.iter()
        {
            if root_ent != cur_root_ent {
                continue;
            }
            let color = match root_selection_state {
                CadGeneratedRootSelectionState::None => Color::NONE,
                CadGeneratedRootSelectionState::Hovered => Color::WHITE.with_alpha(0.6),
                CadGeneratedRootSelectionState::Selected => Color::WHITE,
            };

            for positions in line_strip_positions.iter() {
                let mut positions = positions.clone();
                positions.iter_mut().for_each(|p| {
                    *p = glob_transform.transform_point(*p);
                });
                if positions.len() > 2 {
                    positions.push(*positions.first().unwrap());
                }
                gizmos.linestrip(positions, color);
            }
        }
    }
}
