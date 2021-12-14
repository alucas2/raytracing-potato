// In this file:
// 1. Types and constants
// 2. Ray
// 3. Image sampling
// 4. Specialized math
// 5. Bounding boxes
// 6. Transformations
// 7. Color
// 8. Random distributions

// ------------------------------------------- Types and constants -------------------------------------------

pub type Real = f64;
pub type Rvec2 = nalgebra::Vector2<Real>;
pub type Rvec3 = nalgebra::Vector3<Real>;
pub type Bvec3 = nalgebra::Vector3<bool>;
pub type Rmat3 = nalgebra::Matrix3<Real>;
pub type Id = u32;
pub type Randomizer = rand::rngs::StdRng;

// I do not use naglebra's fancy wrappers like Point and Unit because:
// 1. They are annoying to work with
// 2. I trust myself (a bit)

pub use nalgebra::{vector, matrix};
pub use rand::{prelude::*, Rng};
pub use std::f64::consts::*;
pub use std::f64::INFINITY;
pub const RAY_EPSILON: Real = 1e-3;

// ------------------------------------------- Ray -------------------------------------------

/// A line with equation a*t + b
#[derive(Debug, Clone)]
pub struct Ray {
    /// Ray origin
    pub origin: Rvec3,
    /// Ray direction as a unit vector
    pub direction: Rvec3,
    /// Closer extremity
    pub t_min: Real,
    /// Further extremity 
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

// ------------------------------------------- Image sampling -------------------------------------------

#[derive(Debug, Clone)]
pub struct Multisampler {
    pub width: u32,
    pub height: u32,
    pub num_samples: u32
}

impl Multisampler {
    /// Get the sample coordinates of a pixel, in the range [0, 1]
    pub fn sample(&self, i: u32, j: u32) -> Rvec2 {
        vector![
            i as Real / self.width as Real,
            j as Real / self.height as Real
        ]
    }

    /// Get multiple samples coordinates for a pixel, in the range [0, 1]
    pub fn samples_jitter(&self, i: u32, j: u32, rng: &mut Randomizer) -> impl Iterator<Item=Rvec2> + '_ {
        let mut rng = rng.clone();
        (0..self.num_samples).map(move |_| {
            vector![
                (i as Real + rng.gen::<Real>()) / self.width as Real,
                (j as Real + rng.gen::<Real>()) / self.height as Real
            ]
        })
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

pub type Color = nalgebra::Vector4<Real>;

pub fn rgb(r: Real, g: Real, b: Real) -> Color {
    nalgebra::Vector4::new(r, g, b, 1.0)
}

pub fn to_srgb_u8(color: Color) -> [u8; 4] {
    color.map(|x| (255.0 * x.clamp(0.0, 1.0).powf(1.0/2.2)) as u8).into()
}

// ------------------------------------------- Randomness -------------------------------------------

use rand::distributions::Distribution;

/// A uniform distribution inside a range
pub struct ClosedRange(pub Real, pub Real);

impl Distribution<Real> for ClosedRange {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Real {
        self.0 + rng.gen::<Real>() * (self.1 - self.0)
    }
}

/// A uniform distribution of vectors inside the unit disk
pub struct UnitDisk;

impl Distribution<Rvec2> for UnitDisk {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Rvec2 {
        loop {
            let v = vector![
                2.0 * rng.gen::<Real>() - 1.0,
                2.0 * rng.gen::<Real>() - 1.0
            ];

            if v.norm_squared() < 1.0 {
                return v
            }
        }
    }
}


/// A uniform distribution of vectors inside the unit ball
pub struct UnitBall;

impl Distribution<Rvec3> for UnitBall {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Rvec3 {
        loop {
            let v = vector![
                2.0 * rng.gen::<Real>() - 1.0,
                2.0 * rng.gen::<Real>() - 1.0,
                2.0 * rng.gen::<Real>() - 1.0
            ];

            if v.norm_squared() < 1.0 {
                return v
            }
        }
    }
}

/// A uniform distribution of vectors on the unit sphere
pub struct UnitSphere;

impl Distribution<Rvec3> for UnitSphere {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Rvec3 {
        loop {
            let v = vector![
                2.0 * rng.gen::<Real>() - 1.0,
                2.0 * rng.gen::<Real>() - 1.0
            ];

            let s = v.norm_squared();
            if s < 1.0 {
                let n = 2.0 * (1.0 - s).sqrt();
                return vector![v.x * n, v.y * n, 1.0 - 2.0 * s]
            }
        }
    }
}

/// A distribution with a probability p for true and 1-p of false
pub struct Bernoulli(pub Real);

impl Distribution<bool> for Bernoulli {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> bool {
        rng.gen::<Real>() < self.0
    }
}