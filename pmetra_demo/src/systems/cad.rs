use bevy::prelude::*;
use bevy_pmetra::{bevy_plugin::events::cad::GenerateCadModel, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{
    resources::{CadGeneratedModelParamsId, CadGeneratedModelSpawner},
    utils::cad_models::{
        simple_primitives::simple_lazy_cube_at_cylinder::SimpleLazyCubeAtCylinder,
        space_station::{
            lazy_round_cabin_segment::LazyRoundCabinSegment,
            lazy_tower_extension::LazyTowerExtension,
        },
    },
};

pub fn spawn_cad_model(
    mut commands: Commands,
    cad_model_spawner: Res<CadGeneratedModelSpawner>,
    cad_models: Query<Entity, With<CadGeneratedRoot>>,
    mut spawn_lazy_simple_cube_at_cylinder: EventWriter<
        GenerateCadModel<SimpleLazyCubeAtCylinder>,
    >,
    mut lazy_tower_extension: EventWriter<GenerateCadModel<LazyTowerExtension>>,
    mut lazy_round_cabin_segment: EventWriter<GenerateCadModel<LazyRoundCabinSegment>>,
) {
    if !cad_model_spawner.is_changed() {
        return;
    }
    // delete all existing models...
    for entity in cad_models.iter() {
        let Some(ent_commands) = commands.get_entity(entity) else {
            continue;
        };
        ent_commands.despawn_recursive();
    }
    // fire event to spawn new model...
    match cad_model_spawner.selected_params {
        CadGeneratedModelParamsId::SimplLazyCubeAtCylinder => {
            spawn_lazy_simple_cube_at_cylinder.send(GenerateCadModel::default());
        }
        CadGeneratedModelParamsId::LazyTowerExtension => {
            lazy_tower_extension.send(GenerateCadModel::default());
        }
        CadGeneratedModelParamsId::LazyRoundCabinSegment => {
            lazy_round_cabin_segment.send(GenerateCadModel::default());
        }
    }
}

pub fn add_collider_to_generated_cad_model(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    cad_meshes: Query<
        (
            Entity,
            &BelongsToCadGeneratedRoot,
            &CadMeshName,
            &Handle<Mesh>,
        ),
        (With<CadGeneratedMesh>, Changed<Handle<Mesh>>),
    >,
) {
    for (cad_mesh_ent, BelongsToCadGeneratedRoot(cad_root_ent_cur), cad_mesh_name, mesh_hdl) in
        cad_meshes.iter()
    {
        let Some(mesh) = meshes.get(mesh_hdl) else {
            continue;
        };
        let Some(collider) = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh) else {
            error!("Could not generated collider for {}!", **cad_mesh_name);
            continue;
        };
        commands
            .entity(cad_mesh_ent)
            .insert((RigidBody::Fixed, collider));
    }
}
