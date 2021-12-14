use crate::{utility::*, hittable::Hit};

pub enum Texture {
    Missing,
    Solid(Color),
    Checker {odd: TextureId, even: TextureId}
}

impl Texture {
    pub fn sample(&self, incident: &Ray, hit: &Hit, scene_data: &SceneData, rng: &mut Randomizer) -> Color {
        match self {
            Self::Missing => rgb(0.0, 0.0, 0.0),
            Self::Solid(color) => *color,
            Self::Checker {odd, even} => {
                let p = 10.0 * hit.position;
                match p.x.sin() * p.y.sin() * p.z.sin() > 0.0 {
                    true => scene_data.texture_table[even.to_index()].sample(incident, hit, scene_data, rng),
                    false => scene_data.texture_table[odd.to_index()].sample(incident, hit, scene_data, rng),
                }
            }
        }
    }
}