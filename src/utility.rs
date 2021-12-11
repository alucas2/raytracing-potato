pub type Real = f64;
pub type Point2 = cgmath::Point2<Real>;
pub type Point3 = cgmath::Point3<Real>;
pub type Vector2 = cgmath::Vector2<Real>;
pub type Vector3 = cgmath::Vector3<Real>;
pub type Id = usize;

pub use color::*;
pub use random::*;
pub use rand::prelude::*;
pub use cgmath::prelude::*;
pub use cgmath::{point2, vec2, point3, vec3};
pub use std::f64::consts::*;

// ------------------------------------------- Ray -------------------------------------------

/// A line with equation a*t + b
#[derive(Clone, Copy)]
pub struct Ray {
    pub direction: Vector3,
    pub origin: Point3,
    pub attenuation: Color,
}

impl Ray {
    pub fn at(&self, t: Real) -> Point3 {
        self.origin + self.direction * t
    }
}

// ------------------------------------------- Color -------------------------------------------

mod color {
    use super::*;
    pub type Color = cgmath::Vector4<Real>;
    
    pub fn rgb(r: Real, g: Real, b: Real) -> Color {
        cgmath::vec4(r, g, b, 1.0)
    }

    pub fn to_srgb_u8(color: Color) -> [u8; 4] {
        color.map(|x| (255.0 * x.clamp(0.0, 1.0).powf(1.0/2.2)) as u8).into()
    }
}

// ------------------------------------------- Randomness -------------------------------------------

mod random {
    use super::*;
    use rand::{distributions::Distribution, Rng};

    /// A uniform distribution of vectors inside the unit ball
    pub struct UnitBall;

    impl Distribution<Vector3> for UnitBall {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vector3 {
            loop {
                let v = vec3(
                    2.0 * rng.gen::<Real>() - 1.0,
                    2.0 * rng.gen::<Real>() - 1.0,
                    2.0 * rng.gen::<Real>() - 1.0
                );

                if v.magnitude2() < 1.0 {
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
                let v = vec2(
                    2.0 * rng.gen::<Real>() - 1.0,
                    2.0 * rng.gen::<Real>() - 1.0
                );

                let s = v.magnitude2();
                if s < 1.0 {
                    let n = 2.0 * (1.0 - s).sqrt();
                    return vec3(v.x * n, v.y * n, 1.0 - 2.0 * s)
                }
            }
        }
    }
}