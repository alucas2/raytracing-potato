use crate::utility::*;
use crate::hittable::Hittable;
use crate::material::Material;
use crate::texture::Texture;

/// Global data to be shared by the rendering workers.
pub struct SceneData {
    pub material_table: Vec<Material>,
    pub texture_table: Vec<Texture>,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub aspect_ratio: Real,
    pub fov: Real,
    pub focal_dist: Real,
    pub lens_radius: Real,
    pub transformation: Transformation,
}

// Local camera frame:
// X axis points right
// Y axis points up
// Z axis points behind
impl Camera {
    pub fn shoot(&self, image_uv: Rvec2, rng: &mut Randomizer) -> Ray {
        let tan_fov = (0.5 * self.fov).tan();
        
        // Ray origin in local frame
        let origin = self.lens_radius * rng.sample(UnitDisk);
        let origin = vector![origin.x, origin.y, 0.0];

        // Ray direction in local frame
        let direction = (vector![
            (2.0 * image_uv.x - 1.0) * tan_fov * self.focal_dist * self.aspect_ratio,
            (2.0 * image_uv.y - 1.0) * tan_fov * self.focal_dist,
            -self.focal_dist
        ] - origin).normalize();
        
        Ray {
            direction: self.transformation.transform_vector(&direction),
            origin: self.transformation.transform_point(&origin),
            t_min: RAY_EPSILON,
            t_max: INFINITY,
        }
    }
}

// TODO: could the background be a material too?
pub fn hit_scene<Background>(scene: &Hittable, ray: &Ray, depth: usize, scene_data: &SceneData, rng: &mut Randomizer,
    background: Background)
    -> Color
    where Background: Fn(&Ray) -> Color
{
    if depth == 0 {
        // This ray did not reach any light
        return rgb(0.0, 0.0, 0.0)
    }

    if let Some(hit) = scene.hit(ray) {
        let material = &scene_data.material_table[hit.material.to_index()];
        if let Some((attenuation, scatter)) = material.scatter(ray, &hit, scene_data, rng) {
            // Scatter
            attenuation.component_mul(&hit_scene(scene, &scatter, depth-1, scene_data, rng, background))
        } else {
            // Absorb
            rgb(0.0, 0.0, 0.0)
        }
    } else {
        background(ray)
    }
}