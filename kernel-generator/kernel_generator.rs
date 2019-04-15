pub use rand;

use rand::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct UnitSphereVolume;

impl UnitSphereVolume {
    /// Construct a new `UnitSphereVolume` distribution.
    #[inline]
    pub fn new() -> UnitSphereVolume {
        UnitSphereVolume
    }
}

impl Distribution<[f64; 3]> for UnitSphereVolume {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f64; 3] {
        let uniform = rand::distributions::Uniform::new(-1., 1.);
        loop {
            let p = [
                uniform.sample(rng),
                uniform.sample(rng),
                uniform.sample(rng),
            ];
            let [x, y, z] = p;
            if x * x + y * y + z * z >= 1. {
                continue;
            }
            return p;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HitCircle;

impl HitCircle {
    /// Construct a new `HitCircle` distribution.
    #[inline]
    pub fn new() -> HitCircle {
        HitCircle
    }
}

impl Distribution<f64> for HitCircle {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let uniform = rand::distributions::Uniform::new(-1.0, 1.0);
        loop {
            let [x, y] = [uniform.sample(rng), uniform.sample(rng)];
            if x * x + y * y >= 1.0 {
                continue;
            }
            return x;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Quadr;

impl Quadr {
    /// Construct a new `Quadr` distribution.
    #[inline]
    pub fn new() -> Quadr {
        Quadr
    }
}

impl Distribution<f64> for Quadr {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let n1p1 = rand::distributions::Uniform::new(-1.0, 1.0);
        let z0p1 = rand::distributions::Uniform::new(0.0, 1.0);
        loop {
            let [x, y] = [n1p1.sample(rng), z0p1.sample(rng)];
            if x * x >= y {
                continue;
            }
            return x;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UnitSphereDense;

impl UnitSphereDense {
    /// Construct a new `UnitSphereDense` distribution.
    #[inline]
    pub fn new() -> UnitSphereDense {
        UnitSphereDense
    }
}

impl Distribution<[f64; 3]> for UnitSphereDense {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f64; 3] {
        let pdist = UnitSphereVolume::new();
        let mdist = HitCircle::new();

        let [x, y, z] = pdist.sample(rng);
        let m = mdist.sample(rng);
        return [m * x, m * y, m * z];
    }
}
