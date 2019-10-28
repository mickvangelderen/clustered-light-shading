use crate::light::*;
use crate::*;
use cgmath::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

pub type TextureIndex = u32;
pub type MaterialIndex = u32;

pub struct Texture {
    pub name: gl::TextureName,
    pub dimensions: [f32; 2],
}

pub struct MeshMeta {
    pub element_count: u32,
    pub element_offset: usize,
    pub vertex_base: u32,
    pub vertex_count: u32,
}

#[repr(C)]
pub struct Vertex {
    pub pos_in_obj: [f32; 3],
    pub pos_in_tex: [f32; 2],
    pub nor_in_obj: [f32; 3],
    pub tan_in_obj: [f32; 3],
}

macro_rules! set_attrib {
    ($gl: ident, $vao: ident, $loc: expr, $component_count: expr, $component_type: expr, $Vertex: ident, $field: ident, $bbi: expr) => {
        $gl.vertex_array_attrib_format(
            $vao,
            $loc,
            $component_count,
            $component_type,
            false,
            field_offset!($Vertex, $field) as u32,
        );
        $gl.enable_vertex_array_attrib($vao, $loc);
        $gl.vertex_array_attrib_binding($vao, $loc, $bbi);
    };
}

impl Vertex {
    fn set_format(gl: &gl::Gl, vao: gl::VertexArrayName, vb: gl::BufferName, eb: gl::BufferName) {
        unsafe {
            set_attrib!(
                gl,
                vao,
                rendering::VS_POS_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                Vertex,
                pos_in_obj,
                BBI_00
            );
            set_attrib!(
                gl,
                vao,
                rendering::VS_POS_IN_TEX_LOC,
                2,
                gl::FLOAT,
                Vertex,
                pos_in_tex,
                BBI_00
            );
            set_attrib!(
                gl,
                vao,
                rendering::VS_NOR_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                Vertex,
                nor_in_obj,
                BBI_00
            );
            set_attrib!(
                gl,
                vao,
                rendering::VS_TAN_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                Vertex,
                tan_in_obj,
                BBI_00
            );

            // Bind buffers to vao.
            let stride = std::mem::size_of::<Vertex>() as u32;
            gl.vertex_array_vertex_buffer(vao, BBI_00, vb, 0, stride);
            gl.vertex_array_element_buffer(vao, eb);
        }
    }

    fn set_pos_format(gl: &gl::Gl, vao: gl::VertexArrayName, vb: gl::BufferName, eb: gl::BufferName) {
        unsafe {
            set_attrib!(
                gl,
                vao,
                rendering::VS_POS_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                Vertex,
                pos_in_obj,
                BBI_00
            );

            // Bind buffers to vao.
            let stride = std::mem::size_of::<Vertex>() as u32;
            gl.vertex_array_vertex_buffer(vao, BBI_00, vb, 0, stride);
            gl.vertex_array_element_buffer(vao, eb);
        }
    }
}

pub struct RawMesh<V, I> {
    pub vertices: V,
    pub indices: I,
}

pub type Triangle<T> = [T; 3];

