use crate::*;

pub struct Particle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub tint: Vector3<f32>,
}

impl Particle {
    #[inline]
    pub fn spawn(rng: &mut impl Rng, p0: Point3<f32>, p1: Point3<f32>) -> Self {
        Self {
            position: Vector3::new(rng.gen_range(p0.x, p1.x), rng.gen_range(p0.y, p1.y), rng.gen_range(p0.z, p1.z)),
            velocity: Vector3::new(
                rng.gen_range(0.2, 1.2),
                rng.gen_range(-20.0, -14.0),
                rng.gen_range(1.0, 2.3),
            ),
            tint: Vector3::new(
                rng.gen_range(0.5, 1.0),
                rng.gen_range(0.5, 1.0),
                rng.gen_range(0.5, 1.0),
            ),
        }
    }


    #[inline]
    pub fn update(&mut self, delta_time: f32, rng: &mut impl Rng, p0: Point3<f32>, p1: Point3<f32>) {
        self.position += delta_time * self.velocity;
        if self.position.y < p0.y {
            *self = Particle::spawn(rng, p0, p1);
        }
    }
}
