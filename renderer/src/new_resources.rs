pub struct Resources {
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub buffers: Vec<Buffer>,
    pub meshes: Vec<Mesh>,
    pub instances: Vec<Instance>,
}

pub struct Material {
    pub diffuse_texture_index: usize,
    pub specular_texture_index: usize,
}

pub struct Texture {
    pub path: PathBuf,
    pub dimensions: Vector2<u32>,
    pub 
}

pub struct Mesh {
    pub buffer_index: usize,
    pub byte_offset: usize,
    pub element_count: usize,
}

pub struct Instance {
    pub transform: Transform,
    pub mesh_index: usize,
}

pub struct Transform {
    pub pos: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub ori: Quaternion<f32>,
}
