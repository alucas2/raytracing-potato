use raytracing2::image::*;
use raytracing2::utility::*;
use raytracing2::render::*;
use raytracing2::randomness::*;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::thread;
use indicatif::ProgressBar;

mod example_scenes;

fn main() {
    let (output_width, output_height) = (800, 600);

    // Load the scene
    // let mut scene = example_scenes::three_balls();
    // let mut scene = example_scenes::two_balls();
    // let mut scene = example_scenes::more_balls_optimized();
    // let mut scene = example_scenes::earth();
    // let mut scene = example_scenes::one_triangle();
    let mut scene = example_scenes::bunny();
    scene.camera.aspect_ratio = output_width as Real / output_height as Real;

    // Renderer parameters
    let max_bounce = 8; 
    let tile_size = 32;
    let num_workers = 4;

    let sampler = Multisampler {
        width: output_width,
        height: output_height,
        num_samples: 4,
    };
    
    // Put tiles into the job queue
    let job_queue = Tile::split_in_tiles(output_width, output_height, tile_size, tile_size);
    let progress_bar = ProgressBar::new(job_queue.len() as _);
    
    // Wrap the things into arcs
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

                if let Some(tile) = job {
                    // Create 3 buffers
                    let mut color_buffer = Array2d::new(tile.width, tile.height);
                    let mut foreground_buffer = Array2d::new(tile.width, tile.height);
                    
                    // Walk on each pixel of the tile
                    for tj in 0..tile.height {
                        for ti in 0..tile.width {
                            // Jitter the sample inside its pixel
                            let samples = sampler.make_uv_jitter(ti + tile.offset_i, tj + tile.offset_j, &mut rng);
                            
                            // Accumulate the values of each sample
                            let mut final_color = rgb(0.0, 0.0, 0.0);
                            let mut foreground = 0.0;
                            for s in samples {
                                let ray = scene.camera.shoot(s, &mut rng);
                                let trace_out = trace_path(
                                    &scene.root, &ray, max_bounce, &scene.scene_data, &mut rng, &scene.background
                                );
                                final_color += trace_out.final_color;
                                if trace_out.hit {
                                    foreground += 1.0;
                                }
                            }
                            // Write the final color which is the average of the samples
                            *color_buffer.get_mut(ti, tj) = final_color / sampler.num_samples as Real;
                            *foreground_buffer.get_mut(ti, tj) = foreground / sampler.num_samples as Real;
                        }
                    }
                    // Push the finished job
                    complete_jobs.lock().unwrap().push((tile, color_buffer, foreground_buffer));
                    progress_bar.inc(1);
                } else {
                    break
                }
            }
        })
    }).collect();

    // Wait. Wait. Wait.
    for w in workers {
        w.join().unwrap();
    }

    progress_bar.finish();
    println!("Rendering done in {:.2} seconds", t0.elapsed().as_secs_f64());

    // Combine the tiles into one image
    let complete_jobs = Arc::try_unwrap(complete_jobs).unwrap().into_inner().unwrap();
    let mut output_image = Array2d::new(output_width, output_height);
    let transparent_background = false;
    for (tile, color_buffer, foreground_buffer) in complete_jobs {
        for tj in 0..tile.height {
            for ti in 0..tile.width {
                let mut rgba = to_srgb_u8(color_buffer.get(ti, tj));
                if transparent_background {
                    rgba[3] = (255.0 * foreground_buffer.get(ti, tj)) as u8; // Transparent background
                }
                *output_image.get_mut(ti + tile.offset_i, tj + tile.offset_j) = rgba;
            }
        }
    }

    // Save the output in a file
    let output_name = "output.tga";
    tga::save(&output_image, output_name).unwrap();

    // Open the output in the default image viewer
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd").args(["/c", output_name]).spawn().unwrap();
    }
}
