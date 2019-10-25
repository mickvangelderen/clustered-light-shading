#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct Vertex {
    pub pos_in_obj: [FiniteF32; 3],
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
pub struct FileHeader {
    pub mesh_count: u64,
    pub vertex_count: u64,
    pub triangle_count: u64,
}

type Triangle = [u32; 3];

#[derive(Debug)]
pub struct SceneFile {
    pub mesh_descriptions: Vec<MeshDescription>,
    pub vertex_buffer: Vec<Vertex>,
    pub triangle_buffer: Vec<Triangle>,
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
        let header = FileHeader {
            mesh_count: self.mesh_descriptions.len() as u64,
            vertex_count: self.vertex_buffer.len() as u64,
            triangle_count: self.triangle_buffer.len() as u64,
        };

        unsafe {
            writer.write_all(std::slice::from_raw_parts(
                &header as *const _ as *const u8,
                std::mem::size_of::<FileHeader>(),
            ))?;
            write_vec(&self.mesh_descriptions, writer)?;
            write_vec(&self.vertex_buffer, writer)?;
            write_vec(&self.triangle_buffer, writer)?;
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
            let vertex_buffer = read_vec::<Vertex, _>(header.vertex_count as usize, reader)?;
            let triangle_buffer = read_vec::<Triangle, _>(header.triangle_count as usize, reader)?;

            Ok(SceneFile {
                mesh_descriptions,
                vertex_buffer,
                triangle_buffer,
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
