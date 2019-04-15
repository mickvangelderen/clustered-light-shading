use kernel_generator::*;
use rand::distributions::*;
use rand::prelude::*;

fn slice_to_bytes<T>(s: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(s.as_ptr() as *const u8, std::mem::size_of_val(s)) }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../kernel-generator");

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let mut rng = rand::rngs::StdRng::from_seed(*b"thisismy32byteseedanditneedstime");
    {
        let dist = UnitSphereSurface::new();
        let values: Vec<[u8; 3]> = (0..256 * 256)
            .into_iter()
            .map(|_| {
                let p = dist.sample(&mut rng);
                [to_byte(p[0]), to_byte(p[1]), to_byte(p[2])]
            })
            .collect();
        std::fs::write(
            out_dir.join("unit_sphere_surface.bin"),
            slice_to_bytes(&values[..]),
        )
        .unwrap();

        fn to_byte(val: f64) -> u8 {
            (val * (255.0 / 2.0) + (255.0 / 2.0)) as u8
        }
    }
    {
        let dist = UnitSphereDense::new();
        let values: Vec<[f32; 4]> = (0..128)
            .into_iter()
            .map(|_| {
                let p = dist.sample(&mut rng);
                [p[0] as f32, p[1] as f32, p[2] as f32, 0.0]
            })
            .collect();
        std::fs::write(
            out_dir.join("unit_sphere_volume.bin"),
            slice_to_bytes(&values[..]),
        )
        .unwrap();
    }
}
