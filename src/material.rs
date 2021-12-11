use crate::{utility::*, hittable::Hit};

// ------------------------------------------- Material -------------------------------------------

pub enum Material {
    Lambert {albedo: Color},
    Metal {albedo: Color},
}

impl Material {
    pub fn scatter(&self, incident: Ray, hit: Hit, rng: &mut ThreadRng) -> Option<Ray> {
        match self {
            Self::Lambert {albedo} => scatter_lambert(incident, hit, *albedo, rng),
            Self::Metal {albedo} => scatter_metal(incident, hit, *albedo, rng),
        }
    }
}

// ------------------------------------------- Material implementations -------------------------------------------

fn scatter_lambert(incident: Ray, hit: Hit, albedo: Color, rng: &mut ThreadRng) -> Option<Ray> {
    if hit.normal.dot(incident.direction) > 0.0 {
        return None
    }
    
    let scattered = Ray {
        direction: hit.normal + rng.sample(UnitSphere),
        origin: hit.position,
        attenuation: incident.attenuation.mul_element_wise(albedo)
    };
    Some(scattered)
}

fn scatter_metal(incident: Ray, hit: Hit, albedo: Color, rng: &mut ThreadRng) -> Option<Ray> {
    if hit.normal.dot(incident.direction) > 0.0 {
        return None
    }

    let reflected = Ray {
        direction: incident.direction - 2.0 * hit.normal.dot(incident.direction) * hit.normal,
        origin: hit.position,
        attenuation: incident.attenuation.mul_element_wise(albedo)
    };
    Some(reflected)
}