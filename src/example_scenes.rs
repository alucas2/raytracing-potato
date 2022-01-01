use raytracing2::hittable::*;
use raytracing2::material::*;
use raytracing2::utility::*;
use raytracing2::bvh::*;
use raytracing2::texture::*;
use raytracing2::render::*;
use raytracing2::randomness::*;
use raytracing2::image::*;
use raytracing2::mesh::*;

// TODO: Have a scene verifier that detects missing texture/material and circular references?
// It would use string ids instead of integers for ease of use and to allow the merging or multiple scenes

pub struct ExampleScene {
    pub camera: Camera,
    pub scene_data: SceneData,
    pub root: Hittable,
    pub background: Emit,
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
        Material::new(Scatter::Lambert, Absorb::AlbedoMap(TextureId(0)), Emit::None),
        Material::new(Scatter::Lambert, Absorb::AlbedoMap(TextureId(1)), Emit::None),
        Material::new(Scatter::Dielectric {refraction_index: 1.5}, Absorb::WhiteBody, Emit::None),
        Material::new(Scatter::Metal {fuzziness: 0.0}, Absorb::Albedo(rgb(0.8, 0.6, 0.2)), Emit::None),
    ];

    // List of objects of the scene
    let root = Hittable::List(vec![
        Hittable::Sphere {center: vector![0.0, -100.5, -1.0], radius: 100.0, material: MaterialId(0)}, // Ground
        Hittable::Sphere {center: vector![0.0, 0.0, -1.0], radius: 0.5, material: MaterialId(1)}, // Diffuse sphere
        Hittable::Sphere {center: vector![-1.0, 0.0, -1.0], radius: 0.5, material: MaterialId(2)}, // Metal sphere
        Hittable::Sphere {center: vector![1.0, 0.0, -1.0], radius: 0.5, material: MaterialId(3)}, // Glass sphere
    ]);

    let scene_data = SceneData {material_table, texture_table, mesh_table: Vec::new()};
    let background = Emit::SkyGradient;
    ExampleScene {camera, scene_data, root, background}
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
    let texture_table = vec![
        Texture::Checker {odd: TextureId(1), even: TextureId(2)},
        Texture::Solid(rgb(0.2, 0.3, 0.1)),
        Texture::Solid(rgb(0.9, 0.9, 0.9))
    ];

    // Table of materials
    let mut material_table = vec![
        Material::new(Scatter::Lambert, Absorb::AlbedoMap(TextureId(0)), Emit::None),
        Material::new(Scatter::Lambert, Absorb::Albedo(rgb(0.1, 0.2, 0.5)), Emit::None),
        Material::new(Scatter::Metal {fuzziness: 0.0}, Absorb::Albedo(rgb(0.8, 0.6, 0.2)), Emit::None),
        Material::new(Scatter::Dielectric {refraction_index: 1.5}, Absorb::WhiteBody, Emit::None),
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
                // Random lambert material
                material_table.push(Material::new(
                    Scatter::Lambert, Absorb::Albedo(albedo), Emit::None
                ));
            } else if rng.sample(Bernoulli(0.7)) {
                // Random metal
                material_table.push(Material::new(
                    Scatter::Metal {fuzziness: rng.gen::<Real>()}, Absorb::Albedo(albedo), Emit::None
                ));
            } else {
                // Glass
                material_table.push(Material::new(
                    Scatter::Dielectric {refraction_index: 1.5}, Absorb::WhiteBody, Emit::None
                ));
            }
        }
    }

    let scene_data = SceneData {material_table, texture_table, mesh_table: Vec::new()};
    let background = Emit::SkyGradient;
    ExampleScene {camera, scene_data, root: Hittable::List(root), background}
}

#[allow(dead_code)]
pub fn more_balls_optimized() -> ExampleScene {
    let mut example_scene = more_balls();
    let list = if let Hittable::List(list) = example_scene.root {
        list
    } else {
        unreachable!()
    };
    example_scene.root = Hittable::Bvh(Bvh::new(list, &example_scene.scene_data));
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
        Material::new(Scatter::Lambert, Absorb::AlbedoMap(TextureId(2)), Emit::None),
        Material::new(Scatter::Lambert, Absorb::AlbedoMap(TextureId(3)), Emit::None),
    ];

    let scene_data = SceneData {material_table, texture_table, mesh_table: Vec::new()};

    let root = Hittable::Bvh(Bvh::new(vec![
        Hittable::Sphere {center: vector![0.0, -10.0, 0.0], radius: 10.0, material: MaterialId(0)},
        Hittable::Sphere {center: vector![0.0, 10.0, 0.0], radius: 10.0, material: MaterialId(1)},
    ], &scene_data));

    let background = Emit::SkyGradient;
    ExampleScene {camera, scene_data, root, background}
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
        Material::new(Scatter::Lambert, Absorb::AlbedoMap(TextureId(0)), Emit::None)
    ];

    let scene_data = SceneData {material_table, texture_table, mesh_table: Vec::new()};
    
    let root = Hittable::Bvh(Bvh::new(vec![
        Hittable::Sphere {center: vector![0.0, 0.0, 0.0], radius: 2.0, material: MaterialId(0)}
    ], &scene_data));

    let background = Emit::SkyGradient;
    ExampleScene {camera, root, scene_data, background}
}

