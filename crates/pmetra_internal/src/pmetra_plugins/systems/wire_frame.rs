use bevy::{pbr::wireframe::Wireframe, prelude::*};

use crate::pmetra_plugins::components::wire_frame::WireFrameDisplaySettings;

pub fn control_wire_frame_display(
    mut commands: Commands,
    query: Query<(Entity, &WireFrameDisplaySettings), Changed<WireFrameDisplaySettings>>,
) {
    for (entity, settings) in query.iter() {
        if settings.0 {
            commands.entity(entity).insert(Wireframe);
        } else {
            commands.entity(entity).remove::<Wireframe>();
        }
    }
}
