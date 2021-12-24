use crate::utility::*;
use crate::randomness::*;
use crate::render::SceneData;

// ------------------------------------------- Texture -------------------------------------------

pub enum Texture {
    Missing,
    Solid(Color),
    Checker {odd: TextureId, even: TextureId},
    Noise {seed: isize},
    Perlin {seed: isize},
}

impl Texture {
    pub fn sample(&self, incident: &Ray, hit: &Hit, scene_data: &SceneData, rng: &mut Randomizer) -> Color {
        match self {
            Self::Missing => rgb(0.0, 0.0, 0.0),
            Self::Solid(color) => *color,
            Self::Checker {odd, even}
                => sample_checker(incident, hit, scene_data, rng, *odd, *even),
            Self::Noise {seed}
                => sample_noise(incident, hit, scene_data, rng, *seed),
            Self::Perlin {seed}
                => sample_perlin(incident, hit, scene_data, rng, *seed),
        }
    }
}

// ------------------------------------------- Texture implementations -------------------------------------------

pub fn sample_checker(incident: &Ray, hit: &Hit, scene_data: &SceneData, rng: &mut Randomizer, odd: TextureId,
    even: TextureId) -> Color
{
    let p = hit.position;
    if (p.x.floor() + p.y.floor() + p.z.floor()) % 2.0 == 0.0 {
        scene_data.texture_table[even.to_index()].sample(incident, hit, scene_data, rng)
    } else {
        scene_data.texture_table[odd.to_index()].sample(incident, hit, scene_data, rng)
    }
}

pub fn sample_noise(_incident: &Ray, hit: &Hit, _scene_data: &SceneData, _rng: &mut Randomizer, seed: isize) -> Color
{
    let p = hit.position;
    let mut x = noise::real(p.x.floor() as isize, p.y.floor() as isize, p.z.floor() as isize, seed);
    x = 0.5 * x + 0.5;
    rgb(x, x, x)
}

fn grad_dot(p: &Rvec3, corner_x: isize, corner_y: isize, corner_z: isize, seed: isize) -> Real {
    let grad = vector![
        noise::real(corner_x, corner_y, corner_z, seed + 1),
        noise::real(corner_x, corner_y, corner_z, seed + 2),
        noise::real(corner_x, corner_y, corner_z, seed + 3)
    ];
    (p - vector![corner_x as Real, corner_y as Real, corner_z as Real]).dot(&grad)
}

fn mix(a: Real, b: Real, t: Real) -> Real {
    (b - a) * t + a
}

pub fn sample_perlin(_incident: &Ray, hit: &Hit, _scene_data: &SceneData, _rng: &mut Randomizer, seed: isize) -> Color
{
    let p = hit.position;
    let fp = p.map(|x| x.floor());
    let fl_x = fp.x as isize;
    let fl_y = fp.y as isize;
    let fl_z = fp.z as isize;
    let cl_x = fl_x + 1;
    let cl_y = fl_y + 1;
    let cl_z = fl_z + 1;

    // Dot product with the gradients at the corners
    let k1 = grad_dot(&p, fl_x, fl_y, fl_z, seed);
    let k2 = grad_dot(&p, cl_x, fl_y, fl_z, seed);
    let k3 = grad_dot(&p, fl_x, cl_y, fl_z, seed);
    let k4 = grad_dot(&p, cl_x, cl_y, fl_z, seed);
    let k5 = grad_dot(&p, fl_x, fl_y, cl_z, seed);
    let k6 = grad_dot(&p, cl_x, fl_y, cl_z, seed);
    let k7 = grad_dot(&p, fl_x, cl_y, cl_z, seed);
    let k8 = grad_dot(&p, cl_x, cl_y, cl_z, seed);

    // Smootherstep
    let mut t = p - fp;
    t = t.map(|t| (t * (t * 6.0 - 15.0) + 10.0) * t * t * t);

    // Trilinear interpolation
    let k12 =       mix(k1,     k2,     t.x);
    let k34 =       mix(k3,     k4,     t.x);
    let k56 =       mix(k5,     k6,     t.x);
    let k78 =       mix(k7,     k8,     t.x);
    let k1234 =     mix(k12,    k34,    t.y);
    let k5678 =     mix(k56,    k78,    t.y);
    let k12345678 = mix(k1234,  k5678,  t.z);

    let x = 0.5 * k12345678 + 0.5;
    rgb(x, x, x)
}