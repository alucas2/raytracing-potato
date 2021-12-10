pub type Real = f64;
pub type Point2 = cgmath::Point2<Real>;
pub type Point3 = cgmath::Point3<Real>;
pub type Vector2 = cgmath::Vector2<Real>;
pub type Vector3 = cgmath::Vector3<Real>;

pub use color::*;
pub use cgmath::prelude::*;
pub use cgmath::{point2, vec2, point3, vec3};
pub use std::f64::consts::*;

// ------------------------------------------- Ray -------------------------------------------

/// A line with equation a*t + b
#[derive(Clone, Copy)]
pub struct Ray {
    pub a: Vector3,
    pub b: Point3,
}

impl Ray {
    pub fn at(&self, t: Real) -> Point3 {
        self.b + self.a * t
    }
}

// ------------------------------------------- Color -------------------------------------------

mod color {
    use super::*;
    pub type Color = cgmath::Vector4<Real>;
    
    pub fn rgb(r: Real, g: Real, b: Real) -> Color {
        cgmath::vec4(r, g, b, 1.0)
    }

    pub fn to_u8(color: Color) -> [u8; 4] {
        color.map(|x| (255.0 * x.clamp(0.0, 1.0)) as u8).into()
    }
}
