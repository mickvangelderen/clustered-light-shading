struct Resources {
    materials: Vec<Material>,
    geometries: Vec<Geometry>,
    textures: Vec<Texture>,
    buffers: Vec<Buffer>,
}

struct Material {
    diffuse_texture_id: usize,
    specular_texture_id: usize,
}
