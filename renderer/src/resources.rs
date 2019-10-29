use crate::light::*;
use crate::*;
use cgmath::*;
use std::collections::HashMap;
// use std::convert::TryFrom;
// use std::convert::TryInto;
use std::path::Path;
// use std::path::PathBuf;
// use std::time::Instant;

pub static FULL_SCREEN_VERTICES: [[f32; 2]; 3] = [[0.0, 0.0], [2.0, 0.0], [0.0, 2.0]];
pub static FULL_SCREEN_INDICES: [[u32; 3]; 1] = [[0, 1, 2]];

pub struct Material {
    normal_texture_index: usize,
    ambient_texture_index: usize,
    diffuse_texture_index: usize,
    specular_texture_index: usize,
    shininess: f64,
}

pub struct Texture {
    name: gl::TextureName,
}

#[allow(unused)]
pub struct Resources {
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,

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

// #[inline]
// fn clamp_f32(input: f32, min: f32, max: f32) -> f32 {
//     if input < min {
//         min
//     } else if input > max {
//         max
//     } else {
//         input
//     }
// }

// #[inline]
// fn rgba_f32_to_rgba_u8(input: [f32; 4]) -> [u8; 4] {
//     let mut output = [0; 4];
//     for i in 0..4 {
//         output[i] = clamp_f32(input[i] * 256.0, 0.0, 255.0) as u8
//     }
//     output
// }

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

// #[inline]
// fn create_texture(gl: &gl::Gl, img: image::RgbaImage, srgb: bool) -> Texture {
//     unsafe {
//         let name = gl.create_texture(gl::TEXTURE_2D);

//         gl.bind_texture(gl::TEXTURE_2D, name);
//         gl.tex_image_2d(
//             gl::TEXTURE_2D,
//             0,
//             match srgb {
//                 true => gl::InternalFormat::from(gl::SRGB8_ALPHA8),
//                 false => gl::InternalFormat::from(gl::RGBA8),
//             },
//             img.width() as i32,
//             img.height() as i32,
//             gl::RGBA,
//             gl::UNSIGNED_BYTE,
//             img.as_ptr() as *const std::os::raw::c_void,
//         );
//         gl.generate_texture_mipmap(name);
//         gl.texture_parameteri(name, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
//         gl.texture_parameteri(name, gl::TEXTURE_MAG_FILTER, gl::LINEAR);

//         Texture {
//             name,
//             dimensions: [img.width() as f32, img.height() as f32],
//         }
//     }
// }

// pub struct Mesh {
//     pub name: String,
//     /// Triangle indices.
//     pub triangles: Vec<[u32; 3]>,

//     /// Position in object space.
//     pub pos_in_obj: Vec<[f32; 3]>,

//     /// Position in texture space.
//     pub pos_in_tex: Vec<[f32; 2]>,

//     /// Normal in object space.
//     pub nor_in_obj: Vec<[f32; 3]>,

//     /// Tangent in object space.
//     pub tan_in_obj: Vec<[f32; 3]>,

//     /// Derpiederp.
//     pub translate: Vector3<f32>,

//     /// Material id.
//     pub material_index: Option<MaterialIndex>,
// }

pub const BBI_00: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(0);
pub const BBI_01: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(1);
pub const BBI_02: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(2);

fn load_texture(gl: &gl::Gl, file_path: impl AsRef<Path>) -> io::Result<gl::TextureName> {
    let file = std::fs::File::open(file_path).unwrap();
    let mut reader = std::io::BufReader::new(file);
    let dds = dds::File::parse(&mut reader).unwrap();

    unsafe {
        let name = gl.create_texture(gl::TEXTURE_2D);
        // NOTE(mickvangelderen): No dsa for compressed textures??
        gl.bind_texture(gl::TEXTURE_2D, name);
        for (layer_index, layer) in dds.layers.iter().enumerate() {
            gl.compressed_tex_image_2d(
                gl::TEXTURE_2D,
                layer_index as i32,
                dds.header.pixel_format.to_gl_internal_format(),
                layer.width as i32,
                layer.height as i32,
                &dds.bytes[layer.byte_offset..(layer.byte_offset + layer.byte_count)],
            );
        }

        Ok(name)
    }
}

impl Resources {
    pub fn new<P: AsRef<Path>>(gl: &gl::Gl, resource_dir: P, configuration: &Configuration) -> Self {
        let resource_dir = resource_dir.as_ref();

        let scene_file = {
            let mut file = std::fs::File::open(&resource_dir.join(&configuration.global.scene_path)).unwrap();
            renderer::scene_file::SceneFile::read(&mut file).unwrap()
        };

        let (textures, materials) = {
            let mut textures = Vec::new();
            let mut materials = Vec::new();

            let mut file_texture_index_to_texture_index: HashMap<usize, usize> = HashMap::new();
            let mut color_to_texture_index: HashMap<[u8; 4], usize> = HashMap::new();

            let mut color_texture_index = |color: [u8; 4]| {
                color_to_texture_index.get(&color).unwrap_or_else(|| {
                    let texture = Texture {
                        name: gl::create_texture()
                    };
                    textures.push
                    color_to_texture_index.insert(color)
                })
            };

            let materials: Vec<Material> = scene_file.materials.iter().map(|material| {
                let normal_texture_index = match material.normal_texture_index {

                }
                Material {
                    normal_texture_index: m
                    ambient_texture_index:
                    diffuse_texture_index:
                    specular_texture_index:
                    shininess: material.shininess,
                }
            }).collect();
        }

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

        let (cluster_vao, cluster_vb, cluster_eb, cluster_element_count) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            cube_mesh::Vertex::set_format(gl, vao, vb, eb);

            // Upload data.
            let r = (0.00001, 0.99999);
            let (vertices, indices) = cube_mesh::generate(r, r, r);
            gl.named_buffer_data(vb, vertices.value_as_bytes(), gl::STATIC_DRAW);
            gl.named_buffer_data(eb, indices.value_as_bytes(), gl::STATIC_DRAW);

            (vao, vb, eb, (indices.len() * 3) as u32)
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
