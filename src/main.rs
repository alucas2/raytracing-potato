use raytracing2::image::*;
use raytracing2::utility::*;
use raytracing2::camera::*;
use raytracing2::hittable::*;
use raytracing2::material::*;

fn scene_normals(scene: &Hittable, ray: Ray) -> Color {
    if let Some(hit) = scene.hit(ray, 0.0, Real::INFINITY) {
        rgb(0.5 * hit.normal.x + 0.5, 0.5 * hit.normal.y + 0.5, 0.5 * hit.normal.z + 0.5)
    } else {
        rgb(0.0, 0.0, 0.0)
    }
}

fn sky_background(ray: Ray) -> Color {
    let unit_direction = ray.direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    let background_color = (1.0 - t) * rgb(1.0, 1.0, 1.0) + t * rgb(0.5, 0.7, 1.0);
    ray.attenuation.mul_element_wise(background_color)
}

fn hit_scene(scene: &Hittable, ray: Ray, depth: usize, rng: &mut ThreadRng, material_table: &[Material]) -> Color {
    if depth == 0 {
        // This ray did not reach any light
        return rgb(0.0, 0.0, 0.0)
    }

    if let Some(hit) = scene.hit(ray, 1e-3, Real::INFINITY) {
        if let Some(scatter) = material_table.get(hit.material_id).unwrap().scatter(ray, hit, rng) {
            // Scatter
            hit_scene(scene, scatter, depth-1, rng, material_table)
        } else {
            // Absorb
            rgb(0.0, 0.0, 0.0)
        }
    } else {
        sky_background(ray)
    }
}

fn main() {
    let mut output = RgbaImage::new(800, 600);
    let mut rng = thread_rng();

    let camera = Camera {
        focal_dist: 1.0,
        aspect_ratio: output.aspect_ratio(),
        fov: FRAC_PI_2,
    };

    // Table of materials
    let material_table = vec![
        Material::Lambert {albedo: rgb(0.8, 0.8, 0.0)},
        Material::Lambert {albedo: rgb(0.7, 0.3, 0.3)},
        Material::Metal {albedo: rgb(0.8, 0.8, 0.8), fuzziness: 0.3},
        Material::Metal {albedo: rgb(0.8, 0.6, 0.2), fuzziness: 1.0},
    ];

    // List of object of the scene
    let scene = Hittable::List(vec![
        Hittable::Sphere {
            center: point3(0.0, 0.0, -1.0),
            radius: 0.5,
            material_id: 1,
        },
        Hittable::Sphere {
            center: point3(0.0, -100.5, -1.0),
            radius: 100.0,
            material_id: 0,
        },
        Hittable::Sphere {
            center: point3(-1.0, 0.0, -1.0),
            radius: 0.5,
            material_id: 2,
        },
        Hittable::Sphere {
            center: point3(1.0, 0.0, -1.0),
            radius: 0.5,
            material_id: 3,
        }
    ]);

    // Samples per pixel
    let num_samples = 16;
    let max_bounce = 10;
    
    // Render the image
    for j in (0..output.height()).rev() {
        for i in 0..output.width() {
            let mut color = rgb(0.0, 0.0, 0.0);
            for s in output.samples_jitter(i, j, num_samples, &mut rng) {
                let ray = camera.shoot(s);
                color += hit_scene(&scene, ray, max_bounce, &mut rng, &material_table);
            }
            *output.get_mut(i, j) = to_srgb_u8(color / num_samples as Real);
        }
    }

    // Save the output in a file
    let output_name = "output.tga";
    tga::save(&output, output_name).unwrap();

    // Open the output in the default image viewer
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd").args(["/c", output_name]).spawn().unwrap();
    }
}
