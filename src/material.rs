use crate::{utility::*, hittable::Hit};

// ------------------------------------------- Material -------------------------------------------

pub enum Material {
    Lambert {albedo: Color},
    Metal {albedo: Color, fuzziness: Real},
}

impl Material {
    pub fn scatter(&self, incident: Ray, hit: Hit, rng: &mut ThreadRng) -> Option<Ray> {
        match self {
            Self::Lambert {albedo} => scatter_lambert(incident, hit, *albedo, rng),
            Self::Metal {albedo, fuzziness} => scatter_metal(incident, hit, *albedo, *fuzziness, rng),
        }
    }
}

// ------------------------------------------- Material implementations -------------------------------------------

fn scatter_lambert(incident: Ray, hit: Hit, albedo: Color, rng: &mut ThreadRng) -> Option<Ray> {
    if hit.normal.dot(incident.direction) > 0.0 {
        return None
    }
    
    // Compute the scatter direction with lambertian distribution
    let scatter_dir = hit.normal + rng.sample(UnitSphere);
    
    let scattered = Ray {
        direction: scatter_dir,
        origin: hit.position,
        attenuation: incident.attenuation.mul_element_wise(albedo)
    };
    Some(scattered)
}

fn scatter_metal(incident: Ray, hit: Hit, albedo: Color, fuzziness: Real, rng: &mut ThreadRng) -> Option<Ray> {
    if hit.normal.dot(incident.direction) > 0.0 {
        return None
    }

    // Compute the reflected direction and add random fuzziness
    let mut reflect_dir = incident.direction - 2.0 * hit.normal.dot(incident.direction) * hit.normal;
    reflect_dir += fuzziness * reflect_dir.magnitude() * rng.sample(UnitBall);

    // Check that the fuzziness did not push the ray below the surface
    if hit.normal.dot(reflect_dir) < 0.0 {
        return None
    }

    let reflected = Ray {
        direction: reflect_dir,
        origin: hit.position,
        attenuation: incident.attenuation.mul_element_wise(albedo)
    };
    Some(reflected)
}