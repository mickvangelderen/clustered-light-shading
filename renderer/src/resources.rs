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

    pub draw_resources_pool: Pool<DrawResources>,
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
pub const BBI_05: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(5);

fn load_dds_texture(gl: &gl::Gl, file_path: impl AsRef<Path>, srgb: bool) -> io::Result<Texture> {
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
                dds.header.pixel_format.to_gl_internal_format(srgb),
                layer.width as i32,
                layer.height as i32,
                &dds.bytes[layer.byte_offset..(layer.byte_offset + layer.byte_count)],
            );
        }

        gl.texture_parameterf(name, gl::TEXTURE_MAX_ANISOTROPY, 16.0);

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

            info!(
                "Loaded {:?} with {} instances, {} triangles and {} vertices",
                scene_file_path,
                scene_file.instances.len(),
                total_triangles,
                total_vertices
            );
        }

        let (textures, materials) = {
            // NOTE(mickvangelderen): This is a bit silly, should determine this in the scene file.
            let mut textures: Vec<Texture> = scene_file
                .textures
                .iter()
                .enumerate()
                .map(|(texture_index, texture)| {
                    let srgb = scene_file.materials.iter().any(|material| {
                        material
                            .diffuse_texture_index
                            .map(|i| i.get() as usize == texture_index)
                            .unwrap_or(false)
                    });
                    load_dds_texture(gl, &scene_dir.join(&texture.file_path), srgb).unwrap()
                })
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

            let instance_index_buffer: Vec<u32> = scene_file
                .instances
                .iter()
                .enumerate()
                .map(|(index, _)| index as u32)
                .collect();
            let instance_index_bytes = instance_index_buffer.vec_as_bytes();
            let instance_index_byte_length = instance_index_bytes.len();
            let instance_index_byte_offset = total_byte_length;
            total_byte_length = align_16(total_byte_length + instance_index_byte_length);

            // Upload data.
            gl.named_buffer_reserve(vb, total_byte_length, gl::STATIC_DRAW);
            gl.named_buffer_sub_data(vb, pos_in_obj_byte_offset, pos_in_obj_bytes);
            gl.named_buffer_sub_data(vb, nor_in_obj_byte_offset, nor_in_obj_bytes);
            gl.named_buffer_sub_data(vb, bin_in_obj_byte_offset, bin_in_obj_bytes);
            gl.named_buffer_sub_data(vb, tan_in_obj_byte_offset, tan_in_obj_bytes);
            gl.named_buffer_sub_data(vb, pos_in_tex_byte_offset, pos_in_tex_bytes);
            gl.named_buffer_sub_data(vb, instance_index_byte_offset, instance_index_bytes);
            gl.named_buffer_data(eb, scene_file.triangle_buffer.vec_as_bytes(), gl::STATIC_DRAW);

            // Attribute layout specification.
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_OBJ_LOC, 3, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_format(vao, rendering::VS_NOR_IN_OBJ_LOC, 3, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_format(vao, rendering::VS_BIN_IN_OBJ_LOC, 3, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_format(vao, rendering::VS_TAN_IN_OBJ_LOC, 3, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_TEX_LOC, 2, gl::FLOAT, false, 0);
            gl.vertex_array_attrib_i_format(vao, rendering::VS_INSTANCE_INDEX_LOC, 1, gl::UNSIGNED_INT, 0);

            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_OBJ_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_NOR_IN_OBJ_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_BIN_IN_OBJ_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_TAN_IN_OBJ_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_TEX_LOC);
            gl.enable_vertex_array_attrib(vao, rendering::VS_INSTANCE_INDEX_LOC);

            // Attribute source specification.
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_OBJ_LOC, BBI_00);
            gl.vertex_array_attrib_binding(vao, rendering::VS_NOR_IN_OBJ_LOC, BBI_01);
            gl.vertex_array_attrib_binding(vao, rendering::VS_BIN_IN_OBJ_LOC, BBI_02);
            gl.vertex_array_attrib_binding(vao, rendering::VS_TAN_IN_OBJ_LOC, BBI_03);
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_TEX_LOC, BBI_04);
            gl.vertex_array_attrib_binding(vao, rendering::VS_INSTANCE_INDEX_LOC, BBI_05);

            gl.vertex_array_binding_divisor(vao, BBI_05, 1);

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

            gl.vertex_array_vertex_buffer(
                vao,
                BBI_05,
                vb,
                instance_index_byte_offset,
                std::mem::size_of::<u32>() as u32,
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

        let point_lights: Vec<PointLight> = scene_file
            .instances
            .iter()
            .flat_map(|instance| {
                let material_index = instance.material_index as usize;
                let emissive_color = scene_file.materials[material_index].emissive_color;

                if emissive_color != [0.0; 3] {
                    let mesh_description_index = instance.mesh_index as usize;
                    let mesh_description = &scene_file.mesh_descriptions[mesh_description_index];
                    let vertex_offset = mesh_description.vertex_offset as usize;
                    let vertex_count = mesh_description.vertex_count as usize;
                    let vertex_iter = scene_file.pos_in_obj_buffer[vertex_offset..(vertex_offset + vertex_count)]
                        .iter()
                        .map(|&pos_in_obj| {
                            Point3::new(
                                pos_in_obj[0].get() as f64,
                                pos_in_obj[1].get() as f64,
                                pos_in_obj[2].get() as f64,
                            )
                        });
                    let center = vertex_iter.fold(Point3::origin(), |mut acc, p| {
                        acc += p.to_vec();
                        acc
                    }) * (1.0 / vertex_count as f64);

                    let transform = &scene_file.transforms[instance.transform_index as usize];
                    let pos_from_obj_to_wld = transform.to_parent();
                    let pos_in_wld = pos_from_obj_to_wld.transform_point(center).cast().unwrap();

                    let color = RGB {
                        r: emissive_color[0],
                        g: emissive_color[1],
                        b: emissive_color[2],
                    };

                    Some(PointLight {
                        ambient: color,
                        diffuse: color,
                        specular: color,
                        pos_in_wld,
                        attenuation: AttenParams {
                            intensity: 4.0,
                            clip_near: 0.5,
                            cutoff: 0.2,
                        }
                        .into(),
                    })
                } else {
                    None
                }
            })
            .collect();

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
            point_lights,
            draw_resources_pool: Default::default(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct InstanceMatrices {
    pub obj_to_ren_clp: Matrix4<f32>,
    pub obj_to_clu_cam: Matrix4<f32>,
    pub obj_to_lgt: Matrix4<f32>,
    pub obj_to_lgt_inv_tra: Matrix4<f32>,
}

pub fn compute_instance_matrices(
    wld_to_ren_clp: Matrix4<f64>,
    wld_to_clu_cam: Matrix4<f64>,
    instances: &[scene_file::Instance],
    transforms: &[scene_file::Transform],
) -> Vec<InstanceMatrices> {
    // let start = std::time::Instant::now();

    let instance_matrices: Vec<InstanceMatrices> = instances
        .iter()
        .map(|instance| {
            let transform = &transforms[instance.transform_index as usize];
            let obj_to_wld = transform.to_parent();
            InstanceMatrices {
                obj_to_ren_clp: (wld_to_ren_clp * obj_to_wld).cast().unwrap(),
                obj_to_clu_cam: (wld_to_clu_cam * obj_to_wld).cast().unwrap(),
                obj_to_lgt: obj_to_wld.cast().unwrap(),
                obj_to_lgt_inv_tra: (obj_to_wld.invert().unwrap().transpose()).cast().unwrap(),
            }
        })
        .collect();

    // info!("compute instance matrices elapsed {:?}", start.elapsed());

    instance_matrices
}

pub struct DrawCommandResources {
    pub counts: Vec<usize>,
    pub offsets: Vec<usize>,
    pub buffer: Vec<DrawCommand>,
}

pub fn compute_draw_commands(
    instances: &[scene_file::Instance],
    materials: &[Material],
    mesh_descriptions: &[scene_file::MeshDescription],
) -> DrawCommandResources {
    // let start = std::time::Instant::now();

    // Prefix sum draw counts per material.
    let mut counts: Vec<usize> =
        instances
            .iter()
            .fold(materials.iter().map(|_| 0).collect(), |mut counts, instance| {
                counts[instance.material_index as usize] += 1;
                counts
            });

    let offsets: Vec<usize> = counts
        .iter()
        .scan(0, |offset, &count| {
            let result = Some(*offset);
            *offset += count;
            result
        })
        .collect();

    // Clear counts and initialize buffer.

    for draw_count in counts.iter_mut() {
        *draw_count = 0;
    }

    let mut buffer: Vec<DrawCommand> = std::iter::repeat(DrawCommand {
        count: 0,
        prim_count: 0,
        first_index: 0,
        base_vertex: 0,
        base_instance: 0,
    })
    .take(instances.len())
    .collect();

    // Fill out the buffer.

    for (instance_index, instance) in instances.iter().enumerate() {
        let material_index = instance.material_index as usize;
        let mesh_description = &mesh_descriptions[instance.mesh_index as usize];
        let command_index = offsets[material_index] + counts[material_index];
        buffer[command_index] = DrawCommand {
            count: mesh_description.element_count(),
            prim_count: 1,
            first_index: mesh_description.element_offset(),
            base_vertex: mesh_description.vertex_offset,
            base_instance: instance_index as u32,
        };
        counts[material_index] += 1;
    }

    // info!("compute draw commands elapsed {:?}", start.elapsed());

    DrawCommandResources {
        counts,
        offsets,
        buffer,
    }
}

pub struct DrawResources {
    // GPU
    pub instance_matrices_buffer: gl::BufferName,
    pub draw_command_buffer: gl::BufferName,

    // CPU
    pub instance_matrices_data: Vec<InstanceMatrices>,
    pub draw_command_data: Vec<DrawCommand>,
    pub draw_counts: Vec<usize>,
    pub draw_offsets: Vec<usize>,
}

impl DrawResources {
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            instance_matrices_buffer: unsafe { gl.create_buffer() },
            draw_command_buffer: unsafe { gl.create_buffer() },

            instance_matrices_data: Vec::new(),
            draw_command_data: Vec::new(),
            draw_offsets: Vec::new(),
            draw_counts: Vec::new(),
        }
    }

    pub fn recompute(
        &mut self,
        wld_to_ren_clp: Matrix4<f64>,
        wld_to_clu_cam: Matrix4<f64>,
        instances: &[scene_file::Instance],
        materials: &[Material],
        transforms: &[scene_file::Transform],
        mesh_descriptions: &[scene_file::MeshDescription],
    ) {
        self.instance_matrices_data = compute_instance_matrices(wld_to_ren_clp, wld_to_clu_cam, instances, transforms);
        let DrawCommandResources {
            counts,
            offsets,
            buffer,
        } = compute_draw_commands(instances, materials, mesh_descriptions);
        self.draw_command_data = buffer;
        self.draw_counts = counts;
        self.draw_offsets = offsets;
    }

    pub fn reupload(&mut self, gl: &gl::Gl) {
        unsafe {
            gl.named_buffer_data(
                self.instance_matrices_buffer,
                self.instance_matrices_data.vec_as_bytes(),
                gl::DYNAMIC_DRAW,
            );
            gl.named_buffer_data(
                self.draw_command_buffer,
                self.draw_command_data.vec_as_bytes(),
                gl::DYNAMIC_DRAW,
            );
        }
    }
}
