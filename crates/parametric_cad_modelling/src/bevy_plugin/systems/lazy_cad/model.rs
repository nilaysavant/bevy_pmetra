use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::{
    backends::raycast::bevy_mod_raycast::markers::NoBackfaceCulling, prelude::*, PickableBundle,
};

use crate::{
    bevy_plugin::{components::cad::CadGeneratedRoot, events::lazy_cad::GenerateLazyCadModel},
    cad_core::{
        builders::CadShell,
        lazy_builders::{CadShellLazyBuilder, CadShellName, ParametricLazyCad},
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
        commands.spawn((SpatialBundle::default(), CadGeneratedRoot, params.clone()));

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
            commands.spawn((shell_name.clone(), shell_builder.clone()));
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
        let CadShellLazyBuilder {
            params,
            build_cad_shell,
        } = builder;
        let shell = match build_cad_shell(params) {
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
    shell_builders: Query<
        (
            Entity,
            &CadShellName,
            &CadShellLazyBuilder<Params>,
            &CadShell,
        ),
        Changed<CadShellLazyBuilder<Params>>,
    >,
) {
    // TODO
}
