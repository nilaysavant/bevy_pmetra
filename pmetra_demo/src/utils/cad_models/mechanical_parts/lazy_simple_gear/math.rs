use bevy::math::{DMat3, DQuat, DVec2, DVec3, Mat3, Vec3Swizzles};

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: DVec3, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn from_3_non_collinear_points(p1: DVec3, p2: DVec3, p3: DVec3) -> Self {
        // Step 1: Find midpoints
        let m12 = DVec3::new(
            (p1.x + p2.x) / 2.0,
            (p1.y + p2.y) / 2.0,
            (p1.z + p2.z) / 2.0,
        );
        let m13 = DVec3::new(
            (p1.x + p3.x) / 2.0,
            (p1.y + p3.y) / 2.0,
            (p1.z + p3.z) / 2.0,
        );
        let m23 = DVec3::new(
            (p2.x + p3.x) / 2.0,
            (p2.y + p3.y) / 2.0,
            (p2.z + p3.z) / 2.0,
        );

        // Step 2: Find vectors v12 and v23
        let v12 = DVec3::new(m13.x - m12.x, m13.y - m12.y, m13.z - m12.z);
        let v23 = DVec3::new(m13.x - m23.x, m13.y - m23.y, m13.z - m23.z);

        // Step 3: Find normal vectors n12 and n23
        let n12 = DVec3::new(v12.x, v12.y, 0.0).cross(DVec3::Z);
        let n23 = DVec3::new(v23.x, v23.y, 0.0).cross(DVec3::Z);

        // Step 4: Find rotation matrices to align n12 and n23 with z-axis
        let rot12 = DMat3::from_cols(DVec3::new(n12.x, n12.y, n12.z), DVec3::Z, DVec3::Y);
        let rot23 = DMat3::from_cols(DVec3::new(n23.x, n23.y, n23.z), DVec3::Z, DVec3::Y);

        // Apply rotations to v12 and v23
        let v12_rotated = rot12 * v12;
        let v23_rotated = rot23 * v23;

        // Find the intersection point of the lines passing through m12 and m23 with directions n12 and n23
        let center_rotated = solve_2d_system(v12_rotated.xy(), v23_rotated.xy(), (m23 - m12).xy());

        // Step 5: Find the radius r
        let radius = (DVec3::new(
            p1.x - center_rotated.x,
            p1.y - center_rotated.y,
            p1.z - center_rotated.z,
        ))
        .length();

        // Rotate the center back to the original coordinate system
        let center = rot12.inverse() * center_rotated;

        Self { center, radius }
    }

    pub fn get_intersection_with_line(&self, line: &Line) -> Option<(DVec3, DVec3)> {
        let oc = line.p1 - self.center;
        let line_direction = (line.p2 - line.p1).normalize();
        let a = line_direction.dot(line_direction);
        let b = 2.0 * oc.dot(line_direction);
        let c = oc.dot(oc) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant >= 0.0 {
            let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
            let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

            let intersection1 = line.p1 + t1 * line_direction;
            let intersection2 = line.p1 + t2 * line_direction;

            Some((intersection1, intersection2))
        } else {
            None
        }
    }
}

fn solve_2d_system(v1: DVec2, v2: DVec2, b: DVec2) -> DVec3 {
    let mat = DMat3::from_cols(
        DVec3::new(v1.x, v2.x, 0.0),
        DVec3::new(v1.y, v2.y, 0.0),
        DVec3::Z,
    );
    mat.inverse() * b.extend(1.0)
}

/// Line in 2 point form.
pub struct Line {
    pub p1: DVec3,
    pub p2: DVec3,
}

impl Line {
    pub fn new(p1: DVec3, p2: DVec3) -> Self {
        Self { p1, p2 }
    }

    pub fn from_point_direction(p1: DVec3, direction: DVec3) -> Self {
        let p2 = p1 + direction;
        Self { p1, p2 }
    }
}
