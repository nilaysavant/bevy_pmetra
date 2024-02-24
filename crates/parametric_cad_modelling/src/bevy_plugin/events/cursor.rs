use bevy::prelude::*;
use bevy_mod_picking::{
    backend::HitData,
    events::{Move, Pointer},
    prelude::*,
};

/// Used to Move cursor on **drag plane** [`Pointer<Move>`].
#[derive(Event)]
pub struct TransformCursorEvent {
    pub target: Entity,
    pub hit: HitData,
}

impl From<ListenerInput<Pointer<Move>>> for TransformCursorEvent {
    fn from(event: ListenerInput<Pointer<Move>>) -> Self {
        TransformCursorEvent {
            target: event.target(),
            hit: event.hit.clone(),
        }
    }
}

/// Dispatched on **Cursor** : [`Pointer<Move>`].
#[derive(Event)]
pub struct CursorPointerMoveEvent {
    pub target: Entity,
    pub hit: HitData,
}

impl From<ListenerInput<Pointer<Move>>> for CursorPointerMoveEvent {
    fn from(event: ListenerInput<Pointer<Move>>) -> Self {
        CursorPointerMoveEvent {
            target: event.target(),
            hit: event.hit.clone(),
        }
    }
}

/// Dispatched on **Cursor** : [`Pointer<Out>`].
#[derive(Event)]
pub struct CursorPointerOutEvent {
    pub target: Entity,
    pub hit: HitData,
}

impl From<ListenerInput<Pointer<Out>>> for CursorPointerOutEvent {
    fn from(event: ListenerInput<Pointer<Out>>) -> Self {
        CursorPointerOutEvent {
            target: event.target(),
            hit: event.hit.clone(),
        }
    }
}
