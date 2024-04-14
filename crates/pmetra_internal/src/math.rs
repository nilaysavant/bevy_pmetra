use bevy::prelude::*;

/// Calculate rotation [`Quat`] from normals (default v/s new).
pub fn get_rotation_from_normals(default_normal: Vec3, new_normal: Vec3) -> Quat {
    // Use from rotation arc fn on Quat to do this internally...
    let rotation = Quat::from_rotation_arc(default_normal, new_normal);

    rotation
}
