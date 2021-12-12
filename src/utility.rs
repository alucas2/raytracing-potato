pub type Real = f64;
pub type Vector2 = nalgebra::Vector2<Real>;
pub type Vector3 = nalgebra::Vector3<Real>;
pub type Matrix3 = nalgebra::Matrix3<Real>;
pub type Id = u32;
pub type Randomizer = rand::rngs::StdRng;

pub use rand::{prelude::*, Rng};
pub use std::f64::consts::*;
pub use nalgebra::{vector, matrix};

// ------------------------------------------- Ray -------------------------------------------

/// A line with equation a*t + b
#[derive(Clone, Copy)]
pub struct Ray {
    /// Ray origin
    pub origin: Vector3,
    /// Ray direction as a unit vector
    pub direction: Vector3,
    /// Attenuation from the previous bounces
    pub attenuation: Color,
}

impl Ray {
    pub fn at(&self, t: Real) -> Vector3 {
        self.origin + t * self.direction
    }
}

// ------------------------------------------- Image sampling -------------------------------------------

#[derive(Clone)]
pub struct Multisampler {
    pub width: u32,
    pub height: u32,
    pub num_samples: u32
}

impl Multisampler {
    /// Get the sample coordinates of a pixel, in the range [0, 1]
    pub fn sample(&self, i: u32, j: u32) -> Vector2 {
        vector![
            i as Real / self.width as Real,
            j as Real / self.height as Real
        ]
    }

    /// Get multiple samples coordinates for a pixel, in the range [0, 1]
    pub fn samples_jitter(&self, i: u32, j: u32, rng: & mut Randomizer) -> impl Iterator<Item=Vector2> + '_ {
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
pub fn reflect(incident: &Vector3, normal: &Vector3) -> Vector3 {
    incident - 2.0 * incident.dot(&normal) * normal
}

/// Normal and incident must be unit vectors, then it returns a unit vector
pub fn refract(incident: &Vector3, normal: &Vector3, eta: Real) -> Option<Vector3> {
    let cos_theta = normal.dot(&incident);
    let k = 1.0 - eta * eta * (1.0 - cos_theta * cos_theta);
    if k < 0.0 {
        None // Total reflection
    } else {
        Some(eta * incident - (eta * cos_theta + k.sqrt()) * normal)
    }
}

// ------------------------------------------- Transformation -------------------------------------------

#[derive(Clone)]
pub struct Transformation {
    pub orientation: Matrix3,
    pub position: Vector3,
}

impl Transformation {
    pub fn identity() -> Self {
        let orientation = Matrix3::identity();
        let position = Vector3::zeros();
        Transformation {orientation, position}
    }

    pub fn lookat(position: &Vector3, target: &Vector3, up: &Vector3) -> Self {
        let z = (position - target).normalize();
        let x = up.cross(&z);
        let y = z.cross(&x);
        Transformation {orientation: Matrix3::from_columns(&[x, y, z]), position: *position}
    }

    pub fn inverse(&self) -> Self {
        let inv_orientation = self.orientation.transpose();
        let inv_position = -inv_orientation * self.position;
        Transformation {orientation: inv_orientation, position: inv_position}
    }

    pub fn transform_vector(&self, vector: &Vector3) -> Vector3 {
        self.orientation * vector
    }

    pub fn transform_point(&self, point: &Vector3) -> Vector3 {
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

use rand::{distributions::Distribution};

/// A uniform distribution of vectors inside the unit disk
pub struct UnitDisk;

impl Distribution<Vector2> for UnitDisk {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vector2 {
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

impl Distribution<Vector3> for UnitBall {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vector3 {
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

impl Distribution<Vector3> for UnitSphere {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vector3 {
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