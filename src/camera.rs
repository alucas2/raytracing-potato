use crate::utility::*;

pub struct Camera {
    pub aspect_ratio: Real,
    pub focal_dist: Real,
    pub fov: Real,
}

// Local camera frame:
// X axis points right
// Y axis points up
// Z axis points behind
impl Camera {
    pub fn shoot(&self, image_uv: Point2) -> Ray {
        let tan_fov = (0.5 * self.fov).tan();

        // Ray direction in local frame
        let a = vec3(
            (2.0 * image_uv.x - 1.0) * tan_fov * self.aspect_ratio,
            (2.0 * image_uv.y - 1.0) * tan_fov,
            -self.focal_dist
        );
        
        // Ray origin in local frame
        let b = point3(0.0, 0.0, 0.0);

        Ray {a, b}
    }
}