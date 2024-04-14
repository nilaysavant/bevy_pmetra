use bevy::prelude::*;

/// Extends [`Transform`] for Transforming a UV face.
pub trait UvFaceTransform {
    /// Gets [`Transform`] for UV face.
    ///
    /// This basically helps to translate, rotate, scale the face from its center, ie. (0.5, 0.5).
    fn uv_face_transform(translation: Vec3, rotation: Quat, scale: Vec3) -> Transform;
}

impl UvFaceTransform for Transform {
    fn uv_face_transform(translation: Vec3, rotation: Quat, scale: Vec3) -> Transform {
        let mut transform = Self::default();
        transform.scale = scale;
        transform.translation = (1. - transform.scale) / 2.;
        transform.rotate_around(Vec3::splat(0.5), rotation);
        transform.translation += translation;

        transform
    }
}
