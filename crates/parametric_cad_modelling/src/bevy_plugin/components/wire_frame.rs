use bevy::prelude::*;

/// Display wire frame settings component.
///
/// `true` : show wire frame on entity.
/// `false` : hide wire frame on entity.
#[derive(Debug, Component, Reflect)]
pub struct WireFrameDisplaySettings(pub bool);

impl Default for WireFrameDisplaySettings {
    fn default() -> Self {
        Self(false)
    }
}
