use polygen;

fn main() {
    let subdivisions = 4000;
    let triangles = polygen::cube_tris(subdivisions);
    println!("triangles: {}", triangles.len());
    let vertices = polygen::cubic_sphere_vertices(5.0, subdivisions);
    println!("vertices: {}", vertices.len());
    let normals = polygen::compute_normals(&triangles, &vertices);
    println!("{:?}, {:?}", normals[0], normals[normals.len() - 1]);
}
