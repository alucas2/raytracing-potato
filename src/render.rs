use crate::utility::*;
use crate::randomness::*;
use crate::hittable::Hittable;
use crate::material::Material;
use crate::texture::Texture;
use crate::mesh::Mesh;
use crate::material::Emit;

/// Global data to be shared by the rendering workers.
pub struct SceneData {
    pub material_table: Vec<Material>,
    pub texture_table: Vec<Texture>,
    pub mesh_table: Vec<Mesh>,
}

// ------------------------------------------- Camera -------------------------------------------

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

// ------------------------------------------- Image sampling -------------------------------------------

#[derive(Debug, Clone)]
pub struct Multisampler {
    pub width: u32,
    pub height: u32,
    pub num_samples: u32
}

impl Multisampler {
    /// Get the sample coordinates of a pixel, in the range [0, 1]
    pub fn make_uv(&self, i: u32, j: u32) -> Rvec2 {
        vector![
            i as Real / self.width as Real,
            j as Real / self.height as Real
        ]
    }

    /// Get multiple samples coordinates for a pixel, in the range [0, 1]
    pub fn make_uv_jitter(&self, i: u32, j: u32, rng: &mut Randomizer) -> impl Iterator<Item=Rvec2> + '_ {
        let mut rng = rng.clone();
        (0..self.num_samples).map(move |_| {
            vector![
                (i as Real + rng.gen::<Real>()) / self.width as Real,
                (j as Real + rng.gen::<Real>()) / self.height as Real
            ]
        })
    }
}

// ------------------------------------------- Main rendering -------------------------------------------

pub struct PathTraceOutput {
    pub final_color: Color,
    pub normal: Rvec3,
    pub hit: bool,
}

// TODO: could the background be a material too?
pub fn trace_path(scene: &Hittable, ray: &Ray, depth: usize, scene_data: &SceneData, rng: &mut Randomizer,
    background: &Emit) -> PathTraceOutput
{
    assert!(depth >= 1);
    trace_path_first(scene, ray, depth, scene_data, rng, background)
}

// The first ray of the path tracing provides additional noiseless data like albedo and normal
fn trace_path_first(scene: &Hittable, ray: &Ray, depth: usize, scene_data: &SceneData, rng: &mut Randomizer,
    background: &Emit) -> PathTraceOutput
{
    if let Some((hit, material)) = scene.hit(ray, scene_data) {
        let mut mat_out = scene_data.material_table[material.to_index()].evaluate(ray, &hit, scene_data, rng);
        let normal = hit.normal;
        let final_color = mat_out.emit + mat_out.scatter.take().map_or(
            // Absorb
            rgb(0.0, 0.0, 0.0),
            // Bounce
            |scatter| mat_out.absorb.component_mul(
                &trace_path_continue(scene, &scatter, depth-1, scene_data, rng, background)
            )
        );
        PathTraceOutput {final_color, normal, hit: true}
    } else {
        let final_color = background.evaluate(ray, &Hit::at_infinity(&ray.direction), scene_data, rng);
        let normal = rgb(0.0, 0.0, 0.0); // What to put here? Will advise later
        PathTraceOutput {final_color, normal, hit: false}
    }
}

// The rays that come after the first provide just a color
fn trace_path_continue(scene: &Hittable, ray: &Ray, depth: usize, scene_data: &SceneData, rng: &mut Randomizer,
    background: &Emit) -> Color
{
    if depth == 0 {
        // This ray did not reach any light
        return rgb(0.0, 0.0, 0.0)
    }

    if let Some((hit, material)) = scene.hit(ray, scene_data) {
        let mut mat_out = scene_data.material_table[material.to_index()].evaluate(ray, &hit, scene_data, rng);
        mat_out.emit + mat_out.scatter.take().map_or(
            // Absorb
            rgb(0.0, 0.0, 0.0),
            // Bounce
            |scatter| mat_out.absorb.component_mul(
                &trace_path_continue(scene, &scatter, depth-1, scene_data, rng, background)
            )
        )
    } else {
        background.evaluate(ray, &Hit::at_infinity(&ray.direction), scene_data, rng)
    }
}