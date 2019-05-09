use kernel_generator::*;
use rand::distributions::*;
use rand::prelude::*;
use cgmath::*;

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
                let [x, y, z] = dist.sample(&mut rng);
                [x as f32, y as f32, z as f32]
            })
            .collect();

        std::fs::write(out_dir.join("unit_sphere_volume.bin"), slice_to_bytes(&values[..])).unwrap();

        let values_reflected: Vec<[f32; 3]> = values.into_iter().map(|v| {
            let v = Vector3::from(v);
            let n = Vector3::new(0.0, 1.0, 0.0);

            (v - 2.0 * Vector3::dot(n, v)*n).into()
        }).collect();

        std::fs::write(out_dir.join("unit_sphere_volume_reflected.bin"), slice_to_bytes(&values_reflected[..])).unwrap();
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

        let values_reflected: Vec<[f32; 3]> = values.into_iter().map(|n| {
            let n = Vector3::from(n);
            let v = Vector3::new(0.0, 1.0, 0.0);

            (v - 2.0 * Vector3::dot(n, v)*n).into()
        }).collect();

        std::fs::write(out_dir.join("unit_sphere_surface_reflected.bin"), slice_to_bytes(&values_reflected[..])).unwrap();
    }
}
