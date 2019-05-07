use kernel_generator::*;
use rand::distributions::*;
use rand::prelude::*;

fn slice_to_bytes<T>(s: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(s.as_ptr() as *const u8, std::mem::size_of_val(s)) }
}

fn main() {
    let out_dir = std::path::PathBuf::from(".");

    let mut rng = rand::rngs::StdRng::from_seed(*b"thisismy32byteseedanditneedstime");
    {
        let dist = UnitSphereVolume::new();
        let values: Vec<[f32; 3]> = (0..1024)
            .into_iter()
            .map(|_| {
                let p = dist.sample(&mut rng);
                [p[0] as f32, p[1] as f32, p[2] as f32]
            })
            .collect();
        std::fs::write(out_dir.join("unit_sphere_volume.bin"), slice_to_bytes(&values[..])).unwrap();
    }
    {
        let dist = UnitSphereDense::new();
        let values: Vec<[f32; 3]> = (0..1024)
            .into_iter()
            .map(|_| {
                let p = dist.sample(&mut rng);
                [p[0] as f32, p[1] as f32, p[2] as f32]
            })
            .collect();
        std::fs::write(out_dir.join("unit_sphere_dense.bin"), slice_to_bytes(&values[..])).unwrap();
    }
    {
        let dist = UnitSphereSurface::new();
        let values: Vec<[f32; 3]> = (0..1024)
            .into_iter()
            .map(|_| {
                let p = dist.sample(&mut rng);
                [p[0] as f32, p[1] as f32, p[2] as f32]
            })
            .collect();
        std::fs::write(out_dir.join("unit_sphere_surface.bin"), slice_to_bytes(&values[..])).unwrap();
    }
}