fn generate_cube_mesh(x: (f32, f32), y: (f32, f32), z: (f32, f32)) -> RawMesh<[Vertex; 4 * 6], [Triangle<u32>; 2 * 6]> {
    let (x0, x1) = x;
    let (y0, y1) = y;
    let (z0, z1) = z;
    let (s0, s1) = (0.0, 1.0);
    let (t0, t1) = (0.0, 1.0);
    let nx = [-1.0, 0.0, 0.0];
    let px = [1.0, 0.0, 0.0];
    let ny = [0.0, -1.0, 0.0];
    let py = [0.0, 1.0, 0.0];
    let nz = [0.0, 0.0, -1.0];
    let pz = [0.0, 0.0, 1.0];
    RawMesh {
        vertices: [
            // -X
            Vertex {
                pos_in_obj: [x0, y0, z0],
                pos_in_tex: [s0, t0],
                nor_in_obj: nx,
                tan_in_obj: pz,
            },
            Vertex {
                pos_in_obj: [x0, y0, z1],
                pos_in_tex: [s0, t1],
                nor_in_obj: nx,
                tan_in_obj: pz,
            },
            Vertex {
                pos_in_obj: [x0, y1, z1],
                pos_in_tex: [s1, t1],
                nor_in_obj: nx,
                tan_in_obj: pz,
            },
            Vertex {
                pos_in_obj: [x0, y1, z0],
                pos_in_tex: [s1, t0],
                nor_in_obj: nx,
                tan_in_obj: pz,
            },
            // -Y
            Vertex {
                pos_in_obj: [x0, y0, z0],
                pos_in_tex: [s0, t0],
                nor_in_obj: ny,
                tan_in_obj: px,
            },
            Vertex {
                pos_in_obj: [x1, y0, z0],
                pos_in_tex: [s0, t1],
                nor_in_obj: ny,
                tan_in_obj: px,
            },
            Vertex {
                pos_in_obj: [x1, y0, z1],
                pos_in_tex: [s1, t1],
                nor_in_obj: ny,
                tan_in_obj: px,
            },
            Vertex {
                pos_in_obj: [x0, y0, z1],
                pos_in_tex: [s1, t0],
                nor_in_obj: ny,
                tan_in_obj: px,
            },
            // -Z
            Vertex {
                pos_in_obj: [x0, y0, z0],
                pos_in_tex: [s0, t0],
                nor_in_obj: nz,
                tan_in_obj: py,
            },
            Vertex {
                pos_in_obj: [x0, y1, z0],
                pos_in_tex: [s0, t1],
                nor_in_obj: nz,
                tan_in_obj: py,
            },
            Vertex {
                pos_in_obj: [x1, y1, z0],
                pos_in_tex: [s1, t1],
                nor_in_obj: nz,
                tan_in_obj: py,
            },
            Vertex {
                pos_in_obj: [x1, y0, z0],
                pos_in_tex: [s1, t0],
                nor_in_obj: nz,
                tan_in_obj: py,
            },
            // +X
            Vertex {
                pos_in_obj: [x1, y1, z1],
                pos_in_tex: [s0, t0],
                nor_in_obj: px,
                tan_in_obj: ny,
            },
            Vertex {
                pos_in_obj: [x1, y0, z1],
                pos_in_tex: [s0, t1],
                nor_in_obj: px,
                tan_in_obj: ny,
            },
            Vertex {
                pos_in_obj: [x1, y0, z0],
                pos_in_tex: [s1, t1],
                nor_in_obj: px,
                tan_in_obj: ny,
            },
            Vertex {
                pos_in_obj: [x1, y1, z0],
                pos_in_tex: [s1, t0],
                nor_in_obj: px,
                tan_in_obj: ny,
            },
            // +Y
            Vertex {
                pos_in_obj: [x1, y1, z1],
                pos_in_tex: [s0, t0],
                nor_in_obj: py,
                tan_in_obj: nx,
            },
            Vertex {
                pos_in_obj: [x1, y1, z0],
                pos_in_tex: [s0, t1],
                nor_in_obj: py,
                tan_in_obj: nx,
            },
            Vertex {
                pos_in_obj: [x0, y1, z0],
                pos_in_tex: [s1, t1],
                nor_in_obj: py,
                tan_in_obj: nx,
            },
            Vertex {
                pos_in_obj: [x0, y1, z1],
                pos_in_tex: [s1, t0],
                nor_in_obj: py,
                tan_in_obj: nx,
            },
            // +Z
            Vertex {
                pos_in_obj: [x1, y1, z1],
                pos_in_tex: [s0, t0],
                nor_in_obj: pz,
                tan_in_obj: nx,
            },
            Vertex {
                pos_in_obj: [x0, y1, z1],
                pos_in_tex: [s0, t1],
                nor_in_obj: pz,
                tan_in_obj: nx,
            },
            Vertex {
                pos_in_obj: [x0, y0, z1],
                pos_in_tex: [s1, t1],
                nor_in_obj: pz,
                tan_in_obj: nx,
            },
            Vertex {
                pos_in_obj: [x1, y0, z1],
                pos_in_tex: [s1, t0],
                nor_in_obj: pz,
                tan_in_obj: nx,
            },
        ],
        indices: [
            [0, 1, 2],
            [2, 3, 0],
            [4, 5, 6],
            [6, 7, 4],
            [8, 9, 10],
            [10, 11, 8],
            [12, 13, 14],
            [14, 15, 12],
            [16, 17, 18],
            [18, 19, 16],
            [20, 21, 22],
            [22, 23, 20],
        ],
    }
}

