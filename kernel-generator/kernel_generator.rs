use rand::prelude::*;

// fn sphere_volume(radius: f64) -> f64 {
//     4.0/3.0*std::f64::consts::PI*radius*radius*radius
// }

// fn cube_volume(radius: f64) -> f64 {
//     8.0*radius*radius*radius
// }

fn magsq(v: [f64; 3]) -> f64 {
    v[0]*v[0] + v[1]*v[1] + v[2]*v[2]
}

pub fn generate_hbao_kernel(n_sphere: usize) -> Vec<[f32; 4]> {
    let n_cube_f64 = n_sphere as f64 * 6.0/std::f64::consts::PI;
    let n_side_f64 = n_cube_f64.cbrt().round();

    let n_side = n_side_f64 as usize;
    let n_cube = n_side.pow(3);

    let mut samples: Vec<[f64; 3]> = Vec::with_capacity(n_cube);

    let mut rng = rand::rngs::StdRng::from_seed(*b"thisismy32byteseedanditneedsalot");
    let jitter = rand::distributions::Uniform::new(-0.1, 0.1);

    for z in 0..n_side {
        for y in 0..n_side {
            for x in 0..n_side {
                samples.push([
                    (x as f64)*2.0/n_side_f64 - 1.0 + jitter.sample(&mut rng),
                    (y as f64)*2.0/n_side_f64 - 1.0 + jitter.sample(&mut rng),
                    (z as f64)*2.0/n_side_f64 - 1.0 + jitter.sample(&mut rng),
                ]);
            }
        }
    }

    samples.sort_by(|&a: &[f64; 3], &b: &[f64; 3]| magsq(a).partial_cmp(&magsq(b)).unwrap());

    let mut samples: Vec<[f32; 4]> = samples.into_iter().map(|v| [ v[0] as f32, v[1] as f32, v[2] as f32, 0.0 ]).take(n_sphere).collect();

    samples.sort_by(|&a, b| a.partial_cmp(b).unwrap());

    samples
}
