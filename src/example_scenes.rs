use raytracing2::hittable::*;
use raytracing2::material::*;
use raytracing2::utility::*;
use raytracing2::bvh::*;
use raytracing2::texture::*;
use raytracing2::render::*;
use raytracing2::randomness::*;
use raytracing2::image::*;

// TODO: Have a scene verifier that detects missing texture/material and circular references?
// It would use string ids instead of integers for ease of use and to allow the merging or multiple scenes

pub struct ExampleScene {
    pub camera: Camera,
    pub scene_data: SceneData,
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

    // Table of textures
    let texture_table = vec![
        Texture::Solid(rgb(0.8, 0.8, 0.0)),
        Texture::Solid(rgb(0.1, 0.2, 0.5)),
    ];

    // Table of materials
    let material_table = vec![
        Material::Lambert {albedo: TextureId(0)},
        Material::Lambert {albedo: TextureId(1)},
        Material::Dielectric {refraction_index: 1.5},
        Material::Metal {albedo: rgb(0.8, 0.6, 0.2), fuzziness: 0.0},
    ];

    // List of objects of the scene
    let root = Hittable::List(vec![
        Hittable::Sphere {center: vector![0.0, -100.5, -1.0], radius: 100.0, material: MaterialId(0)}, // Ground
        Hittable::Sphere {center: vector![0.0, 0.0, -1.0], radius: 0.5, material: MaterialId(1)}, // Diffuse sphere
        Hittable::Sphere {center: vector![-1.0, 0.0, -1.0], radius: 0.5, material: MaterialId(2)}, // Metal sphere
        Hittable::Sphere {center: vector![1.0, 0.0, -1.0], radius: 0.5, material: MaterialId(3)}, // Glass sphere
    ]);

    let scene_data = SceneData {material_table, texture_table};
    ExampleScene {camera, scene_data, root}
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

    // Table of textures
    let mut texture_table = vec![
        Texture::Checker {odd: TextureId(2), even: TextureId(3)},
        Texture::Solid(rgb(0.1, 0.2, 0.5)),
        Texture::Solid(rgb(0.2, 0.3, 0.1)),
        Texture::Solid(rgb(0.9, 0.9, 0.9))
    ];

    // Table of materials
    let mut material_table = vec![
        Material::Lambert {albedo: TextureId(0)},
        Material::Lambert {albedo: TextureId(1)},
        Material::Metal {albedo: rgb(0.8, 0.6, 0.2), fuzziness: 0.0},
        Material::Dielectric {refraction_index: 1.5},
    ];

    // List of objects of the scene
    let mut root = vec![
        Hittable::Sphere {center: vector![0.0, -1000.0, -1.0], radius: 1000.0, material: MaterialId(0)}, // Ground
        Hittable::Sphere {center: vector![-4.0, 1.8, 0.0], radius: 1.8, material: MaterialId(1)}, // Diffuse sphere
        Hittable::Sphere {center: vector![4.0, 1.8, 0.0], radius: 1.8, material: MaterialId(2)}, // Metal sphere
        Hittable::Sphere {center: vector![0.0, 1.8, 0.0], radius: 1.8, material: MaterialId(3)}, // Glass sphere
    ];
    let mut rng = Randomizer::from_seed([249; 32]);
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
                material: MaterialId(material_table.len() as _)
            });

            let albedo = rgb(rng.gen::<Real>(), rng.gen::<Real>(), rng.gen::<Real>());
            if rng.sample(Bernoulli(0.7)) {
                let texture_id = TextureId(texture_table.len() as _);
                texture_table.push(Texture::Solid(albedo));
                material_table.push(Material::Lambert {albedo: texture_id});
            } else if rng.sample(Bernoulli(0.7)) {
                material_table.push(Material::Metal {albedo, fuzziness: rng.gen::<Real>()})
            } else {
                material_table.push(Material::Dielectric {refraction_index: 1.5})
            }
        }
    }

    let scene_data = SceneData {material_table, texture_table};
    ExampleScene {camera, scene_data, root: Hittable::List(root)}
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

#[allow(dead_code)]
pub fn two_balls() -> ExampleScene {
    let camera = Camera {
        aspect_ratio: 1.0,
        fov: FRAC_PI_2,
        focal_dist: 7.5,
        lens_radius: 0.0,
        transformation: Transformation::lookat(
            &vector![6.0, 0.0, 4.0],
            &vector![0.0, 0.0, 0.0],
            &vector![0.0, 1.0, 0.0]
        ),
    };

    let texture_table = vec![
        Texture::Solid(rgb(0.2, 0.2, 0.2)),
        Texture::Solid(rgb(0.9, 0.0, 0.5)),
        Texture::Checker {odd: TextureId(0), even: TextureId(1)},
        Texture::Perlin {seed: 0},
    ];

    let material_table = vec![
        Material::Lambert {albedo: TextureId(2)},
        Material::Lambert {albedo: TextureId(3)}
    ];

    let root = Hittable::Bvh(Bvh::new(vec![
        Hittable::Sphere {center: vector![0.0, -10.0, 0.0], radius: 10.0, material: MaterialId(0)},
        Hittable::Sphere {center: vector![0.0, 10.0, 0.0], radius: 10.0, material: MaterialId(1)},
    ]));

    let scene_data = SceneData {material_table, texture_table};
    ExampleScene {camera, scene_data, root}
}

#[allow(dead_code)]
pub fn earth() -> ExampleScene {
    let camera = Camera {
        aspect_ratio: 1.0,
        fov: PI / 9.0,
        focal_dist: 1.0,
        lens_radius: 0.0,
        transformation: Transformation::lookat(
            &vector![13.0, 7.0, 3.0],
            &vector![0.0, 0.0, 0.0],
            &vector![0.0, 1.0, 0.0]
        ),
    };

    let texture_table = vec![
        Texture::Image(tga::load("assets/earthmap.tga").unwrap())
    ];

    let material_table = vec![
        Material::Lambert {albedo: TextureId(0)}
    ];

    let root = Hittable::Bvh(Bvh::new(vec![
        Hittable::Sphere {center: vector![0.0, 0.0, 0.0], radius: 2.0, material: MaterialId(0)}
    ]));

    let scene_data = SceneData {material_table, texture_table};
    ExampleScene {camera, root, scene_data}
}