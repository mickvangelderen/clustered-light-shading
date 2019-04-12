use rand::prelude::*;

pub fn generate_hbao_kernel(out: &mut [[f32; 4]], radius: f32) {
    let mut rng = rand::rngs::StdRng::from_seed(*b"thisismy32byteseedanditneedsalot");

    // Chopped off at radius.
    let dist = rand::distributions::Normal::new(0.0, radius as f64/2.0);

    for v in out.iter_mut() {
        loop {
            let mut x;
            loop {
                x = dist.sample(&mut rng) as f32;
                if x > radius || x < -radius {
                    continue;
                }
                break;
            }
            let mut y;
            loop {
                y = dist.sample(&mut rng) as f32;
                if y > radius || y < -radius {
                    continue;
                }
                break;
            }
            let mut z;
            loop {
                z = (dist.sample(&mut rng) as f32).abs();
                if z > radius {
                    continue;
                }
                break;
            }
            if x * x + y * y + z * z < radius * radius {
                *v = [x, y, z, 0.0];
                break;
            }
        }
    }
}

pub fn generate_unit_vectors(out: &mut [[f32; 4]]) {
    let mut rng = rand::rngs::StdRng::from_seed(*b"anotheronebitesthedust-ahhhhhhhh");
    let dist = rand::distributions::Uniform::new_inclusive(-1.0f64, 1.0f64);

    for v in out.iter_mut() {
        loop {
            let x = dist.sample(&mut rng);
            let y = dist.sample(&mut rng);
            let z = dist.sample(&mut rng);
            let r = (x*x + y*y + z*z).sqrt();
            if r > 0.1 && r <= 1.0 {
                *v = [(x/r) as f32, (y/r) as f32, (z/r) as f32, 0.0];
                break;
            }
        }
    }
}