pub static FULL_SCREEN_VERTICES: [[f32; 2]; 3] = [[0.0, 0.0], [2.0, 0.0], [0.0, 2.0]];
pub static FULL_SCREEN_INDICES: [[u32; 3]; 1] = [[0, 1, 2]];

#[allow(unused)]
pub struct Resources {
    // pub meshes: Vec<Mesh>,
    // pub materials: Vec<Material>,
    // pub textures: Vec<Texture>,
    // pub path_to_texture_index: HashMap<PathBuf, TextureIndex>,
    // pub color_to_texture_index: HashMap<[u8; 4], TextureIndex>,
    pub scene_vao: gl::VertexArrayName,
    pub scene_vb: gl::BufferName,
    pub scene_eb: gl::BufferName,

    pub scene_file: renderer::scene_file::SceneFile,

    pub point_lights: Vec<PointLight>,

    pub full_screen_vao: gl::VertexArrayName,
    pub full_screen_vb: gl::BufferName,
    pub full_screen_eb: gl::BufferName,

    pub cluster_vao: gl::VertexArrayName,
    pub cluster_vb: gl::BufferName,
    pub cluster_eb: gl::BufferName,
    pub cluster_element_count: u32,
}

#[inline]
fn clamp_f32(input: f32, min: f32, max: f32) -> f32 {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

#[inline]
fn rgba_f32_to_rgba_u8(input: [f32; 4]) -> [u8; 4] {
    let mut output = [0; 4];
    for i in 0..4 {
        output[i] = clamp_f32(input[i] * 256.0, 0.0, 255.0) as u8
    }
    output
}

// impl Resources {
//     fn load_or_create_texture(&mut self, gl: &gl::Gl, path: impl AsRef<Path>, srgb: bool) {
//         self.path_to_texture_index
//             .entry(file_path.clone())
//             .or_insert_with(|| match file_path.extension().unwrap() {
//                 x if x == "dds" => {
//                     let file = std::fs::File::open(Path::new(&file_path)).unwrap();
//                     let mut reader = std::io::BufReader::new(file);
//                     let dds = dds::DDS::decode(&mut reader).unwrap();

//                     let internal_format = match dds.header.compression {
//                         dds::Compression::DXT1 => {

//                         },
//                         dds::Compression::DXT3 => {

//                         }
//                         dds::Compression::DXT5 => {

//                         }
//                         other => {
//                             panic!("Compression {:?} not supported.", other);
//                         }
//                     };

//                     unsafe {
//                         let name = gl.create_texture(gl::TEXTURE_2D);

//                         gl.bind_texture(gl::TEXTURE_2D, name);

//                         for level in
//                         gl.compressed_tex_image_2d(
//                             gl::TEXTURE_2D,
//                             0,
//                             match srgb {
//                                 true => gl::SRGB8_ALPHA8,
//                                 false => gl::RGBA8,
//                             },
//                             img.width() as i32,
//                             img.height() as i32,
//                             gl::RGBA,
//                             gl::UNSIGNED_BYTE,
//                             img.as_ptr() as *const std::os::raw::c_void,
//                         );
//                         gl.generate_texture_mipmap(name);
//                         gl.texture_parameteri(name, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
//                         gl.texture_parameteri(name, gl::TEXTURE_MAG_FILTER, gl::LINEAR);

//                         Texture {
//                             name,
//                             dimensions: [img.width() as f32, img.height() as f32],
//                         }
//                     }
//                     let components =
//                     println!("{:?}", &dds.header);
//                 }
//                 _ => {
//                     let img = image::open(&file_path)
//                         .expect("Failed to load image.")
//                         .flipv()
//                         .to_rgba();
//                     total_bytes += std::mem::size_of_val(&*img);

//                     let texture = create_texture(gl, img, srgb);

//                     let index = self.textures.len() as TextureIndex;
//                     self.textures.push(texture);
//                     index
//                 }
//             })
//     }
// }

#[inline]
fn create_texture(gl: &gl::Gl, img: image::RgbaImage, srgb: bool) -> Texture {
    unsafe {
        let name = gl.create_texture(gl::TEXTURE_2D);

        gl.bind_texture(gl::TEXTURE_2D, name);
        gl.tex_image_2d(
            gl::TEXTURE_2D,
            0,
            match srgb {
                true => gl::InternalFormat::from(gl::SRGB8_ALPHA8),
                false => gl::InternalFormat::from(gl::RGBA8),
            },
            img.width() as i32,
            img.height() as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            img.as_ptr() as *const std::os::raw::c_void,
        );
        gl.generate_texture_mipmap(name);
        gl.texture_parameteri(name, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
        gl.texture_parameteri(name, gl::TEXTURE_MAG_FILTER, gl::LINEAR);

        Texture {
            name,
            dimensions: [img.width() as f32, img.height() as f32],
        }
    }
}

pub struct Mesh {
    pub name: String,
    /// Triangle indices.
    pub triangles: Vec<[u32; 3]>,

    /// Position in object space.
    pub pos_in_obj: Vec<[f32; 3]>,

    /// Position in texture space.
    pub pos_in_tex: Vec<[f32; 2]>,

    /// Normal in object space.
    pub nor_in_obj: Vec<[f32; 3]>,

    /// Tangent in object space.
    pub tan_in_obj: Vec<[f32; 3]>,

    /// Derpiederp.
    pub translate: Vector3<f32>,

    /// Material id.
    pub material_index: Option<MaterialIndex>,
}

pub struct Material {
    pub diffuse: TextureIndex,
    pub normal: TextureIndex,
    pub specular: TextureIndex,
    pub shininess: f32,
}

const BBI_00: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(0);
const BBI_01: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(1);
const BBI_02: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(2);

impl Resources {
    pub fn new<P: AsRef<Path>>(gl: &gl::Gl, resource_dir: P, configuration: &Configuration) -> Self {
        let resource_dir = resource_dir.as_ref();

        let scene_file = {
            let mut file = std::fs::File::open(&resource_dir.join(&configuration.global.scene_path)).unwrap();
            renderer::scene_file::SceneFile::read(&mut file).unwrap()
        };

        let (scene_vao, scene_vb, scene_eb) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            fn align_16(n: usize) -> usize {
                ((n + 15) / 16) * 16
            }

            let pos_in_obj_bytes = scene_file.pos_in_obj_buffer.vec_as_bytes();
            let pos_in_obj_byte_length = pos_in_obj_bytes.len();
            let pos_in_obj_byte_offset = 0;

            let nor_in_obj_bytes = scene_file.nor_in_obj_buffer.vec_as_bytes();
            let nor_in_obj_byte_length = nor_in_obj_bytes.len();
            let nor_in_obj_byte_offset = align_16(pos_in_obj_byte_offset + pos_in_obj_byte_length);

            let pos_in_tex_bytes = scene_file.pos_in_tex_buffer.vec_as_bytes();
            let pos_in_tex_byte_length = pos_in_tex_bytes.len();
            let pos_in_tex_byte_offset = align_16(nor_in_obj_byte_offset + nor_in_obj_byte_length);

            let total_byte_length = align_16(pos_in_tex_byte_offset + pos_in_tex_byte_length);

            // Upload data.
            gl.named_buffer_reserve(vb, total_byte_length, gl::STATIC_DRAW);
            gl.named_buffer_sub_data(vb, pos_in_obj_byte_offset, pos_in_obj_bytes);
            gl.named_buffer_sub_data(vb, nor_in_obj_byte_offset, nor_in_obj_bytes);
            gl.named_buffer_sub_data(vb, pos_in_tex_byte_offset, pos_in_tex_bytes);
            gl.named_buffer_data(eb, scene_file.triangle_buffer.vec_as_bytes(), gl::STATIC_DRAW);

            // Attribute layout specification.
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_OBJ_LOC, 3, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_format(vao, rendering::VS_NOR_IN_OBJ_LOC, 3, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_TEX_LOC, 2, gl::FLOAT, false, 0);

            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_OBJ_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_NOR_IN_OBJ_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_TEX_LOC);

            // Attribute source specification.
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_OBJ_LOC, BBI_00);
            gl.vertex_array_attrib_binding(vao, rendering::VS_NOR_IN_OBJ_LOC, BBI_01);
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_TEX_LOC, BBI_02);

            gl.vertex_array_vertex_buffer(
                vao,
                BBI_00,
                vb,
                pos_in_obj_byte_offset,
                std::mem::size_of::<[f32; 3]>() as u32,
            );
            gl.vertex_array_vertex_buffer(
                vao,
                BBI_01,
                vb,
                nor_in_obj_byte_offset,
                std::mem::size_of::<[f32; 3]>() as u32,
            );
            gl.vertex_array_vertex_buffer(
                vao,
                BBI_02,
                vb,
                pos_in_tex_byte_offset,
                std::mem::size_of::<[f32; 2]>() as u32,
            );

            // Element buffer.
            gl.vertex_array_element_buffer(vao, eb);

            (vao, vb, eb)
        };

        let (full_screen_vao, full_screen_vb, full_screen_eb) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            // Set up attributes.
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_TEX_LOC, 2, gl::FLOAT, false, 0);
            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_TEX_LOC);
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_TEX_LOC, BBI_00);

            // Bind buffers to vao.
            let stride = std::mem::size_of::<[f32; 2]>() as u32;
            gl.vertex_array_vertex_buffer(vao, BBI_00, vb, 0, stride);
            gl.vertex_array_element_buffer(vao, eb);

            // Upload data.
            gl.named_buffer_data(vb, FULL_SCREEN_VERTICES.slice_as_bytes(), gl::STATIC_DRAW);
            gl.named_buffer_data(eb, FULL_SCREEN_INDICES.slice_as_bytes(), gl::STATIC_DRAW);

            (vao, vb, eb)
        };

        let (cluster_vao, cluster_vb, cluster_eb, cluster_element_count) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            Vertex::set_format(gl, vao, vb, eb);

            // Upload data.
            let r = (0.00001, 0.99999);
            let mesh = generate_cube_mesh(r, r, r);
            gl.named_buffer_data(vb, mesh.vertices.value_as_bytes(), gl::STATIC_DRAW);
            gl.named_buffer_data(eb, mesh.indices.value_as_bytes(), gl::STATIC_DRAW);

            (vao, vb, eb, (mesh.indices.len() * 3) as u32)
        };

        Resources {
            scene_vao,
            scene_vb,
            scene_eb,
            scene_file,
            full_screen_vao,
            full_screen_vb,
            full_screen_eb,
            cluster_vao,
            cluster_vb,
            cluster_eb,
            cluster_element_count,
            point_lights: vec![
                PointLight {
                    ambient: RGB::new(0.2000, 0.2000, 0.2000),
                    diffuse: RGB::new(4.0000, 4.0000, 4.0000),
                    specular: RGB::new(1.0000, 1.0000, 1.0000),
                    pos_in_wld: Point3::new(-12.9671, 1.8846, -4.4980),
                    attenuation: AttenParams {
                        intensity: 2.0,
                        clip_near: 0.5,
                        cutoff: 0.02,
                    }
                    .into(),
                },
                PointLight {
                    ambient: RGB::new(0.2000, 0.2000, 0.2000),
                    diffuse: RGB::new(4.0000, 4.0000, 4.0000),
                    specular: RGB::new(1.0000, 1.0000, 1.0000),
                    pos_in_wld: Point3::new(-11.9563, 2.6292, 3.8412),
                    attenuation: AttenParams {
                        intensity: 3.0,
                        clip_near: 0.5,
                        cutoff: 0.02,
                    }
                    .into(),
                },
                PointLight {
                    ambient: RGB::new(0.2000, 0.2000, 0.2000),
                    diffuse: RGB::new(4.0000, 4.0000, 4.0000),
                    specular: RGB::new(1.0000, 1.0000, 1.0000),
                    pos_in_wld: Point3::new(13.6090, 2.6292, 3.3216),
                    attenuation: AttenParams {
                        intensity: 2.0,
                        clip_near: 0.5,
                        cutoff: 0.02,
                    }
                    .into(),
                },
                PointLight {
                    ambient: RGB::new(0.2000, 0.2000, 0.2000),
                    diffuse: RGB::new(4.0000, 4.0000, 4.0000),
                    specular: RGB::new(1.0000, 1.0000, 1.0000),
                    pos_in_wld: Point3::new(12.5982, 1.8846, -5.0176),
                    attenuation: AttenParams {
                        intensity: 1.0,
                        clip_near: 0.5,
                        cutoff: 0.02,
                    }
                    .into(),
                },
                PointLight {
                    ambient: RGB::new(0.2000, 0.2000, 0.2000),
                    diffuse: RGB::new(4.0000, 4.0000, 4.0000),
                    specular: RGB::new(1.0000, 1.0000, 1.0000),
                    pos_in_wld: Point3::new(3.3116, 4.3440, 5.1447),
                    attenuation: AttenParams {
                        intensity: 1.2,
                        clip_near: 0.5,
                        cutoff: 0.02,
                    }
                    .into(),
                },
                PointLight {
                    ambient: RGB::new(0.2000, 0.2000, 0.2000),
                    diffuse: RGB::new(0.0367, 4.0000, 0.0000),
                    specular: RGB::new(0.0092, 1.0000, 0.0000),
                    pos_in_wld: Point3::new(8.8820, 6.7391, -1.0279),
                    attenuation: AttenParams {
                        intensity: 0.5,
                        clip_near: 0.5,
                        cutoff: 0.02,
                    }
                    .into(),
                },
                PointLight {
                    ambient: RGB::new(0.2000, 0.2000, 0.2000),
                    diffuse: RGB::new(4.0000, 0.1460, 0.1006),
                    specular: RGB::new(1.0000, 0.0365, 0.0251),
                    pos_in_wld: Point3::new(-4.6988, 10.0393, 0.8667),
                    attenuation: AttenParams {
                        intensity: 1.0,
                        clip_near: 0.5,
                        cutoff: 0.02,
                    }
                    .into(),
                },
                PointLight {
                    ambient: RGB::new(0.2000, 0.2000, 0.2000),
                    diffuse: RGB::new(0.8952, 0.8517, 4.0000),
                    specular: RGB::new(0.2238, 0.2129, 1.0000),
                    pos_in_wld: Point3::new(-4.6816, 1.0259, -2.1767),
                    attenuation: AttenParams {
                        intensity: 3.0,
                        clip_near: 0.5,
                        cutoff: 0.02,
                    }
                    .into(),
                },
            ],
        }
    }
}
