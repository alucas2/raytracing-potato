/*
In this file:
- Types and constants
- Ray
- Hit
- Some math
- Bounding boxes
- Transformations
- Color
*/

// ------------------------------------------- Types and constants -------------------------------------------

pub type Real = f64; // <-- Choose here between f64 and f32
pub use std::f64::{consts::*, INFINITY}; // <-- and here as well
pub type Rvec2 = nalgebra::Vector2<Real>;
pub type Rvec3 = nalgebra::Vector3<Real>;
pub type Bvec3 = nalgebra::Vector3<bool>;
pub type Rmat3 = nalgebra::Matrix3<Real>;

/*
I do not use naglebra's fancy wrappers like Point and Unit because:
1. They are annoying to work with
2. I trust myself (a bit)
*/

pub use nalgebra::{vector, matrix};

/// Nudge the start of the ray to avoid self-intersection
pub const RAY_EPSILON: Real = 1e-3;

/// An index into the material table
#[derive(Debug, Clone, Copy)]
pub struct MaterialId(pub u32);

impl MaterialId {
    pub fn to_index(self) -> usize {self.0 as usize}
}

/// An index into the texture table
#[derive(Debug, Clone, Copy)]
pub struct TextureId(pub u32);

impl TextureId {
    pub fn to_index(self) -> usize {self.0 as usize}
}

// ------------------------------------------- Ray -------------------------------------------

/// A segment with equation b+a*t, with t ranging from t_min to t_max
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Rvec3,
    pub direction: Rvec3, // <-- Keep this vector normalized
    pub t_min: Real,
    pub t_max: Real,
}

/// A ray with some additional cached information
#[derive(Debug, Clone)]
pub struct RayExpanded {
    pub inner: Ray,
    pub inv_direction: Rvec3,
}

impl Ray {
    pub fn at(&self, t: Real) -> Rvec3 {
        self.origin + t * self.direction
    }

    pub fn expand(self) -> RayExpanded {
        let inv_direction = vector![1.0 / self.direction.x, 1.0 / self.direction.y, 1.0 / self.direction.z];
        RayExpanded {
            inner: self,
            inv_direction,
        }
    }
}

// ------------------------------------------- Hit -------------------------------------------

/// A collision between a ray and an object
#[derive(Debug, Clone)]
pub struct Hit {
    pub t: Real,
    pub position: Rvec3,
    pub normal: Rvec3, // <-- Keep this vector normalized
    pub uv: Rvec2,
}

impl Hit {
    pub fn at_infinity(direction: &Rvec3) -> Hit {
        Hit {
            t: INFINITY,
            position: direction.clone(),
            normal: direction.clone(),
            uv: vector![0.5 - direction.z.atan2(direction.x) / TAU, direction.y.asin() / PI + 0.5],
        }
    }
}

// ------------------------------------------- Some math -------------------------------------------

/// Normal must be a unit vector, then it returns a vector of the same length as incident
pub fn reflect(incident: &Rvec3, normal: &Rvec3) -> Rvec3 {
    incident - 2.0 * incident.dot(&normal) * normal
}

/// Normal and incident must be unit vectors, then it returns a unit vector
pub fn refract(incident: &Rvec3, normal: &Rvec3, eta: Real) -> Option<Rvec3> {
    let cos_theta = normal.dot(&incident);
    let k = 1.0 - eta * eta * (1.0 - cos_theta * cos_theta);
    if k < 0.0 {
        None // Total reflection
    } else {
        Some(eta * incident - (eta * cos_theta + k.sqrt()) * normal)
    }
}

// ------------------------------------------- Bounding boxes -------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct AABB {
    pub min: Rvec3,
    pub max: Rvec3,
}

impl AABB {
    pub fn union(&self, other: &AABB) -> AABB {
        AABB {
            min: vector![self.min.x.min(other.min.x), self.min.y.min(other.min.y), self.min.z.min(other.min.z)],
            max: vector![self.max.x.max(other.max.x), self.max.y.max(other.max.y), self.max.z.max(other.max.z)],
        }
    }

    pub fn collide(&self, ray: &RayExpanded) -> bool {
        // This is a hot function, optimizations are welcome
        // https://tavianator.com/2011/ray_box.html
        let t0 = (self.min - ray.inner.origin).component_mul(&ray.inv_direction);
        let t1 = (self.max - ray.inner.origin).component_mul(&ray.inv_direction);
        
        let t_min = ray.inner.t_min
            .max(t0.x.min(t1.x))
            .max(t0.y.min(t1.y))
            .max(t0.z.min(t1.z));
        
        let t_max = ray.inner.t_max
            .min(t0.x.max(t1.x))
            .min(t0.y.max(t1.y))
            .min(t0.z.max(t1.z));

        t_max > t_min
    }
}

// ------------------------------------------- Transformation -------------------------------------------

#[derive(Debug, Clone)]
pub struct Transformation {
    pub orientation: Rmat3,
    pub position: Rvec3,
}

impl Transformation {
    pub fn identity() -> Self {
        let orientation = Rmat3::identity();
        let position = Rvec3::zeros();
        Transformation {orientation, position}
    }

    pub fn lookat(position: &Rvec3, target: &Rvec3, up: &Rvec3) -> Self {
        let z = (position - target).normalize();
        let x = up.cross(&z);
        let y = z.cross(&x);
        Transformation {orientation: Rmat3::from_columns(&[x, y, z]), position: *position}
    }

    pub fn inverse(&self) -> Self {
        let inv_orientation = self.orientation.transpose();
        let inv_position = -inv_orientation * self.position;
        Transformation {orientation: inv_orientation, position: inv_position}
    }

    pub fn transform_vector(&self, vector: &Rvec3) -> Rvec3 {
        self.orientation * vector
    }

    pub fn transform_point(&self, point: &Rvec3) -> Rvec3 {
        self.orientation * point + self.position
    }
}

// ------------------------------------------- Color -------------------------------------------

pub type Color = nalgebra::Vector3<Real>;

pub fn rgb(r: Real, g: Real, b: Real) -> Color {
    vector![r, g, b]
}

pub fn to_u8(color: &Color) -> [u8; 4] {
    let clamp_and_cast = |x: Real| (255.0 * x.clamp(0.0, 1.0)) as u8;
    [
        clamp_and_cast(color.x),
        clamp_and_cast(color.y),
        clamp_and_cast(color.z),
        0xff,
    ]
}

pub fn to_srgb_u8(color: &Color) -> [u8; 4] {
    let clamp_and_gamma_correct = |x: Real| (255.0 * x.clamp(0.0, 1.0).powf(1.0/2.2)) as u8;
    [
        clamp_and_gamma_correct(color.x),
        clamp_and_gamma_correct(color.y),
        clamp_and_gamma_correct(color.z),
        0xff,
    ]
}
