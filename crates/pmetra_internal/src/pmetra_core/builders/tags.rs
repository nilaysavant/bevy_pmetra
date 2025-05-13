use bevy::{platform::collections::HashMap, prelude::*};
use truck_modeling::{Edge, Face, Shell, Solid, Vertex, Wire};

/// Holds the [`CadElementTag`] mapped to [`CadElement`]
///
/// Useful for getting actual vertex/edge/face/etc object from [`CadElementTag`].
#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct CadTaggedElements(pub HashMap<CadElementTag, CadElement>);

/// Used for tagging a [`CadElement`]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CadElementTag(pub String);

impl CadElementTag {
    /// Create new [`TaggedFace`] from str.
    pub fn new(str: &str) -> Self {
        Self(str.to_string())
    }
}

/// Cad primitive element used for tagging.
#[derive(Debug, Clone)]
pub enum CadElement {
    Vertex(Vertex),
    Edge(Edge),
    Wire(Wire),
    Face(Face),
    Shell(Shell),
    Solid(Solid),
}