#[allow(dead_code)]
pub fn one_triangle() -> ExampleScene {
    let normal = vector![1.0, 1.0, 1.0].normalize();
    let uv = vector![0.0, 0.0];

    let material_table = vec![
        Material::new(Scatter::None, Absorb::BlackBody, Emit::DebugNormals),
        Material::new(Scatter::Lambert, Absorb::Albedo(rgb(0.1, 0.2, 0.5)), Emit::None)
    ];

    let mesh_table = vec![
        Mesh {
            vertices: vec![
                Vertex {position: vector![1.0, 0.0, 0.0], normal, uv},
                Vertex {position: vector![0.0, 1.0, 0.0], normal, uv},
                Vertex {position: vector![0.0, 0.0, 1.0], normal, uv},
            ],
            indices: vec![0, 1, 2],
            material: MaterialId(0)
        }
    ];

    let scene_data = SceneData {material_table, mesh_table, texture_table: Vec::new()};
    let root = Hittable::Bvh(Bvh::new(vec![
        Hittable::Triangle {triangle: TriangleId(0), mesh: MeshId(0)}, // One lone triangle
        Hittable::Sphere {center: vector![0.0, -1000.0, -1.0], radius: 1000.0, material: MaterialId(1)}, // Ground
    ], &scene_data));
    let background = Emit::SkyGradient;
    let camera = Camera {
        aspect_ratio: 1.0,
        fov: FRAC_PI_2,
        focal_dist: 1.0,
        lens_radius: 0.0,
        transformation: Transformation::lookat(
            &vector![2.0, 0.5, 1.0],
            &vector![0.0, 0.0, 0.0],
            &vector![0.0, 1.0, 0.0]
        ),
    };

    ExampleScene {root, camera, scene_data, background}
}

#[allow(dead_code)]
pub fn glass_bunny() -> ExampleScene {
    let bunny = obj::load("assets/bunny_flat.obj").unwrap();
    let mut hittable_list = Vec::new();

    let material_table = vec![
        Material::new(Scatter::Dielectric {refraction_index: 1.5}, Absorb::Albedo(rgb(0.7, 0.8, 0.7)), Emit::None),
        Material::new(Scatter::Metal {fuzziness: 0.05}, Absorb::Albedo(rgb(0.8, 0.8, 0.8)), Emit::None)
    ];

    let texture_table = vec![
        Texture::Image(tga::load("assets/sky_panorama.tga").unwrap())
    ];

    hittable_list.extend(
        bunny.iter_triangles().map(|tid| Hittable::Triangle {triangle: tid, mesh: MeshId(0)})
    );
    hittable_list.push(
        Hittable::Sphere {center: vector![0.0, -1000.0, -1.0], radius: 1000.0, material: MaterialId(1)}
    );

    let mesh_table = vec![
        bunny
    ];

    let scene_data = SceneData {material_table, mesh_table, texture_table};
    let root = Hittable::Bvh(Bvh::new(hittable_list, &scene_data));
    // let root = Hittable::List(hittable_list); // OOH THAT'S SLOW
    let background = Emit::SkySphere(TextureId(0));
    let camera = Camera {
        aspect_ratio: 1.0,
        fov: FRAC_PI_4,
        focal_dist: 1.0,
        lens_radius: 0.0,
        transformation: Transformation::lookat(
            &vector![-1.5, 1.5, 2.5],
            &vector![0.0, 0.5, 0.0],
            &vector![0.0, 1.0, 0.0]
        ),
    };

    ExampleScene {root, camera, scene_data, background}
}

#[allow(dead_code)]
pub fn bunny() -> ExampleScene {
    let bunny = obj::load("assets/bunny.obj").unwrap();
    let mut hittable_list = Vec::new();

    let material_table = vec![
        Material::new(Scatter::None, Absorb::BlackBody, Emit::DebugNormals),
        Material::new(Scatter::Metal {fuzziness: 0.05}, Absorb::Albedo(rgb(0.8, 0.8, 0.8)), Emit::None)
    ];

    let texture_table = vec![
        Texture::Image(tga::load("assets/sky_panorama.tga").unwrap())
    ];

    hittable_list.extend(
        bunny.iter_triangles().map(|tid| Hittable::Triangle {triangle: tid, mesh: MeshId(0)})
    );
    hittable_list.push(
        Hittable::Sphere {center: vector![0.0, -1000.0, -1.0], radius: 1000.0, material: MaterialId(1)}
    );

    let mesh_table = vec![
        bunny
    ];

    let scene_data = SceneData {material_table, mesh_table, texture_table};
    let root = Hittable::Bvh(Bvh::new(hittable_list, &scene_data));
    // let root = Hittable::List(hittable_list); // OOH THAT'S SLOW
    let background = Emit::SkySphere(TextureId(0));
    let camera = Camera {
        aspect_ratio: 1.0,
        fov: FRAC_PI_4,
        focal_dist: 1.0,
        lens_radius: 0.0,
        transformation: Transformation::lookat(
            &vector![-1.5, 1.5, 2.5],
            &vector![0.0, 0.5, 0.0],
            &vector![0.0, 1.0, 0.0]
        ),
    };

    ExampleScene {root, camera, scene_data, background}
}