use crate::{utility::*, hittable::Hit};

// ------------------------------------------- Material -------------------------------------------

#[derive(Debug, Clone)]
pub enum Material {
    Missing,
    Lambert {albedo: TextureId},
    Metal {albedo: Color, fuzziness: Real},
    Dielectric {refraction_index: Real},
}

impl Material {
    pub fn scatter(&self, incident: &Ray, hit: &Hit, scene_data: &SceneData, rng: &mut Randomizer)
        -> Option<(Color, Ray)>
    {
        match self {
            Self::Missing => None,
            Self::Lambert {albedo} => {
                scatter_lambert(incident, hit, scene_data, rng, *albedo)
            },
            Self::Metal {albedo, fuzziness} => {
                scatter_metal(incident, hit, *albedo, *fuzziness, rng)
            },
            Self::Dielectric {refraction_index} => {
                scatter_dielectric(incident, hit, *refraction_index, rng)
            },
        }
    }
}

// ------------------------------------------- Material implementations -------------------------------------------

fn scatter_lambert(incident: &Ray, hit: &Hit, scene_data: &SceneData, rng: &mut Randomizer, albedo: TextureId)
    -> Option<(Color, Ray)>
{
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

    let albedo = scene_data.texture_table[albedo.to_index()].sample(incident, hit, scene_data, rng);

    Some((albedo, scattered))
}

fn scatter_metal(incident: &Ray, hit: &Hit, albedo: Color, fuzziness: Real, rng: &mut Randomizer) -> Option<(Color, Ray)> {
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
    Some((albedo, reflected))
}

fn scatter_dielectric(incident: &Ray, hit: &Hit, refraction_index: Real, rng: &mut Randomizer) -> Option<(Color, Ray)> {
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
    Some((rgb(1.0, 1.0, 1.0), bounce))
}