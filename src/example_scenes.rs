use raytracing2::camera::*;
use raytracing2::hittable::*;
use raytracing2::material::*;
use raytracing2::utility::*;

#[derive(Clone)]
pub struct ExampleScene {
    pub camera: Camera,
    pub material_table: Vec<Material>,
    pub root: Hittable,
}

pub fn three_balls() -> ExampleScene {
    let camera = Camera {
        aspect_ratio: 1.0,
        fov: FRAC_PI_2,
        focal_dist: 3.46,
        lens_radius: 0.1,
        transformation: Transformation::lookat(
            &vector![-2.0, 2.0, 1.0],
            &vector![0.0, 0.0, -1.0],
            &vector![0.0, 1.0, 0.0]
        ),
    };

    // Table of materials
    let material_table = vec![
        Material::Lambert {albedo: rgb(0.8, 0.8, 0.0)},
        Material::Lambert {albedo: rgb(0.1, 0.2, 0.5)},
        Material::Dielectric {refraction_index: 1.5},
        Material::Metal {albedo: rgb(0.8, 0.6, 0.2), fuzziness: 0.0},
    ];

    // List of object of the scene
    let root = Hittable::List(vec![
        Hittable::Sphere {center: vector![0.0, -100.5, -1.0], radius: 100.0, material_id: 0}, // Ground
        Hittable::Sphere {center: vector![0.0, 0.0, -1.0], radius: 0.5, material_id: 1}, // Diffuse sphere
        Hittable::Sphere {center: vector![-1.0, 0.0, -1.0], radius: 0.5, material_id: 2}, // Metal sphere
        Hittable::Sphere {center: vector![1.0, 0.0, -1.0], radius: 0.5, material_id: 3}, // Glass sphere
    ]);

    ExampleScene {camera, material_table, root}
}