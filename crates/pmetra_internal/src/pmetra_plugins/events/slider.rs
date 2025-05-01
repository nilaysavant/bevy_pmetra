use bevy::{picking::backend::HitData, prelude::*};

/// Used to Move slider on **drag plane** [`Pointer<Move>`].
#[derive(Event)]
pub struct TransformSliderEvent {
    pub target: Entity,
    pub hit: HitData,
}

impl From<Pointer<Move>> for TransformSliderEvent {
    fn from(event: Pointer<Move>) -> Self {
        TransformSliderEvent {
            target: event.target,
            hit: event.hit.clone(),
        }
    }
}

/// Dispatched on **Slider** : [`Pointer<Move>`].
#[derive(Event)]
pub struct SliderPointerMoveEvent {
    pub target: Entity,
    pub hit: HitData,
}

impl From<Pointer<Move>> for SliderPointerMoveEvent {
    fn from(event: Pointer<Move>) -> Self {
        SliderPointerMoveEvent {
            target: event.target,
            hit: event.hit.clone(),
        }
    }
}

/// Dispatched on **Slider** : [`Pointer<Out>`].
#[derive(Event)]
pub struct SliderPointerOutEvent {
    pub target: Entity,
    pub hit: HitData,
}

impl From<Pointer<Out>> for SliderPointerOutEvent {
    fn from(event: Pointer<Out>) -> Self {
        SliderPointerOutEvent {
            target: event.target,
            hit: event.hit.clone(),
        }
    }
}
