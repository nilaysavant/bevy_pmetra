use anyhow::Result;
use bevy::{prelude::*, utils::HashMap};

#[derive(Debug, Clone, Deref, DerefMut, Default)]
pub struct CadSliders(pub HashMap<CadSliderName, CadSlider>);

impl CadSliders {
    /// Add new slider to the [`CadSliders`] collection.
    pub fn add_slider(&mut self, name: CadSliderName, slider: CadSlider) -> Result<Self> {
        self.insert(name, slider);
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, Deref, DerefMut, Hash, PartialEq, Eq, Component)]
pub struct CadSliderName(pub String);

impl From<String> for CadSliderName {
    fn from(value: String) -> Self {
        CadSliderName(value)
    }
}

#[derive(Debug, Clone)]
pub struct CadSlider {
    pub drag_plane_normal: Vec3,
    pub transform: Transform,
    pub thumb_radius: f32,
    pub slider_type: CadSliderType,
}

impl Default for CadSlider {
    fn default() -> Self {
        Self {
            drag_plane_normal: Vec3::Y,
            transform: Default::default(),
            thumb_radius: 0.1,
            slider_type: Default::default(),
        }
    }
}

impl CadSlider {
    pub fn with_drag_direction(&mut self, drag_direction: CadSliderType) -> Self {
        self.slider_type = drag_direction;
        self.clone()
    }
}

#[derive(Debug, Clone, Default, Reflect)]
pub enum CadSliderType {
    #[default]
    /// Allows *dragging* [`CadSlider`] anywhere across its plane.
    Planer,
    /// [`CadSlider`] is Restricted to linear direction [`Vec3`] in its plane.
    Linear {
        direction: Vec3,
        limit_min: Option<Vec3>,
        limit_max: Option<Vec3>,
    },
}
