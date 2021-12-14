use raytracing2::camera::*;
use raytracing2::hittable::*;
use raytracing2::material::*;
use raytracing2::utility::*;
use raytracing2::bvh::*;

pub struct ExampleScene {
    pub camera: Camera,
    pub material_table: Vec<Material>,
    pub root: Hittable,
}

#[allow(dead_code)]
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

    // List of objects of the scene
    let root = Hittable::List(vec![
        Hittable::Sphere {center: vector![0.0, -100.5, -1.0], radius: 100.0, material_id: 0}, // Ground
        Hittable::Sphere {center: vector![0.0, 0.0, -1.0], radius: 0.5, material_id: 1}, // Diffuse sphere
        Hittable::Sphere {center: vector![-1.0, 0.0, -1.0], radius: 0.5, material_id: 2}, // Metal sphere
        Hittable::Sphere {center: vector![1.0, 0.0, -1.0], radius: 0.5, material_id: 3}, // Glass sphere
    ]);

    ExampleScene {camera, material_table, root}
}

#[allow(dead_code)]
pub fn more_balls() -> ExampleScene {
    let camera = Camera {
        aspect_ratio: 1.0,
        fov: FRAC_PI_2,
        focal_dist: 7.5,
        lens_radius: 0.02,
        transformation: Transformation::lookat(
            &vector![6.0, 2.0, 4.0],
            &vector![0.0, 0.0, 0.0],
            &vector![0.0, 1.0, 0.0]
        ),
    };

    // Table of materials
    let mut material_table = vec![
        Material::Lambert {albedo: rgb(0.8, 0.8, 0.0)},
        Material::Lambert {albedo: rgb(0.1, 0.2, 0.5)},
        Material::Metal {albedo: rgb(0.8, 0.6, 0.2), fuzziness: 0.0},
        Material::Dielectric {refraction_index: 1.5},
    ];

    // List of objects of the scene
    let mut root = vec![
        Hittable::Sphere {center: vector![0.0, -1000.0, -1.0], radius: 1000.0, material_id: 0}, // Ground
        Hittable::Sphere {center: vector![-4.0, 1.8, 0.0], radius: 1.8, material_id: 1}, // Diffuse sphere
        Hittable::Sphere {center: vector![4.0, 1.8, 0.0], radius: 1.8, material_id: 2}, // Metal sphere
        Hittable::Sphere {center: vector![0.0, 1.8, 0.0], radius: 1.8, material_id: 3}, // Glass sphere
    ];
    let mut rng = StdRng::from_seed([249; 32]);
    for x in -31..31 {
        for z in -31..31 {
            if z == 0 {
                continue
            }
            let radius = rng.sample(ClosedRange(0.1, 0.3));
            root.push(Hittable::Sphere {
                center: vector![
                    x as Real + rng.sample(ClosedRange(-0.5 + radius, 0.5 - radius)),
                    radius,
                    z as Real + rng.sample(ClosedRange(-0.5 + radius, 0.5 - radius))
                ],
                radius,
                material_id: material_table.len() as Id
            });

            let albedo = rgb(rng.gen::<Real>(), rng.gen::<Real>(), rng.gen::<Real>());
            if rng.sample(Bernoulli(0.7)) {
                material_table.push(Material::Lambert {albedo});
            } else if rng.sample(Bernoulli(0.7)) {
                material_table.push(Material::Metal {albedo, fuzziness: rng.gen::<Real>()})
            } else {
                material_table.push(Material::Dielectric {refraction_index: 1.5})
            }
        }
    }

    ExampleScene {camera, material_table, root: Hittable::List(root)}
}

#[allow(dead_code)]
pub fn more_balls_optimized() -> ExampleScene {
    let mut example_scene = more_balls();
    let list = if let Hittable::List(list) = example_scene.root {
        list
    } else {
        unreachable!()
    };
    example_scene.root = Hittable::Bvh(Bvh::new(list));
    example_scene
}