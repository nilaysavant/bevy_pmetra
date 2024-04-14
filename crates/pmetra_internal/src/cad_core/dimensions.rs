use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
};
use truck_modeling::{Point3, Vector2, Vector3};

/// Trait that allows converting primitive and using as bevy [`Vec3`].
pub trait AsBevyVec3 {
    /// Convert and use as Bevy [`Vec3`].
    fn as_bevy_vec3(&self) -> Vec3;
}

/// Trait that allows converting primitive and using as bevy [`DVec3`].
pub trait AsBevyDVec3 {
    /// Convert and use as Bevy [`DVec3`].
    fn as_bevy_dvec3(&self) -> DVec3;
}

/// Trait that allows converting primitive and using as bevy [`Vec2`].
pub trait AsBevyVec2 {
    /// Convert and use as Bevy [`Vec2`].
    fn as_bevy_vec2(&self) -> Vec2;
}

/// Trait that allows converting primitive and using as bevy [`DVec2`].
pub trait AsBevyDVec2 {
    /// Convert and use as Bevy [`DVec2`].
    fn as_bevy_dvec2(&self) -> DVec2;
}

// For [`Point3`]...

impl AsBevyVec3 for Point3 {
    fn as_bevy_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl AsBevyDVec3 for Point3 {
    fn as_bevy_dvec3(&self) -> DVec3 {
        DVec3::new(self.x, self.y, self.z)
    }
}

// For [`Vector3`]...

impl AsBevyVec3 for Vector3 {
    fn as_bevy_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl AsBevyDVec3 for Vector3 {
    fn as_bevy_dvec3(&self) -> DVec3 {
        DVec3::new(self.x, self.y, self.z)
    }
}

// For [`Vector2`]...

impl AsBevyVec2 for Vector2 {
    fn as_bevy_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

impl AsBevyDVec2 for Vector2 {
    fn as_bevy_dvec2(&self) -> DVec2 {
        DVec2::new(self.x, self.y)
    }
}
