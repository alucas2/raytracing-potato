use raytracing2::image::*;
use raytracing2::utility::*;
use raytracing2::render::hit_scene;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::thread;
use indicatif::ProgressBar;

mod example_scenes;

fn sky_background(ray: &Ray) -> Color {
    let unit_direction = ray.direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * rgb(1.0, 1.0, 1.0) + t * rgb(0.5, 0.7, 1.0)
}

fn main() {
    let (output_width, output_height) = (1280, 720);

    // Load the scene
    let mut scene = example_scenes::two_balls();
    // let mut scene = example_scenes::more_balls_optimized();
    scene.camera.aspect_ratio = output_width as Real / output_height as Real;

    // Renderer parameters
    let max_bounce = 8;
    let tile_size = 32;
    let num_workers = 4;

    let sampler = Multisampler {
        width: output_width,
        height: output_height,
        num_samples: 16,
    };
    
    let job_queue = Tile::generate(output_width, output_height, tile_size, tile_size);
    let progress_bar = ProgressBar::new(job_queue.len() as _);
    
    // Put the things into arcs
    let scene = Arc::new(scene);
    let job_queue = Arc::new(Mutex::new(job_queue));
    let complete_jobs = Arc::new(Mutex::new(Vec::new()));
    
    // Start the rendering workers
    let t0 = Instant::now();
    let workers: Vec<_> = (0..num_workers).map(|_| {
        let job_queue = Arc::clone(&job_queue);
        let complete_jobs = Arc::clone(&complete_jobs);
        let progress_bar = progress_bar.clone();
        let sampler = sampler.clone();
        let scene = Arc::clone(&scene);
        let mut rng = Randomizer::from_entropy();

        thread::spawn(move || {
            loop {
                let job = {
                    // Momentarily lock the job queue to pop a new job
                    job_queue.lock().unwrap().pop()
                };

                if let Some(mut tile) = job {
                    // Walk on each pixel of the tile
                    for tj in 0..tile.height() {
                        for ti in 0..tile.width() {
                            // Sample the scene multiple times
                            let mut color = rgb(0.0, 0.0, 0.0);
                            let samples = sampler.make_uv_jitter(ti + tile.offset_i(), tj + tile.offset_j(), &mut rng);
                            for s in samples {
                                let ray = scene.camera.shoot(s, &mut rng);
                                color += hit_scene(
                                    &scene.root, &ray, max_bounce, &scene.scene_data, &mut rng, sky_background
                                );
                            }
                            // The final color is the average of the samples
                            *tile.get_mut(ti, tj) = to_srgb_u8(color / sampler.num_samples as Real);
                        }
                    }
                    // Push the finished job
                    complete_jobs.lock().unwrap().push(tile);
                    progress_bar.inc(1);
                } else {
                    break
                }
            }
        })
    }).collect();

    for w in workers {
        w.join().unwrap();
    }

    progress_bar.finish();
    println!("Rendering done in {:.2} seconds", t0.elapsed().as_secs_f64());

    // Combine the tiles
    let mut output = RgbaImage::new(output_width, output_height);
    for tile in complete_jobs.lock().unwrap().iter() {
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
