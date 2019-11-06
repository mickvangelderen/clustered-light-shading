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
    pub normal_texture_index: usize,
    pub emissive_texture_index: usize,
    pub ambient_texture_index: usize,
    pub diffuse_texture_index: usize,
    pub specular_texture_index: usize,
    pub shininess: f32,
}

pub struct Texture {
    pub name: gl::TextureName,
    pub has_alpha: bool,
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

fn f32_to_unorm(val: f32) -> u8 {
    assert!(val.is_finite());
    let val = val * 255.0;
    if val < 0.0 {
        0
    } else if val > 255.0 {
        255
    } else {
        val as u8
    }
}

fn f32_3_to_unorm(a: [f32; 3]) -> [u8; 3] {
    [f32_to_unorm(a[0]), f32_to_unorm(a[1]), f32_to_unorm(a[2])]
}

#[inline]
fn create_1x1_rgb_texture(gl: &gl::Gl, rgb: [u8; 3]) -> Texture {
    unsafe {
        let name = gl.create_texture(gl::TEXTURE_2D);

        let width = 1;
        let height = 1;
        let xoffset = 0;
        let yoffset = 0;

        gl.texture_storage_2d(name, 1, gl::RGB8, width, height);

        gl.texture_sub_image_2d(
            name,
            0,
            xoffset,
            yoffset,
            width,
            height,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            rgb.as_ptr() as *const _,
        );

        gl.texture_parameteri(name, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
        gl.texture_parameteri(name, gl::TEXTURE_MAG_FILTER, gl::NEAREST);

        Texture { name, has_alpha: false }
    }
}

pub const BBI_00: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(0);
pub const BBI_01: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(1);
pub const BBI_02: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(2);
pub const BBI_03: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(3);
pub const BBI_04: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(4);

fn load_dds_texture(gl: &gl::Gl, file_path: impl AsRef<Path>) -> io::Result<Texture> {
    let file = std::fs::File::open(file_path).unwrap();
    let mut reader = std::io::BufReader::new(file);
    let dds = dds::File::parse(&mut reader).unwrap();

    unsafe {
        let name = gl.create_texture(gl::TEXTURE_2D);
        // NOTE(mickvangelderen): No dsa for compressed textures??
        gl.bind_texture(gl::TEXTURE_2D, name);
        assert!(dds.layers.len() > 0);
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

        let has_alpha = match dds.header.pixel_format {
            dds::Format::BC1_UNORM_RGBA | dds::Format::BC2_UNORM_RGBA | dds::Format::BC3_UNORM_RGBA => true,
            _ => false,
        };

        Ok(Texture { name, has_alpha })
    }
}

impl Resources {
    pub fn new<P: AsRef<Path>>(gl: &gl::Gl, resource_dir: P, configuration: &Configuration) -> Self {
        let resource_dir = resource_dir.as_ref();

        let scene_file_path = std::fs::canonicalize(resource_dir.join(&configuration.global.scene_path)).unwrap();
        let scene_dir = scene_file_path.parent().unwrap();

        let scene_file = {
            let mut file = std::fs::File::open(&scene_file_path).unwrap();
            renderer::scene_file::SceneFile::read(&mut file).unwrap()
        };

        // TODO(mickvangelderen): Use these for culling?
        // let bounding_boxes: Vec<Range3<f32>> = scene_file.mesh_descriptions.iter().map(|mesh_description| {
        //     let vertex_offset = mesh_description.vertex_offset as usize;
        //     let vertex_count = mesh_description.vertex_count as usize;
        //     let vertex_iter = scene_file.pos_in_obj_buffer[vertex_offset..(vertex_offset + vertex_count)].iter().map(|&pos_in_obj| {
        //         Point3::new(pos_in_obj[0].get(), pos_in_obj[1].get(), pos_in_obj[2].get())
        //     });
        //     Range3::from_points(vertex_iter).unwrap()
        // }).collect();

        // let bounding_spheres: Vec<(Point3<f32>, f32)> = scene_file.mesh_descriptions.iter().enumerate().map(|(mesh_index, mesh_description)| {
        //     let center = bounding_boxes[mesh_index].center();
        //     let vertex_offset = mesh_description.vertex_offset as usize;
        //     let vertex_count = mesh_description.vertex_count as usize;
        //     let mut max_squared_distance = 0.0;
        //     for &pos_in_obj in scene_file.pos_in_obj_buffer[vertex_offset..(vertex_offset + vertex_count)].iter() {
        //         let pos_in_obj = Point3::new(pos_in_obj[0].get(), pos_in_obj[1].get(), pos_in_obj[2].get());
        //         let squared_distance = (pos_in_obj - center).magnitude2();
        //         if squared_distance > max_squared_distance {
        //             max_squared_distance = squared_distance;
        //         }
        //     }
        //     (center, max_squared_distance.sqrt())
        // }).collect();

        {
            let mut total_triangles = 0;
            let mut total_vertices = 0;

            for instance in scene_file.instances.iter() {
                let mesh_description = &scene_file.mesh_descriptions[instance.mesh_index as usize];
                total_triangles += mesh_description.triangle_count as u64;
                total_vertices += mesh_description.vertex_count as u64;
            }

            info!("Loaded {:?} with {} triangles and {} vertices", scene_file_path, total_triangles, total_vertices);
        }

        let (textures, materials) = {
            let mut textures: Vec<Texture> = scene_file
                .textures
                .iter()
                .map(|texture| load_dds_texture(gl, &scene_dir.join(&texture.file_path)).unwrap())
                .collect();

            let mut color_to_texture_index: HashMap<[u8; 3], usize> = HashMap::new();

            let mut color_texture_index = |color: [u8; 3]| match color_to_texture_index.get(&color) {
                Some(&index) => index,
                None => {
                    let texture = create_1x1_rgb_texture(gl, color);
                    let index = textures.len();
                    textures.push(texture);
                    color_to_texture_index.insert(color, index);
                    index
                }
            };

            let materials: Vec<Material> = scene_file
                .materials
                .iter()
                .map(|material| Material {
                    normal_texture_index: match material.normal_texture_index {
                        Some(file_texture_index) => file_texture_index.get() as usize,
                        None => color_texture_index([127, 127, 255]),
                    },
                    emissive_texture_index: match material.emissive_texture_index {
                        Some(file_texture_index) => file_texture_index.get() as usize,
                        None => {
                            let [r, g, b] = material.emissive_color;
                            color_texture_index(f32_3_to_unorm([r, g, b]))
                        }
                    },
                    ambient_texture_index: match material.ambient_texture_index {
                        Some(file_texture_index) => file_texture_index.get() as usize,
                        None => {
                            let [r, g, b] = material.ambient_color;
                            color_texture_index(f32_3_to_unorm([r, g, b]))
                        }
                    },
                    diffuse_texture_index: match material.diffuse_texture_index {
                        Some(file_texture_index) => file_texture_index.get() as usize,
                        None => {
                            let [r, g, b] = material.diffuse_color;
                            color_texture_index(f32_3_to_unorm([r, g, b]))
                        }
                    },
                    specular_texture_index: match material.specular_texture_index {
                        Some(file_texture_index) => file_texture_index.get() as usize,
                        None => {
                            let [r, g, b] = material.specular_color;
                            color_texture_index(f32_3_to_unorm([r, g, b]))
                        }
                    },
                    shininess: material.shininess,
                })
                .collect();

            (textures, materials)
        };

        let (scene_vao, scene_vb, scene_eb) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            fn align_16(n: usize) -> usize {
                ((n + 15) / 16) * 16
            }

            let mut total_byte_length = 0;

            let pos_in_obj_bytes = scene_file.pos_in_obj_buffer.vec_as_bytes();
            let pos_in_obj_byte_length = pos_in_obj_bytes.len();
            let pos_in_obj_byte_offset = total_byte_length;
            total_byte_length = align_16(total_byte_length + pos_in_obj_byte_length);

            let nor_in_obj_bytes = scene_file.nor_in_obj_buffer.vec_as_bytes();
            let nor_in_obj_byte_length = nor_in_obj_bytes.len();
            let nor_in_obj_byte_offset = total_byte_length;
            total_byte_length = align_16(total_byte_length + nor_in_obj_byte_length);

            let bin_in_obj_bytes = scene_file.bin_in_obj_buffer.vec_as_bytes();
            let bin_in_obj_byte_length = bin_in_obj_bytes.len();
            let bin_in_obj_byte_offset = total_byte_length;
            total_byte_length = align_16(total_byte_length + bin_in_obj_byte_length);

            let tan_in_obj_bytes = scene_file.tan_in_obj_buffer.vec_as_bytes();
            let tan_in_obj_byte_length = tan_in_obj_bytes.len();
            let tan_in_obj_byte_offset = total_byte_length;
            total_byte_length = align_16(total_byte_length + tan_in_obj_byte_length);

            let pos_in_tex_bytes = scene_file.pos_in_tex_buffer.vec_as_bytes();
            let pos_in_tex_byte_length = pos_in_tex_bytes.len();
            let pos_in_tex_byte_offset = total_byte_length;
            total_byte_length = align_16(total_byte_length + pos_in_tex_byte_length);

            // Upload data.
            gl.named_buffer_reserve(vb, total_byte_length, gl::STATIC_DRAW);
            gl.named_buffer_sub_data(vb, pos_in_obj_byte_offset, pos_in_obj_bytes);
            gl.named_buffer_sub_data(vb, nor_in_obj_byte_offset, nor_in_obj_bytes);
            gl.named_buffer_sub_data(vb, bin_in_obj_byte_offset, bin_in_obj_bytes);
            gl.named_buffer_sub_data(vb, tan_in_obj_byte_offset, tan_in_obj_bytes);
            gl.named_buffer_sub_data(vb, pos_in_tex_byte_offset, pos_in_tex_bytes);
            gl.named_buffer_data(eb, scene_file.triangle_buffer.vec_as_bytes(), gl::STATIC_DRAW);

            // Attribute layout specification.
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_OBJ_LOC, 3, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_format(vao, rendering::VS_NOR_IN_OBJ_LOC, 3, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_format(vao, rendering::VS_BIN_IN_OBJ_LOC, 3, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_format(vao, rendering::VS_TAN_IN_OBJ_LOC, 3, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_TEX_LOC, 2, gl::FLOAT, false, 0);

            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_OBJ_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_NOR_IN_OBJ_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_BIN_IN_OBJ_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_TAN_IN_OBJ_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_TEX_LOC);

            // Attribute source specification.
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_OBJ_LOC, BBI_00);
            gl.vertex_array_attrib_binding(vao, rendering::VS_NOR_IN_OBJ_LOC, BBI_01);
            gl.vertex_array_attrib_binding(vao, rendering::VS_BIN_IN_OBJ_LOC, BBI_02);
            gl.vertex_array_attrib_binding(vao, rendering::VS_TAN_IN_OBJ_LOC, BBI_03);
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_TEX_LOC, BBI_04);

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
                bin_in_obj_byte_offset,
                std::mem::size_of::<[f32; 3]>() as u32,
            );

            gl.vertex_array_vertex_buffer(
                vao,
                BBI_03,
                vb,
                tan_in_obj_byte_offset,
                std::mem::size_of::<[f32; 3]>() as u32,
            );

            gl.vertex_array_vertex_buffer(
                vao,
                BBI_04,
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
            materials,
            textures,
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
