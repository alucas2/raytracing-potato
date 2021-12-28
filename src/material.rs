/*
In this file:
- Scattering functions
- Absorption functions
- Emission functions
- Material = aggregate of one scattering, one absorption and one emission function
*/

use crate::utility::*;
use crate::randomness::*;
use crate::render::SceneData;

// ------------------------------------------- Scattering -------------------------------------------

#[derive(Debug, Clone)]
pub enum Scatter {
    None,
    Lambert,
    Metal {fuzziness: Real},
    Dielectric {refraction_index: Real},
}

impl Scatter {
    pub fn evaluate(&self, incident: &Ray, hit: &Hit, _scene_data: &SceneData, rng: &mut Randomizer) -> Option<Ray> {
        match self {
            Self::None => None,
            Self::Lambert => evaluate_lambert(incident, hit, rng),
            Self::Metal {fuzziness} => evaluate_metal(incident, hit, rng, *fuzziness),
            Self::Dielectric {refraction_index} => evaluate_dielectric(incident, hit, rng, *refraction_index),
        }
    }
}

// ------------------------------------------- Emission -------------------------------------------

#[derive(Debug, Clone)]
pub enum Emit {
    None,
    SkyBackground,
    Normal,
}

impl Emit {
    pub fn evaluate(&self, incident: &Ray, hit: &Hit, _scene_data: &SceneData, _rng: &mut Randomizer) -> Color {
        match self {
            Self::None => rgb(0.0, 0.0, 0.0),
            Self::Normal => hit.normal,
            Self::SkyBackground => {
                let t = 0.5 * (incident.direction.y / incident.direction.magnitude() + 1.0);
                (1.0 - t) * rgb(1.0, 1.0, 1.0) + t * rgb(0.5, 0.7, 1.0)
            }
        }
    }
}

// ------------------------------------------- Absorption -------------------------------------------

#[derive(Debug, Clone)]
pub enum Absorb {
    BlackBody,
    WhiteBody,
    Albedo(Color),
    AlbedoMap(TextureId),
}

impl Absorb {
    pub fn evaluate(&self, incident: &Ray, hit: &Hit, scene_data: &SceneData, rng: &mut Randomizer) -> Color {
        match self {
            Self::BlackBody => rgb(0.0, 0.0, 0.0),
            Self::WhiteBody => rgb(1.0, 1.0, 1.0),
            Self::Albedo(color) => *color,
            Self::AlbedoMap(tid) => scene_data.texture_table[tid.to_index()].sample(incident, hit, scene_data, rng),
        }
    }
}

// ------------------------------------------- Material -------------------------------------------

#[derive(Debug, Clone)]
pub struct Material {
    scatter: Scatter,
    absorb: Absorb,
    emit: Emit,
}

pub struct MaterialOutput {
    pub scatter: Option<Ray>,
    pub absorb: Color,
    pub emit: Color,
}

impl Material {
    pub fn new(scatter: Scatter, absorb: Absorb, emit: Emit) -> Material {
        Material {scatter, emit, absorb}
    }

    pub fn evaluate(&self, incident: &Ray, hit: &Hit, scene_data: &SceneData, rng: &mut Randomizer) -> MaterialOutput
    {
        let scatter = self.scatter.evaluate(incident, hit, scene_data, rng);
        let absorb = self.absorb.evaluate(incident, hit, scene_data, rng);
        let emit = self.emit.evaluate(incident, hit, scene_data, rng);
        MaterialOutput {scatter, emit, absorb}
    }
}

// ------------------------------------------- Scattering implementations -------------------------------------------

fn evaluate_lambert(incident: &Ray, hit: &Hit, rng: &mut Randomizer) -> Option<Ray> {
    if hit.normal.dot(&incident.direction) > 0.0 {
        return None
    }
    
    // Compute the scatter direction with lambertian distribution
    let scatter_dir = (hit.normal + rng.sample(UnitSphere)).normalize();
    
    let scattered = Ray {
        direction: scatter_dir,
        origin: hit.position,
        t_min: RAY_EPSILON,
        t_max: INFINITY,
    };
    Some(scattered)
}

fn evaluate_metal(incident: &Ray, hit: &Hit, rng: &mut Randomizer, fuzziness: Real) -> Option<Ray> {
    if hit.normal.dot(&incident.direction) > 0.0 {
        return None
    }

    // Compute the reflected direction and add random fuzziness
    let reflect_dir = (reflect(&incident.direction, &hit.normal) + fuzziness * rng.sample(UnitBall)).normalize();

    // Check that the fuzziness did not push the ray below the surface
    if hit.normal.dot(&reflect_dir) < 0.0 {
        return None
    }

    let reflected = Ray {
        direction: reflect_dir,
        origin: hit.position,
        t_min: RAY_EPSILON,
        t_max: INFINITY,
    };
    Some(reflected)
}

fn evaluate_dielectric(incident: &Ray, hit: &Hit, rng: &mut Randomizer, refraction_index: Real) -> Option<Ray> {
    let (eta, normal) = if hit.normal.dot(&incident.direction) > 0.0 {
        // Interior
        (refraction_index, -hit.normal)
    } else {
        // Exterior
        (1.0 / refraction_index, hit.normal)
    };

    let reflectance = {
        let r0 = ((1.0 - eta) / (1.0 + eta)).powi(2);
        r0 + (1.0 - r0) * (1.0 + normal.dot(&incident.direction)).powi(5)
    };

    let bounce_direction = if rng.sample(Bernoulli(reflectance)) {
        reflect(&incident.direction, &normal)
    } else {
        refract(&incident.direction, &normal, eta).unwrap_or(reflect(&incident.direction, &normal))
    };
    let bounce = Ray {
        direction: bounce_direction,
        origin: hit.position,
        t_min: RAY_EPSILON,
        t_max: INFINITY,
    };
    Some(bounce)
}