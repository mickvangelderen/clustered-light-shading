fn write_obj_quads(name: &str, vertices: &[[f32; 3]], quads: &[[u32; 4]]) -> std::io::Result<()> {
    use std::io::Write;
    let mut bufwriter =
        std::io::BufWriter::new(std::fs::File::create(format!("{}.obj", name)).unwrap());
    let f = &mut bufwriter;

    for p in vertices.iter() {
        writeln!(f, "v {} {} {}", p[0], p[1], p[2])?;
    }

    writeln!(f, "o {}", name)?;
    for q in quads.iter() {
        writeln!(f, "f {} {} {} {}", q[0] + 1, q[1] + 1, q[2] + 1, q[3] + 1)?;
    }

    Ok(())
}

trait Vector3f32 {
    fn dot(a: Self, b: Self) -> f32;
    fn magnitude_sq(self) -> f32;
    fn magnitude(self) -> f32;
    fn scale(self, scale: f32) -> [f32; 3];
}

impl Vector3f32 for [f32; 3] {
    fn dot(a: Self, b: Self) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    fn magnitude_sq(self) -> f32 {
        Vector3f32::dot(self, self)
    }

    fn magnitude(self) -> f32 {
        f32::sqrt(self.magnitude_sq())
    }

    fn scale(self, scale: f32) -> [f32; 3] {
        [self[0] * scale, self[1] * scale, self[2] * scale]
    }
}

fn normalize_to(vector: [f32; 3], magnitude: f32) -> [f32; 3] {
    vector.scale(magnitude / vector.magnitude())
}

fn main() {
    let radius = 1.0;
    for subdivisions in 0..=4 {
        let spherical = polygen::cubic_sphere_vertices(radius, subdivisions);
        let mut projected = polygen::cube_vertices(radius, subdivisions);
        for vertex in projected.iter_mut() {
            *vertex = normalize_to(*vertex, radius)
        }
        let quads = polygen::cube_quads(subdivisions);
        write_obj_quads(
            &format!("cubic_sphere_{}", subdivisions),
            &spherical,
            &quads,
        )
        .unwrap();
        write_obj_quads(
            &format!("cube_projected_onto_sphere_{}", subdivisions),
            &projected,
            &quads,
        )
        .unwrap();
    }
}
