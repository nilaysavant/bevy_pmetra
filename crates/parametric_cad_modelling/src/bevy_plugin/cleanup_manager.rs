use bevy::prelude::*;

pub struct CleanupManagerPlugin;

impl Plugin for CleanupManagerPlugin {
    fn build(&self, app: &mut App) {
        app // App
            .add_systems(First, cleanup)
            .add_systems(Startup, || info!("CleanupManagerPlugin started..."));
    }
}

/// Component used to mark entities for cleanup.
#[derive(Debug, Component, Clone)]
pub enum Cleanup {
    /// Normal cleanup. Cleans'up/deletes only itself.
    Normal,
    /// Cleanup itself and all descendants recursively.
    Recursive,
    /// Cleanup only descendants.
    Descendants,
}

/// Cleanup routine system.
pub fn cleanup(mut commands: Commands, query: Query<(Entity, &Cleanup)>) {
    for (entity, cleanup) in query.iter() {
        let Some(mut entity_commands) = commands.get_entity(entity) else {
            continue;
        };
        match cleanup {
            Cleanup::Normal => entity_commands.despawn(),
            Cleanup::Recursive => entity_commands.despawn_recursive(),
            Cleanup::Descendants => {
                entity_commands.despawn_descendants();
            }
        }
    }
}
