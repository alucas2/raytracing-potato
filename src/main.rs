use raytracing2::image::*;
use raytracing2::utility::*;
use raytracing2::camera::*;
use raytracing2::hittable::*;

fn main() {
    let mut output = RgbaImage::new(800, 600);

    let camera = Camera {
        focal_dist: 1.0,
        aspect_ratio: output.aspect_ratio(),
        fov: FRAC_PI_2,
    };

    let scene = Hittable::List(vec![
        Hittable::Sphere {
            center: point3(0.0, 0.0, -1.0),
            radius: 0.5,
        },
        Hittable::Sphere {
            center: point3(0.0, -100.5, -1.0),
            radius: 100.0,
        }
    ]);

    let hit_scene = |ray: Ray| {
        if let Some(hit) = scene.hit(ray, 0.0, Real::INFINITY) {
            rgb(0.5 * hit.normal.x + 0.5, 0.5 * hit.normal.y + 0.5, 0.5 * hit.normal.z + 0.5)
        } else {
            let unit_direction = ray.a.normalize();
            let t = 0.5 * (unit_direction.y + 1.0);
            (1.0 - t) * rgb(1.0, 1.0, 1.0) + t * rgb(0.5, 0.7, 1.0)
        }
    };

    // Samples per pixel
    let num_samples = 10;
    
    // Render the image
    for j in (0..output.height()).rev() {
        for i in 0..output.width() {
            let mut color = rgb(0.0, 0.0, 0.0);
            for s in output.samples_jitter(i, j, num_samples) {
                let ray = camera.shoot(s);
                color += hit_scene(ray);
            }
            *output.get_mut(i, j) = to_u8(color / num_samples as Real);
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
