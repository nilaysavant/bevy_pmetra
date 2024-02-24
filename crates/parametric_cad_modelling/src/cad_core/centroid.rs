use bevy::math::DVec3;
use truck_modeling::{Point3, Wire};

/// Trait that allows calculating and getting centroid of a CAD(truck) primitive.
pub trait CadCentroid {
    /// Get centroid of the CAD(truck) primitive.
    fn get_centroid(&self) -> DVec3;
}

impl CadCentroid for Wire {
    /// Get centroid of wire.
    fn get_centroid(&self) -> DVec3 {
        let mut centroid = DVec3::ZERO;
        let mut total_count = 0;
        for vertex in self.vertex_iter() {
            let Point3 { x, y, z } = vertex.point();
            centroid += DVec3::new(x, y, z);
            total_count += 1;
        }
        centroid /= total_count as f64;
        centroid
    }
}
