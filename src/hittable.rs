use crate::utility::*;

// ------------------------------------------- Hittable -------------------------------------------

pub enum Hittable {
    Sphere {center: Point3, radius: Real},
    List(Vec<Hittable>),
}

pub struct Hit {
    pub t: Real,
    pub position: Point3,
    pub normal: Vector3, // The normal is assumed to be a unit vector
}

impl Hittable {
    pub fn hit(&self, ray: Ray, t_min: Real, t_max: Real) -> Option<Hit> {
        match self {
            Self::Sphere {center, radius} => hit_sphere(*center, *radius, ray, t_min, t_max),
            Self::List(list) => hit_list(list, ray, t_min, t_max),
        }
    }
}

// ------------------------------------------- Hit implementations -------------------------------------------

fn hit_sphere(center: Point3, radius: Real, ray: Ray, t_min: Real, t_max: Real) -> Option<Hit> {
    let to_center = ray.b - center;
    let a = ray.a.magnitude2();
    let half_b = ray.a.dot(to_center);
    let c = to_center.magnitude2() - radius*radius;
    let delta = half_b*half_b - a*c;
    if delta <= 0.0 {
        return None
    }
    
    let sqrt_delta = delta.sqrt();
    let mut t = (-half_b - sqrt_delta) / a; // Try the closer hit
    if t < t_min || t > t_max {
        t = (-half_b + sqrt_delta) / a; // Try the further hit
        if t < t_min || t > t_max {
            return None
        }
    }

    let position = ray.at(t);
    let normal = (position - center).normalize();
    Some(Hit {t, position, normal})
}

fn hit_list(list: &[Hittable], ray: Ray, t_min: Real, mut t_max: Real) -> Option<Hit> {
    let mut hit = None;
    for x in list {
        if let Some(new_hit) = x.hit(ray, t_min, t_max) {
            t_max = new_hit.t;
            hit.replace(new_hit);
        }
    }
    hit
}