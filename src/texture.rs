use crate::utility::*;
use crate::render::SceneData;

// ------------------------------------------- Texture -------------------------------------------

pub enum Texture {
    Missing,
    Solid(Color),
    Checker {odd: TextureId, even: TextureId, freq: Real},
    Noise {freq: Real}
}

impl Texture {
    pub fn sample(&self, incident: &Ray, hit: &Hit, scene_data: &SceneData, rng: &mut Randomizer) -> Color {
        match self {
            Self::Missing => rgb(0.0, 0.0, 0.0),
            Self::Solid(color) => *color,
            Self::Checker {odd, even, freq}
                => sample_checker(incident, hit, scene_data, rng, *odd, *even, *freq),
            Self::Noise {freq}
                => sample_noise(incident, hit, scene_data, rng, *freq),
        }
    }
}

// ------------------------------------------- Texture implementations -------------------------------------------

pub fn sample_checker(incident: &Ray, hit: &Hit, scene_data: &SceneData, rng: &mut Randomizer, odd: TextureId,
    even: TextureId, freq: Real) -> Color
{
    let p = freq * hit.position;
    if (p.x.floor() + p.y.floor() + p.z.floor()) % 2.0 == 0.0 {
        scene_data.texture_table[even.to_index()].sample(incident, hit, scene_data, rng)
    } else {
        scene_data.texture_table[odd.to_index()].sample(incident, hit, scene_data, rng)
    }
}

pub fn sample_noise(_incident: &Ray, hit: &Hit, _scene_data: &SceneData, _rng: &mut Randomizer, freq: Real) -> Color {
    let p = freq * hit.position;
    let mut x = p.x.floor() * 12.9898 + p.y.floor() * 78.2333 + p.z.floor() * 42.5077;
    x = (x.sin() * 43758.5453).fract().abs();
    rgb(x, x, x)
}