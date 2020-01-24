use crate::*;

pub struct Particle {
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub tint: Vector3<f32>,
}

impl Particle {
    #[inline]
    pub fn spawn(rng: &mut impl Rng, bounds: Range3<f32>) -> Self {
        Self {
            position: Point3::new(
                rng.gen_range(bounds.x0, bounds.x1),
                rng.gen_range(bounds.y0, bounds.y1),
                rng.gen_range(bounds.z0, bounds.z1),
            ),
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
    pub fn update(&mut self, delta_time: f32, rng: &mut impl Rng, bounds: Range3<f32>) {
        self.position += delta_time * self.velocity;
        if !bounds.contains(self.position) {
            *self = Particle::spawn(
                rng,
                Range3 {
                    x0: bounds.x0,
                    x1: bounds.x1,
                    y0: 0.01 * bounds.y0 + 0.99 * bounds.y1,
                    y1: bounds.y1,
                    z0: bounds.z0,
                    z1: bounds.z1,
                },
            );
        }
    }
}
