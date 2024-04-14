use bevy::prelude::*;
use bevy_mod_picking::{
    backend::HitData,
    events::{Move, Pointer},
    prelude::*,
};

/// Used to Move slider on **drag plane** [`Pointer<Move>`].
#[derive(Event)]
pub struct TransformSliderEvent {
    pub target: Entity,
    pub hit: HitData,
}

impl From<ListenerInput<Pointer<Move>>> for TransformSliderEvent {
    fn from(event: ListenerInput<Pointer<Move>>) -> Self {
        TransformSliderEvent {
            target: event.target(),
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

impl From<ListenerInput<Pointer<Move>>> for SliderPointerMoveEvent {
    fn from(event: ListenerInput<Pointer<Move>>) -> Self {
        SliderPointerMoveEvent {
            target: event.target(),
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

impl From<ListenerInput<Pointer<Out>>> for SliderPointerOutEvent {
    fn from(event: ListenerInput<Pointer<Out>>) -> Self {
        SliderPointerOutEvent {
            target: event.target(),
            hit: event.hit.clone(),
        }
    }
}
