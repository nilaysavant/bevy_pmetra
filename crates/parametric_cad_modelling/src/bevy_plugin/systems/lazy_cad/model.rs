use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::{
    backends::raycast::bevy_mod_raycast::markers::NoBackfaceCulling, prelude::*, PickableBundle,
};

use crate::{
    bevy_plugin::{
        components::cad::CadGeneratedRoot,
        events::lazy_cad::GenerateLazyCadModel,
        systems::cad::mesh::{mesh_pointer_move, mesh_pointer_out},
    },
    cad_core::{
        builders::{CadMeshName, CadShell},
        lazy_builders::{
            CadLazyMesh, CadMeshLazyBuilder, CadShellLazyBuilder, CadShellName, ParametricLazyCad,
        },
    },
    prelude::{
        BelongsToCadGeneratedRoot, CadGeneratedMesh, CadGeneratedMeshOutlines,
        CadGeneratedMeshOutlinesState, WireFrameDisplaySettings,
    },
};

pub fn spawn_shells_lazy_builders_on_generate<Params: ParametricLazyCad + Component + Clone>(
    mut commands: Commands,
    mut events: EventReader<GenerateLazyCadModel<Params>>,
    cad_generated: Query<Entity, (With<Params>, With<CadGeneratedRoot>)>,
) {
    for GenerateLazyCadModel {
        params,
        remove_existing_models,
    } in events.read()
    {
        if *remove_existing_models {
            for entity in cad_generated.iter() {
                let Some(ent_commands) = commands.get_entity(entity) else {
                    continue;
                };
                ent_commands.despawn_recursive();
            }
        }

        // Spawn root...
        let root = commands
            .spawn((SpatialBundle::default(), CadGeneratedRoot, params.clone()))
            .id();

        // Get the shell builders from params...
        let shells_lazy_builders = match params.shells_builders() {
            Ok(result) => result,
            Err(e) => {
                error!("shells_builders failed with error: {:?}", e);
                return;
            }
        };

        // Spawn shell builder entities for parallel shell building in later systems...
        for (shell_name, shell_builder) in shells_lazy_builders.builders.iter() {
            commands.spawn((
                shell_name.clone(),
                shell_builder.clone(),
                BelongsToCadGeneratedRoot(root),
            ));
        }
    }
}

pub fn build_shells_from_builders<Params: ParametricLazyCad + Component + Clone>(
    mut commands: Commands,
    shell_builders: Query<
        (Entity, &CadShellName, &CadShellLazyBuilder<Params>),
        (Without<CadShell>, Changed<CadShellLazyBuilder<Params>>),
    >,
) {
    for (entity, name, builder) in shell_builders.iter() {
        let shell = match builder.build_cad_shell() {
            Ok(shell) => shell,
            Err(e) => {
                error!(
                    "build_cad_shell for shell_name: {:?} failed, error: {:?}",
                    name, e
                );
                continue;
            }
        };

        commands.entity(entity).insert(shell);
    }
}

pub fn shells_to_mesh_builder<Params: ParametricLazyCad + Component + Clone>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    shell_builders: Query<
        (
            Entity,
            &CadShellName,
            &CadShellLazyBuilder<Params>,
            &CadShell,
            &BelongsToCadGeneratedRoot,
        ),
        Changed<CadShellLazyBuilder<Params>>,
    >,
) {
    for (entity, shell_name, shell_builder, shell, belongs_to_root) in shell_builders.iter() {
        let meshes_builder = match shell_builder
            .params
            .meshes_builders_by_shell(shell_name.clone(), shell.clone())
        {
            Ok(meshes_builder) => meshes_builder,
            Err(e) => {
                error!(
                    "meshes_builders_by_shell for shell_name: {:?} failed, error: {:?}",
                    shell_name, e
                );
                continue;
            }
        };
        let Ok(bevy_mesh) = meshes_builder.build_bevy_mesh() else {
            continue;
        };
        let mesh_hdl = meshes.add(bevy_mesh);

        for (mesh_name, mut mesh_builder) in meshes_builder.mesh_builders {
            let Ok(mesh_builder) = mesh_builder.set_mesh_hdl(mesh_hdl.clone()) else {
                continue;
            };
            commands.spawn((
                shell.clone(),
                mesh_name,
                mesh_builder,
                belongs_to_root.clone(),
            ));
        }

        // De-spawn shell builder...
        let Some(ent_commands) = commands.get_entity(entity) else {
            continue;
        };
        ent_commands.despawn_recursive();
    }
}

pub fn mesh_builder_to_bundle<Params: ParametricLazyCad + Component + Clone>(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_builders: Query<
        (
            Entity,
            &CadShellName,
            &CadShell,
            &CadMeshName,
            &CadMeshLazyBuilder<Params>,
            &BelongsToCadGeneratedRoot,
        ),
        Changed<CadMeshLazyBuilder<Params>>,
    >,
) {
    for (entity, shell_name, shell, mesh_name, mesh_builder, belongs_to_root) in
        mesh_builders.iter()
    {
        let cad_mesh = match mesh_builder.build() {
            Ok(cad_mesh) => cad_mesh,
            Err(e) => {
                error!(
                    "Failed to build cad_mesh with name {:?} with error: {:?}",
                    mesh_name, e
                );
                continue;
            }
        };

        let CadLazyMesh {
            mesh_hdl,
            base_material,
            transform,
            outlines,
            cursors,
        } = cad_mesh;

        let material_hdl = materials.add(base_material);

        let mesh_entity = commands
            .entity(entity)
            .insert(PickableBundle::default())
            .insert(MaterialMeshBundle {
                material: material_hdl.clone(),
                mesh: mesh_hdl,
                transform,
                ..Default::default()
            })
            .insert((
                Name::new(mesh_name.0.clone()),
                CadGeneratedMesh,
                belongs_to_root.clone(),
                CadGeneratedMeshOutlines(outlines.clone()),
                CadGeneratedMeshOutlinesState::default(),
                WireFrameDisplaySettings::default(),
                // picking...
                PickableBundle::default(), // <- Makes the mesh pickable.
                // Pickable::IGNORE,
                // Disable highlight...
                Highlight::<StandardMaterial> {
                    hovered: Some(HighlightKind::Fixed(material_hdl.clone())),
                    pressed: Some(HighlightKind::Fixed(material_hdl.clone())),
                    selected: Some(HighlightKind::Fixed(material_hdl.clone())),
                },
                // Add drag plane on drag start...
                On::<Pointer<Move>>::run(mesh_pointer_move),
                On::<Pointer<Out>>::run(mesh_pointer_out),
            ))
            .id();

        let Some(mut root_ent_commands) = commands.get_entity(belongs_to_root.0) else {
            warn!("Could not get commands for entity: {:?}", belongs_to_root.0);
            continue;
        };
        root_ent_commands.push_children(&[mesh_entity]);
    }
}
