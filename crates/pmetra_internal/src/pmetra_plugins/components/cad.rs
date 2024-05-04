use bevy::prelude::*;

use crate::pmetra_core::builders::{CadMeshOutlines, CadSliderType};

/// Marker for CAD generated entities root.
#[derive(Debug, Component, Reflect)]
pub struct CadGeneratedRoot;

/// Root level selection state.
#[derive(Debug, Component, Reflect, Default)]
pub enum CadGeneratedRootSelectionState {
    /// No selection.
    #[default]
    None,
    /// Hovered.
    Hovered,
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

#[derive(Debug, Component)]
pub struct CadGeneratedSlider;

#[derive(Debug, Component)]
pub struct CadGeneratedSliderConfig {
    pub thumb_radius: f32,
    pub drag_plane_normal: Vec3,
    pub slider_type: CadSliderType,
}

#[derive(Debug, Component, Default)]
pub enum CadGeneratedSliderState {
    #[default]
    Normal,
    Dragging,
}

#[derive(Debug, Component)]
pub struct CadGeneratedSliderPreviousTransform(pub Transform);

#[derive(Debug, Component)]
pub struct BelongsToCadGeneratedSlider(pub Entity);

#[derive(Debug, Component)]
pub struct CadGeneratedSliderDragPlane;
