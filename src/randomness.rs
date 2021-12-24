use crate::utility::*;
pub use rand::{prelude::*, Rng};
use rand::distributions::Distribution;

pub type Randomizer = rand::rngs::StdRng; 

// ------------------------------------------- Random distributions -------------------------------------------

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

// ------------------------------------------- Noise -------------------------------------------

pub mod noise {
    use super::*;

    /// Generates an integer in the range [isize::MIN, isize::MAX]
    // http://libnoise.sourceforge.net/noisegen/index.html#coherentnoise
    pub fn integer(x: isize, y: isize, z: isize, seed: isize) -> isize {
        use std::num::Wrapping as W;
        const A: W<isize> = W(0x369E6D3B899E43CF);
        const B: W<isize> = W(0x53F89E7FFDA3B07D);
        const C: W<isize> = W(0x3B13C1CA4937E629);
        const D: W<isize> = W(0x577C2C6E4019D645);
        const E: W<isize> = W(60493);
        const F: W<isize> = W(19990303);
        const G: W<isize> = W(1376312589);

        let mut h = A * W(x) + B * W(y) + C * W(z) + D * W(seed);
        h = (h >> 13) ^ h;
        h = h * (h * h * E + F) + G;
        h.0
    }

    /// Generates a real number in the range [-1, 1]
    pub fn real(x: isize, y: isize, z: isize, seed: isize) -> Real {
        integer(x, y, z, seed) as Real / std::isize::MAX as Real
    }
}