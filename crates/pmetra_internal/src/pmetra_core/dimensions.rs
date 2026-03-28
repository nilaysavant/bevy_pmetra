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

/// Trait that allows converting from bevy [`Vec3`] into primitive.
pub trait FromBevyVec3 {
    /// Convert from Bevy [`Vec3`].
    fn from_bevy_vec3(vec: Vec3) -> Self;
}

/// Trait that allows converting primitive and using as bevy [`DVec3`].
pub trait AsBevyDVec3 {
    /// Convert and use as Bevy [`DVec3`].
    fn as_bevy_dvec3(&self) -> DVec3;
}

/// Trait that allows converting from bevy [`DVec3`] into primitive.
pub trait FromBevyDVec3 {
    /// Convert from Bevy [`DVec3`].
    fn from_bevy_dvec3(vec: DVec3) -> Self;
}

/// Trait that allows converting primitive and using as bevy [`Vec2`].
pub trait AsBevyVec2 {
    /// Convert and use as Bevy [`Vec2`].
    fn as_bevy_vec2(&self) -> Vec2;
}

/// Trait that allows converting from bevy [`Vec2`] into primitive.
pub trait FromBevyVec2 {
    /// Convert from Bevy [`Vec2`].
    fn from_bevy_vec2(vec: Vec2) -> Self;
}

/// Trait that allows converting primitive and using as bevy [`DVec2`].
pub trait AsBevyDVec2 {
    /// Convert and use as Bevy [`DVec2`].
    fn as_bevy_dvec2(&self) -> DVec2;
}

/// Trait that allows converting from bevy [`DVec2`] into primitive.
pub trait FromBevyDVec2 {
    /// Convert from Bevy [`DVec2`].
    fn from_bevy_dvec2(vec: DVec2) -> Self;
}

// For [`Point3`]...

impl AsBevyVec3 for Point3 {
    fn as_bevy_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl FromBevyVec3 for Point3 {
    fn from_bevy_vec3(vec: Vec3) -> Self {
        Point3::new(vec.x as f64, vec.y as f64, vec.z as f64)
    }
}

impl AsBevyDVec3 for Point3 {
    fn as_bevy_dvec3(&self) -> DVec3 {
        DVec3::new(self.x, self.y, self.z)
    }
}

impl FromBevyDVec3 for Point3 {
    fn from_bevy_dvec3(vec: DVec3) -> Self {
        Point3::new(vec.x, vec.y, vec.z)
    }
}

// For [`Vector3`]...

impl AsBevyVec3 for Vector3 {
    fn as_bevy_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl FromBevyVec3 for Vector3 {
    fn from_bevy_vec3(vec: Vec3) -> Self {
        Vector3::new(vec.x as f64, vec.y as f64, vec.z as f64)
    }
}

impl AsBevyDVec3 for Vector3 {
    fn as_bevy_dvec3(&self) -> DVec3 {
        DVec3::new(self.x, self.y, self.z)
    }
}

impl FromBevyDVec3 for Vector3 {
    fn from_bevy_dvec3(vec: DVec3) -> Self {
        Vector3::new(vec.x, vec.y, vec.z)
    }
}

// For [`Vector2`]...

impl AsBevyVec2 for Vector2 {
    fn as_bevy_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

impl FromBevyVec2 for Vector2 {
    fn from_bevy_vec2(vec: Vec2) -> Self {
        Vector2::new(vec.x as f64, vec.y as f64)
    }
}

impl AsBevyDVec2 for Vector2 {
    fn as_bevy_dvec2(&self) -> DVec2 {
        DVec2::new(self.x, self.y)
    }
}

impl FromBevyDVec2 for Vector2 {
    fn from_bevy_dvec2(vec: DVec2) -> Self {
        Vector2::new(vec.x, vec.y)
    }
}
