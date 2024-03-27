use anyhow::Result;
use bevy::{prelude::*, utils::HashMap};

use super::CadCursorName;

#[derive(Debug, Clone, Deref, DerefMut, Default)]
pub struct CadCursors(pub HashMap<CadCursorName, CadCursor>);

impl CadCursors {
    /// Add new cursor to the [`CadCursors`] collection.
    pub fn add_cursor(&mut self, cursor_name: CadCursorName, cursor: CadCursor) -> Result<Self> {
        self.insert(cursor_name, cursor);
        Ok(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct CadCursor {
    pub normal: Vec3,
    pub transform: Transform,
    pub cursor_radius: f32,
    pub cursor_type: CadCursorType,
}

impl Default for CadCursor {
    fn default() -> Self {
        Self {
            normal: Default::default(),
            transform: Default::default(),
            cursor_radius: 0.1,
            cursor_type: Default::default(),
        }
    }
}

impl CadCursor {
    pub fn with_drag_direction(&mut self, drag_direction: CadCursorType) -> Self {
        self.cursor_type = drag_direction;
        self.clone()
    }
}

#[derive(Debug, Clone, Default, Reflect)]
pub enum CadCursorType {
    #[default]
    /// Cursor that allows drag across [`CadCursor`] plane.
    Planer,
    /// Restricted to linear direction [`Vec3`] on the [`CadCursor`] plane.
    Linear {
        direction: Vec3,
        limit_min: Option<Vec3>,
        limit_max: Option<Vec3>,
    },
}
