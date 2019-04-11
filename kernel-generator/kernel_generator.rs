use rand::prelude::*;

pub fn generate_hbao_kernel(radius: f32, length: usize) -> Vec<[f32; 4]> {
    let mut samples: Vec<[f32; 4]> = Vec::with_capacity(length);

    let mut rng = rand::rngs::StdRng::from_seed(*b"thisismy32byteseedanditneedsalot");

    // Chopped off at radius.
    let dist = rand::distributions::Normal::new(0.0, radius as f64/2.0);

    while samples.len() < length {
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
            samples.push([x, y, z, 0.0]);
        }
    }

    samples
}
