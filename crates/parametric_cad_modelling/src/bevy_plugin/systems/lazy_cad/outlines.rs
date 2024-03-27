use bevy::prelude::*;

use crate::bevy_plugin::components::cad::{
    CadGeneratedMesh, CadGeneratedMeshOutlines, CadGeneratedMeshOutlinesState,
};

pub fn generate_mesh_outlines(
    cad_meshes: Query<
        (
            Entity,
            &Transform,
            &CadGeneratedMeshOutlinesState,
            &CadGeneratedMeshOutlines,
        ),
        With<CadGeneratedMesh>,
    >,
    mut gizmos: Gizmos,
) {
    for (cad_mesh_ent, transform, outlines_state, CadGeneratedMeshOutlines(line_strip_positions)) in
        cad_meshes.iter()
    {
        let color = match *outlines_state {
            CadGeneratedMeshOutlinesState::Invisible => Color::NONE,
            CadGeneratedMeshOutlinesState::SlightlyVisible => Color::WHITE.with_a(0.6),
            CadGeneratedMeshOutlinesState::Visible => Color::WHITE,
        };

        for positions in line_strip_positions.iter() {
            let mut positions = positions.clone();
            positions.iter_mut().for_each(|p| {
                *p = transform.transform_point(*p);
            });
            if positions.len() > 2 {
                positions.push(*positions.first().unwrap());
            }
            gizmos.linestrip(positions, color);
        }
    }
}
