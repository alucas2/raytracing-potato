use raytracing2::image::*;
use raytracing2::utility::*;
use raytracing2::camera::*;
use raytracing2::hittable::*;
use raytracing2::material::*;
use std::time::Instant;
use indicatif::ProgressBar;

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
    ray.attenuation.component_mul(&background_color)
}

fn hit_scene(scene: &Hittable, ray: Ray, depth: usize, material_table: &[Material], rng: &mut Randomizer) -> Color {
    if depth == 0 {
        // This ray did not reach any light
        return rgb(0.0, 0.0, 0.0)
    }

    if let Some(hit) = scene.hit(ray, 1e-3, Real::INFINITY) {
        if let Some(scatter) = material_table.get(hit.material_id as usize).unwrap().scatter(ray, hit, rng) {
            // Scatter
            hit_scene(scene, scatter, depth-1, material_table, rng)
        } else {
            // Absorb
            rgb(0.0, 0.0, 0.0)
        }
    } else {
        sky_background(ray)
    }
}

fn main() {
    let (output_width, output_height) = (800, 450);

    let camera = Camera {
        aspect_ratio: output_width as Real / output_height as Real,
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
    let scene = Hittable::List(vec![
        Hittable::Sphere {
            center: vector![0.0, 0.0, -1.0],
            radius: 0.5,
            material_id: 1,
        },
        Hittable::Sphere {
            center: vector![0.0, -100.5, -1.0],
            radius: 100.0,
            material_id: 0,
        },
        Hittable::Sphere {
            center: vector![-1.0, 0.0, -1.0],
            radius: 0.5,
            material_id: 2,
        },
        Hittable::Sphere {
            center: vector![1.0, 0.0, -1.0],
            radius: 0.5,
            material_id: 3,
        }
    ]);

    // Renderer parameters
    let max_bounce = 10;
    let tile_size = 64;

    let sampler = Multisampler {
        width: output_width,
        height: output_height,
        num_samples: 16,
    };
    
    // Render the image
    let mut tiles = Tile::new(output_width, output_height, tile_size, tile_size);
    let mut rng = Randomizer::from_entropy();
    let t0 = Instant::now();
    let pbar = ProgressBar::new(tiles.len() as _);
    pbar.set_message("Rendering...");

    for tile in tiles.iter_mut() {
        for j in 0..tile.pixels.height() {
            for i in 0..tile.pixels.width() {
                let mut color = rgb(0.0, 0.0, 0.0);
                for s in sampler.samples_jitter(i + tile.pixel_offset.0, j + tile.pixel_offset.1, &mut rng) {
                    let ray = camera.shoot(s, &mut rng);
                    color += hit_scene(&scene, ray, max_bounce, &material_table, &mut rng);
                }
                *tile.pixels.get_mut(i, j) = to_srgb_u8(color / sampler.num_samples as Real);
            }
        }
        pbar.inc(1);
    }
    pbar.finish();
    println!("Rendering done in {:.2} seconds", t0.elapsed().as_secs_f64());

    // Combine the tiles
    let mut output = RgbaImage::new(output_width, output_height);
    for tile in tiles {
        tile.write_to(&mut output);
    }

    // Save the output in a file
    let output_name = "output.tga";
    tga::save(&output, output_name).unwrap();

    // Open the output in the default image viewer
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd").args(["/c", output_name]).spawn().unwrap();
    }
}
