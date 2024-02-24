use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_rapier3d::prelude::*;
use bevy_pmetra::prelude::*;

use crate::{
    resources::{CadGeneratedModelParamsId, CadGeneratedModelSpawner},
    utils::cad_models::{
        mechanical_parts::simple_gear::SimpleGear, simple_primitives::SimpleCubeAtCylinder,
        space_station::{round_cabin_segment::CadMaterialIds, RoundCabinSegment},
    },
};

pub fn spawn_cad_model(
    mut commands: Commands,
    cad_model_spawner: Res<CadGeneratedModelSpawner>,
    cad_models: Query<Entity, With<CadGeneratedRoot>>,
    mut spawn_simple_cube_at_cylinder: EventWriter<GenerateCadModel<SimpleCubeAtCylinder>>,
    mut spawn_round_cabin_segment: EventWriter<GenerateCadModel<RoundCabinSegment>>,
    mut spawn_simple_gear: EventWriter<GenerateCadModel<SimpleGear>>,
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
            spawn_simple_cube_at_cylinder.send(GenerateCadModel::default());
        }
        CadGeneratedModelParamsId::RoundCabinSegment => {
            let mut cad_material_textures = CadMaterialTextures::default();
            // cad_material_textures.insert(
            //     CadMaterialIds::Base.to_string().into(),
            //     CadMaterialTextureSet {
            //         base_color_texture: Some(
            //             asset_server.load("textures/Prototype_Grid_Teal_10-512x512.png"),
            //         ),
            //         ..Default::default()
            //     },
            // );
            cad_material_textures.insert(
                CadMaterialIds::Roof.to_string().into(),
                CadMaterialTextureSet {
                    base_color_texture: Some(
                        asset_server.load("textures/terrazzo_11_basecolor-1K.png"),
                    ),
                    normal_map_texture: Some(
                        asset_server.load("textures/terrazzo_11_normal-1K.png"),
                    ),
                    metallic_roughness_texture: Some(
                        asset_server.load("textures/terrazzo_11_roughness-1K.png"),
                    ),
                    ..Default::default()
                },
            );
            spawn_round_cabin_segment.send(GenerateCadModel {
                textures: cad_material_textures,
                ..Default::default()
            });
        }
        CadGeneratedModelParamsId::SimpleGear => {
            spawn_simple_gear.send(GenerateCadModel::default());
        }
    }
}

pub fn add_collider_to_generated_cad_model<Params: ParametricCad + Component>(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    cad_roots: Query<Entity, (With<CadGeneratedRoot>, Changed<Params>)>,
    cad_meshes: Query<
        (
            Entity,
            &BelongsToCadGeneratedRoot,
            &CadMeshName,
            &Handle<Mesh>,
        ),
        With<CadGeneratedMesh>,
    >,
) {
    for cad_root_ent in cad_roots.iter() {
        for (cad_mesh_ent, BelongsToCadGeneratedRoot(cad_root_ent_cur), cad_mesh_name, mesh_hdl) in
            cad_meshes.iter()
        {
            if *cad_root_ent_cur != cad_root_ent {
                continue;
            }
            let Some(mesh) = meshes.get(mesh_hdl) else {
                continue;
            };
            let Some(collider) = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh)
            else {
                error!("Could not generated collider for {}!", **cad_mesh_name);
                continue;
            };
            commands
                .entity(cad_mesh_ent)
                .insert((RigidBody::Fixed, collider));
        }
    }
}
