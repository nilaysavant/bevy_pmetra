use bevy::prelude::*;

use crate::cad_core::builders::{CadCursorType, CadMeshOutlines};

/// Marker for CAD generated entities root.
#[derive(Debug, Component, Reflect)]
pub struct CadGeneratedRoot;

/// Root level selection state.
#[derive(Debug, Component, Reflect, Default)]
pub enum CadGeneratedRootSelectionState {
    /// No selection.
    #[default]
    None,
    /// Selected.
    Selected,
}

/// Marker indicating which [`CadGenerated`] [`Entity`] it belongs to.
#[derive(Debug, Component, Clone, Reflect)]
pub struct BelongsToCadGeneratedRoot(pub Entity);

/// Marker for CAD generated mesh root.
#[derive(Debug, Component, Reflect)]
pub struct CadGeneratedMesh;

/// Marker indicating which [`CadGeneratedMesh`] [`Entity`] it belongs to.
#[derive(Debug, Component, Reflect)]
pub struct BelongsToCadGeneratedMesh(pub Entity);

/// Holds the data to construct the outline `linestrip` [`Gizmos`]
/// for [`CadGeneratedMesh`].
#[derive(Debug, Component)]
pub struct CadGeneratedMeshOutlines(pub CadMeshOutlines);

#[derive(Debug, Component, Default)]
pub enum CadGeneratedMeshOutlinesState {
    #[default]
    Invisible,
    SlightlyVisible,
    Visible,
}

#[derive(Debug, Component)]
pub struct CadGeneratedCursor;

#[derive(Debug, Component)]
pub struct CadGeneratedCursorConfig {
    pub cursor_radius: f32,
    pub drag_plane_normal: Vec3,
    pub cursor_type: CadCursorType,
}

#[derive(Debug, Component, Default)]
pub enum CadGeneratedCursorState {
    #[default]
    Normal,
    Dragging,
}

#[derive(Debug, Component)]
pub struct CadGeneratedCursorPreviousTransform(pub Transform);

#[derive(Debug, Component)]
pub struct BelongsToCadGeneratedCursor(pub Entity);

#[derive(Debug, Component)]
pub struct CadGeneratedCursorDragPlane;
