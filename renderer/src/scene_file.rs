#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct Vertex {
    pub pos_in_obj: [FiniteF32; 3],
    pub nor_in_obj: [FiniteF32; 3],
    pub pos_in_tex: [FiniteF32; 2],
}

#[derive(Debug)]
#[repr(C)]
pub struct MeshDescription {
    pub index_byte_offset: u64,
    pub vertex_offset: u32,
    pub element_count: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 3],
    pub scaling: [f32; 3],
}

#[derive(Debug)]
#[repr(C)]
pub struct TransformRelation {
    pub parent_index: u32,
    pub child_index: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct Instance {
    pub mesh_index: u32,
    pub transform_index: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct RawMaterial {
    ambient_color: [f32; 3],
    ambient_texture: Option<NonZeroU32>,
    diffuse_color: [f32; 3],
    diffuse_texture: Option<NonZeroU32>,
    specular_color: [f32; 3],
    specular_texture: Option<NonZeroU32>,
    shininess: f32,
    opacity: f32,
}

#[derive(Debug)]
#[repr(C)]
pub struct RawTexture {
    path_byte_offset: u64,
    path_byte_length: u64,
}

#[derive(Debug)]
pub struct Texture {
    path: PathBuf,
}

#[derive(Debug)]
#[repr(C)]
pub struct FileHeader {
    pub mesh_count: u64,
    pub vertex_count: u64,
    pub triangle_count: u64,
    pub transform_count: u64,
    pub transform_relation_count: u64,
    pub instance_count: u64,
    pub material_count: u64,
    pub texture_count: u64,
    pub string_byte_count: u64,
}

type Triangle = [u32; 3];

#[derive(Debug)]
pub struct SceneFile {
    pub mesh_descriptions: Vec<MeshDescription>,
    pub pos_in_obj_buffer: Vec<[FiniteF32; 3]>,
    pub nor_in_obj_buffer: Vec<[FiniteF32; 3]>,
    pub pos_in_tex_buffer: Vec<[FiniteF32; 2]>,
    pub triangle_buffer: Vec<Triangle>,
    pub transforms: Vec<Transform>,
    pub transform_relations: Vec<TransformRelation>,
    pub instances: Vec<Instance>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
}

unsafe fn write_vec<T, W: std::io::Write>(vec: &Vec<T>, writer: &mut W) -> std::io::Result<usize> {
    let byte_count = std::mem::size_of_val(&vec[..]);
    writer.write_all(std::slice::from_raw_parts(vec.as_ptr() as *const u8, byte_count))?;
    Ok(byte_count)
}

unsafe fn read_vec<T, R: std::io::Read>(count: usize, reader: &mut R) -> std::io::Result<Vec<T>> {
    let mut vec = Vec::<T>::with_capacity(count);
    vec.set_len(count);
    reader.read_exact(std::slice::from_raw_parts_mut(
        vec.as_mut_ptr() as *mut u8,
        std::mem::size_of_val(&vec[..]),
    ))?;
    Ok(vec)
}

impl SceneFile {
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let vertex_count = self.pos_in_obj_buffer.len();

        assert_eq!(vertex_count, self.pos_in_obj_buffer.len());
        assert_eq!(vertex_count, self.nor_in_obj_buffer.len());
        assert_eq!(vertex_count, self.pos_in_tex_buffer.len());

        let mut string_bytes = Vec::new();

        let materials: Vec<RawMaterial> = self.materials.iter().map(|material| {
            RawMaterial {

            }
        }).collect();

        let textures: Vec<RawTextures> = self.textures.iter().map(|texture| {

        }).collect();

        let header = FileHeader {
            mesh_count: self.mesh_descriptions.len() as u64,
            vertex_count: vertex_count as u64,
            triangle_count: self.triangle_buffer.len() as u64,
            transform_count: self.transforms.len() as u64,
            transform_relation_count: self.transform_relations.len() as u64,
            instance_count: self.instances.len() as u64,
            material_count: self.materials.len() as u64,
            texture_count: self.textures.len() as u64,
        };

        unsafe {
            writer.write_all(std::slice::from_raw_parts(
                &header as *const _ as *const u8,
                std::mem::size_of::<FileHeader>(),
            ))?;
            write_vec(&self.mesh_descriptions, writer)?;
            write_vec(&self.pos_in_obj_buffer, writer)?;
            write_vec(&self.nor_in_obj_buffer, writer)?;
            write_vec(&self.pos_in_tex_buffer, writer)?;
            write_vec(&self.triangle_buffer, writer)?;
            write_vec(&self.transforms, writer)?;
            write_vec(&self.transform_relations, writer)?;
            write_vec(&self.instances, writer)?;
        }

        Ok(())
    }

    pub fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        unsafe {
            let mut header = std::mem::MaybeUninit::<FileHeader>::uninit();

            reader.read_exact(std::slice::from_raw_parts_mut(
                header.as_mut_ptr() as *mut u8,
                std::mem::size_of::<FileHeader>(),
            ))?;

            let header = header.assume_init();

            let mesh_descriptions = read_vec::<MeshDescription, _>(header.mesh_count as usize, reader)?;
            let pos_in_obj_buffer = read_vec::<[FiniteF32; 3], _>(header.vertex_count as usize, reader)?;
            let nor_in_obj_buffer = read_vec::<[FiniteF32; 3], _>(header.vertex_count as usize, reader)?;
            let pos_in_tex_buffer = read_vec::<[FiniteF32; 2], _>(header.vertex_count as usize, reader)?;
            let triangle_buffer = read_vec::<Triangle, _>(header.triangle_count as usize, reader)?;
            let transforms = read_vec::<Transform, _>(header.transform_count as usize, reader)?;
            let transform_relations =
                read_vec::<TransformRelation, _>(header.transform_relation_count as usize, reader)?;
            let instances = read_vec::<Instance, _>(header.instance_count as usize, reader)?;

            Ok(SceneFile {
                mesh_descriptions,
                pos_in_obj_buffer,
                nor_in_obj_buffer,
                pos_in_tex_buffer,
                triangle_buffer,
                transforms,
                transform_relations,
                instances,
            })
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct FiniteF32(f32);

impl FiniteF32 {
    pub fn new(val: f32) -> Option<Self> {
        if val.is_finite() {
            Some(Self(val))
        } else {
            None
        }
    }
}

impl std::hash::Hash for FiniteF32 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}

impl std::cmp::PartialEq for FiniteF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl std::cmp::Eq for FiniteF32 {}

// #[repr(C)]
// pub struct OpaqueDepthVertex {
//     pub pos_in_obj: [f32; 3],
// }

// #[repr(C)]
// pub struct MaskedDepthVertex {
//     pub pos_in_obj: [f32; 3],
//     pub pos_in_tex: [f32; 2],
// }

// #[repr(C)]
// pub struct FullVertex {
//     pub pos_in_obj: [f32; 3],
//     pub nor_in_obj: [f32; 3],
//     pub pos_in_tex: [f32; 2],
// }
