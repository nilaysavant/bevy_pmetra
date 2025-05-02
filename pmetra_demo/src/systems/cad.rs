use bevy::prelude::*;
use bevy_pmetra::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    resources::{CadGeneratedModelParamsId, CadGeneratedModelSpawner},
    utils::cad_models::{
        simple_primitives::simple_cube_at_cylinder::SimpleCubeAtCylinder,
        space_station::{round_cabin_segment::RoundCabinSegment, tower_extension::TowerExtension},
    },
};

pub fn spawn_cad_model(
    mut commands: Commands,
    cad_model_spawner: Res<CadGeneratedModelSpawner>,
    cad_models: Query<Entity, With<CadGeneratedRoot>>,
    mut spawn_simple_cube_at_cylinder: EventWriter<GenerateCadModel<SimpleCubeAtCylinder>>,
    mut tower_extension: EventWriter<GenerateCadModel<TowerExtension>>,
    mut round_cabin_segment: EventWriter<GenerateCadModel<RoundCabinSegment>>,
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
        CadGeneratedModelParamsId::SimplCubeAtCylinder => {
            spawn_simple_cube_at_cylinder.send(GenerateCadModel::default());
        }
        CadGeneratedModelParamsId::TowerExtension => {
            tower_extension.send(GenerateCadModel::default());
        }
        CadGeneratedModelParamsId::RoundCabinSegment => {
            round_cabin_segment.send(GenerateCadModel::default());
        }
        CadGeneratedModelParamsId::MultiModelsSimplCubeAtCylinderAndTowerExtension => {
            spawn_simple_cube_at_cylinder.send(GenerateCadModel::default());
            tower_extension.send(GenerateCadModel {
                remove_existing_models: false,
                ..Default::default()
            });
        }
        CadGeneratedModelParamsId::MultiModels2TowerExtensions => {
            tower_extension.send(GenerateCadModel::default());
            tower_extension.send(GenerateCadModel {
                transform: Transform::from_translation(Vec3::X * 1.),
                remove_existing_models: false,
                ..Default::default()
            });
        }
    }
}

pub fn add_collider_to_generated_cad_model(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    cad_meshes: Query<
        (Entity, &BelongsToCadGeneratedRoot, &CadMeshName, &Mesh3d),
        (With<CadGeneratedMesh>, Changed<Mesh3d>),
    >,
) {
    for (cad_mesh_ent, BelongsToCadGeneratedRoot(_cad_root_ent_cur), cad_mesh_name, mesh_hdl) in
        cad_meshes.iter()
    {
        let Some(mesh) = meshes.get(mesh_hdl) else {
            continue;
        };
        let Some(collider) = Collider::from_bevy_mesh(
            mesh,
            &ComputedColliderShape::TriMesh(TriMeshFlags::default()),
        ) else {
            error!("Could not generated collider for {}!", **cad_mesh_name);
            continue;
        };
        commands
            .entity(cad_mesh_ent)
            .insert((RigidBody::Fixed, collider));
    }
}
