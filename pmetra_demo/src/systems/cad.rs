use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_pmetra::{
    bevy_plugin::events::lazy_cad::GenerateLazyCadModel,
    cad_core::lazy_builders::ParametricLazyCad, prelude::*,
};
use bevy_rapier3d::prelude::*;

use crate::{
    resources::{CadGeneratedModelParamsId, CadGeneratedModelSpawner},
    utils::cad_models::{
        mechanical_parts::simple_gear::SimpleGear,
        simple_primitives::{
            simple_lazy_cube_at_cylinder::SimpleLazyCubeAtCylinder, SimpleCubeAtCylinder,
        },
        space_station::{round_cabin_segment::CadMaterialIds, RoundCabinSegment},
    },
};

pub fn spawn_cad_model(
    mut commands: Commands,
    cad_model_spawner: Res<CadGeneratedModelSpawner>,
    cad_models: Query<Entity, With<CadGeneratedRoot>>,
    // mut spawn_simple_cube_at_cylinder: EventWriter<GenerateCadModel<SimpleCubeAtCylinder>>,
    // mut spawn_round_cabin_segment: EventWriter<GenerateCadModel<RoundCabinSegment>>,
    // mut spawn_simple_gear: EventWriter<GenerateCadModel<SimpleGear>>,
    mut spawn_lazy_simple_cube_at_cylinder: EventWriter<
        GenerateLazyCadModel<SimpleLazyCubeAtCylinder>,
    >,
    mut asset_server: ResMut<AssetServer>,
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
        CadGeneratedModelParamsId::SimpleCubeAtCylinder => {
            // spawn_simple_cube_at_cylinder.send(GenerateCadModel::default());
        }
        CadGeneratedModelParamsId::RoundCabinSegment => {
            // spawn_round_cabin_segment.send(GenerateCadModel::default());
        }
        CadGeneratedModelParamsId::SimpleGear => {
            // spawn_simple_gear.send(GenerateCadModel::default());
        }
        CadGeneratedModelParamsId::SimplLazyCubeAtCylinder => {
            spawn_lazy_simple_cube_at_cylinder.send(GenerateLazyCadModel::default());
        }
    }
}

pub fn add_collider_to_generated_cad_model<Params: ParametricLazyCad + Component>(
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
