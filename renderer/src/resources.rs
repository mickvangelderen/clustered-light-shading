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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MaterialKind {
    Opaque,
    Masked,
    Transparent,
}

pub struct Material {
    pub kind: MaterialKind,
    pub normal_texture_index: usize,
    pub emissive_texture_index: usize,
    pub ambient_texture_index: usize,
    pub diffuse_texture_index: usize,
    pub specular_texture_index: usize,
}

pub struct Texture {
    pub name: gl::TextureName,
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
    pub quad_vao: gl::VertexArrayName,
    pub quad_vb: gl::BufferName,
    pub quad_eb: gl::BufferName,

    pub full_screen_vao: gl::VertexArrayName,
    pub full_screen_vb: gl::BufferName,
    pub full_screen_eb: gl::BufferName,

    pub icosphere1280: icosphere1280::Resources,

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

        Texture { name }
    }
}

pub const BBI_00: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(0);
pub const BBI_01: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(1);
pub const BBI_02: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(2);
pub const BBI_03: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(3);
pub const BBI_04: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(4);
pub const BBI_05: gl::VertexArrayBufferBindingIndex = gl::VertexArrayBufferBindingIndex::from_u32(5);

pub const F32_3: gl::AttributeFormat = gl::AttributeFormat::F(gl::AttributeFormatF::F32(gl::ComponentCount::P3));
pub const F32_2: gl::AttributeFormat = gl::AttributeFormat::F(gl::AttributeFormatF::F32(gl::ComponentCount::P2));
pub const U32_1: gl::AttributeFormat = gl::AttributeFormat::I(gl::AttributeFormatI::U32(gl::ComponentCount::P1));

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

        Ok(Texture { name })
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
                    kind: if material.transparent {
                        MaterialKind::Transparent
                    } else {
                        if material.masked {
                            MaterialKind::Masked
                        } else {
                            MaterialKind::Opaque
                        }
                    },
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
                })
                .collect();

            (textures, materials)
        };

        let (scene_vao, scene_vb, scene_eb) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            fn align_256(n: usize) -> usize {
                ((n + 255) / 256) * 256
            }

            let instance_index_buffer: Vec<u32> = scene_file
                .instances
                .iter()
                .enumerate()
                .map(|(index, _)| index as u32)
                .collect();

            let spec = [
                (
                    rendering::VS_POS_IN_OBJ_LOC,
                    F32_3,
                    None,
                    BBI_00,
                    scene_file.pos_in_obj_buffer.vec_as_bytes(),
                ),
                (
                    rendering::VS_NOR_IN_OBJ_LOC,
                    F32_3,
                    None,
                    BBI_01,
                    scene_file.nor_in_obj_buffer.vec_as_bytes(),
                ),
                (
                    rendering::VS_BIN_IN_OBJ_LOC,
                    F32_3,
                    None,
                    BBI_02,
                    scene_file.bin_in_obj_buffer.vec_as_bytes(),
                ),
                (
                    rendering::VS_TAN_IN_OBJ_LOC,
                    F32_3,
                    None,
                    BBI_03,
                    scene_file.tan_in_obj_buffer.vec_as_bytes(),
                ),
                (
                    rendering::VS_POS_IN_TEX_LOC,
                    F32_2,
                    None,
                    BBI_04,
                    scene_file.pos_in_tex_buffer.vec_as_bytes(),
                ),
                (
                    rendering::VS_INSTANCE_INDEX_LOC,
                    U32_1,
                    Some(1),
                    BBI_05,
                    instance_index_buffer.vec_as_bytes(),
                ),
            ];

            let mut capacity = 0;
            for &(_, _, _, _, bytes) in spec.iter() {
                capacity = align_256(capacity + bytes.len());
            }
            gl.named_buffer_reserve(vb, capacity, gl::STATIC_DRAW);

            let mut offset = 0;
            for &(location, format, divisor, binding, bytes) in spec.iter() {
                // Upload bytes to buffer.
                gl.named_buffer_sub_data(vb, offset, bytes);

                // Specify format.
                gl.vertex_array_attrib_format(vao, location, format, 0);
                if let Some(divisor) = divisor {
                    gl.vertex_array_binding_divisor(vao, binding, divisor);
                }
                gl.enable_vertex_array_attrib(vao, location);
                gl.vertex_array_attrib_binding(vao, location, binding);

                // Connect format and buffer.
                gl.vertex_array_vertex_buffer(vao, binding, vb, offset, format.byte_size());

                offset = align_256(offset + bytes.len());
            }

            gl.named_buffer_data(eb, scene_file.triangle_buffer.vec_as_bytes(), gl::STATIC_DRAW);
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

        let (quad_vao, quad_vb, quad_eb) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            // Set up attributes.
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_TEX_LOC, F32_2, 0);
            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_TEX_LOC);
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_TEX_LOC, BBI_00);

            let vertices: [Point2<f32>; 4] = [
                Point2::new(0.0, 0.0),
                Point2::new(1.0, 0.0),
                Point2::new(1.0, 1.0),
                Point2::new(0.0, 1.0),
            ];

            let indices: [u32; 6] = [
                0, 1, 2, //
                2, 3, 0, //
            ];

            // Bind buffers to vao.
            let stride = std::mem::size_of::<Point2<f32>>() as u32;
            gl.vertex_array_vertex_buffer(vao, BBI_00, vb, 0, stride);
            gl.vertex_array_element_buffer(vao, eb);

            // Upload data.
            gl.named_buffer_data(vb, vertices.slice_as_bytes(), gl::STATIC_DRAW);
            gl.named_buffer_data(eb, indices.slice_as_bytes(), gl::STATIC_DRAW);

            (vao, vb, eb)
        };

        let (full_screen_vao, full_screen_vb, full_screen_eb) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            // Set up attributes.
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_TEX_LOC, F32_2, 0);
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

                    Some(PointLight {
                        tint: Vector3::from(emissive_color).normalize().into(),
                        position: pos_in_wld,
                        attenuation: light::AttenCoefs::from(configuration.light.attenuation).cast().unwrap(),
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
            quad_vao,
            quad_vb,
            quad_eb,
            icosphere1280: unsafe { icosphere1280::Resources::new(gl) },
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

fn clear_and_reserve<T>(v: &mut Vec<T>, n: usize) {
    v.clear();
    if v.capacity() < n {
        v.reserve(n - v.capacity())
    }
}

#[derive(Debug)]
pub struct WorldTransforms {
    pub obj_to_wld: Vec<Matrix4<f64>>,
    pub wld_to_obj: Vec<Matrix4<f64>>,
    pub compute_world_transforms_profiler: profiling::SampleIndex,
}

impl WorldTransforms {
    pub fn new(profiling_context: &mut ProfilingContext) -> Self {
        Self {
            obj_to_wld: Default::default(),
            wld_to_obj: Default::default(),
            compute_world_transforms_profiler: profiling_context.add_sample("world transforms"),
        }
    }

    pub fn recompute(
        &mut self,
        gl: &gl::Gl,
        profiling_context: &mut ProfilingContext,
        scene_file: &scene_file::SceneFile,
    ) {
        let profiler_index = profiling_context.start(gl, self.compute_world_transforms_profiler);

        let scene_file::SceneFile {
            ref instances,
            ref transforms,
            ..
        } = *scene_file;

        clear_and_reserve(&mut self.obj_to_wld, instances.len());
        self.obj_to_wld.extend(instances.iter().map(|instance| {
            let transform = &transforms[instance.transform_index as usize];
            let obj_to_wld = transform.to_parent();
            obj_to_wld
        }));

        clear_and_reserve(&mut self.wld_to_obj, instances.len());
        self.wld_to_obj
            .extend(self.obj_to_wld.iter().map(|obj_to_wld| obj_to_wld.invert().unwrap()));

        profiling_context.stop(gl, profiler_index);
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct InstanceMatrices {
    pub obj_to_ren_clp: Matrix4<f32>,
    pub obj_to_lgt: Matrix4<f32>,
    pub obj_to_lgt_inv_tra: Matrix4<f32>,
}

pub enum ProjectionKind {
    Perspective,
    Orthographic,
}

pub struct CullingCamera {
    pub wld_to_cam: Matrix4<f64>,
    pub frustum: Frustum<f64>,
    pub projection_kind: ProjectionKind,
}

fn intersect_sphere_enlarged_frustum(sphere: scene_file::Sphere3<f64>, frustum: Frustum<f64>) -> bool {
    let nx0 = Vector2::new(-1.0, -frustum.x0).normalize();
    let nx1 = Vector2::new(1.0, frustum.x1).normalize();
    let ny0 = Vector2::new(-1.0, -frustum.y0).normalize();
    let ny1 = Vector2::new(1.0, frustum.y1).normalize();
    ((frustum.z0 - sphere.p.z) < sphere.r)
        && ((sphere.p.z - frustum.z1) < sphere.r)
        && (Vector2::dot(nx0, Vector2::new(sphere.p.x, sphere.p.z)) < sphere.r)
        && (Vector2::dot(nx1, Vector2::new(sphere.p.x, sphere.p.z)) < sphere.r)
        && (Vector2::dot(ny0, Vector2::new(sphere.p.y, sphere.p.z)) < sphere.r)
        && (Vector2::dot(ny1, Vector2::new(sphere.p.y, sphere.p.z)) < sphere.r)
}

fn intersect_sphere_box(sphere: scene_file::Sphere3<f64>, box3: scene_file::Box3<f64>) -> bool {
    let mut r_sq_acc = 0.0;
    for axis in 0..3 {
        let d = sphere.p[axis] - box3.p0[axis];
        if d < -0.0 {
            r_sq_acc += d * d;
        }
        let d = sphere.p[axis] - box3.p1[axis];
        if d > 0.0 {
            r_sq_acc += d * d;
        }
    }
    r_sq_acc < sphere.r * sphere.r
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

    // Profiling
    pub compute_instance_matrices_profiler: profiling::SampleIndex,
    pub compute_draw_commands_profiler: profiling::SampleIndex,
}

impl DrawResources {
    pub fn new(gl: &gl::Gl, profiling_context: &mut ProfilingContext) -> Self {
        Self {
            instance_matrices_buffer: unsafe { gl.create_buffer() },
            draw_command_buffer: unsafe { gl.create_buffer() },

            instance_matrices_data: Vec::new(),
            draw_command_data: Vec::new(),
            draw_offsets: Vec::new(),
            draw_counts: Vec::new(),

            compute_instance_matrices_profiler: profiling_context.add_sample("inst matrs"),
            compute_draw_commands_profiler: profiling_context.add_sample("draw cmds"),
        }
    }

    pub fn recompute(
        &mut self,
        gl: &gl::Gl,
        profiling_context: &mut ProfilingContext,
        culling_camera: CullingCamera,
        wld_to_ren_clp: Matrix4<f64>,
        world_transforms: &WorldTransforms,
        materials: &[Material],
        scene_file: &scene_file::SceneFile,
    ) {
        let scene_file::SceneFile {
            ref instances,
            ref mesh_descriptions,
            ..
        } = *scene_file;

        {
            let profiler_index = profiling_context.start(gl, self.compute_instance_matrices_profiler);

            let instance_count = instances.len();

            clear_and_reserve(&mut self.instance_matrices_data, instance_count);
            self.instance_matrices_data
                .extend((0..instance_count).into_iter().map(|instance_index| {
                    let obj_to_wld = world_transforms.obj_to_wld[instance_index];
                    let wld_to_obj = world_transforms.wld_to_obj[instance_index];
                    InstanceMatrices {
                        obj_to_ren_clp: (wld_to_ren_clp * obj_to_wld).cast().unwrap(),
                        obj_to_lgt: obj_to_wld.cast().unwrap(),
                        obj_to_lgt_inv_tra: wld_to_obj.transpose().cast().unwrap(),
                    }
                }));

            unsafe {
                gl.named_buffer_data(
                    self.instance_matrices_buffer,
                    self.instance_matrices_data.vec_as_bytes(),
                    gl::DYNAMIC_DRAW,
                );
            }

            profiling_context.stop(gl, profiler_index);
        }

        {
            let profiler_index = profiling_context.start(gl, self.compute_draw_commands_profiler);

            let visible_instance_indices: Vec<usize> = instances
                .iter()
                .enumerate()
                .filter_map(|(instance_index, instance)| {
                    let obj_to_cam = culling_camera.wld_to_cam * world_transforms.obj_to_wld[instance_index];

                    let mesh_description = &mesh_descriptions[instance.mesh_index as usize];
                    let sphere_obj = mesh_description.bounding_sphere.cast::<f64>();

                    let sphere_cam = scene_file::Sphere3 {
                        p: obj_to_cam.transform_point(sphere_obj.p),
                        r: {
                            let r_cam = obj_to_cam
                                .transform_vector(Vector3::from_value(sphere_obj.r))
                                .map(f64::abs);
                            r_cam[r_cam.dominant_axis()]
                        },
                    };

                    if match culling_camera.projection_kind {
                        ProjectionKind::Orthographic => intersect_sphere_box(sphere_cam, {
                            let Frustum { x0, x1, y0, y1, z0, z1 } = culling_camera.frustum;
                            scene_file::Box3 {
                                p0: Point3::new(x0, y0, z0),
                                p1: Point3::new(x1, y1, z1),
                            }
                        }),

                        ProjectionKind::Perspective => {
                            intersect_sphere_enlarged_frustum(sphere_cam, culling_camera.frustum)
                        }
                    } {
                        Some(instance_index)
                    } else {
                        None
                    }
                })
                .collect();

            // Prefix sum draw counts per material.
            clear_and_reserve(&mut self.draw_counts, materials.len());
            self.draw_counts.extend(std::iter::repeat(0).take(materials.len()));

            for &instance_index in visible_instance_indices.iter() {
                let material_index = instances[instance_index].material_index;
                self.draw_counts[material_index as usize] += 1;
            }

            clear_and_reserve(&mut self.draw_offsets, materials.len());
            self.draw_offsets
                .extend(self.draw_counts.iter().scan(0, |offset, &count| {
                    let result = Some(*offset);
                    *offset += count;
                    result
                }));

            // Clear counts and initialize draw command buffer.
            for draw_count in self.draw_counts.iter_mut() {
                *draw_count = 0;
            }

            clear_and_reserve(&mut self.draw_command_data, visible_instance_indices.len());
            self.draw_command_data.extend(
                std::iter::repeat(DrawCommand {
                    count: 0,
                    prim_count: 0,
                    first_index: 0,
                    base_vertex: 0,
                    base_instance: 0,
                })
                .take(visible_instance_indices.len()),
            );

            // Fill out the buffer.

            for &instance_index in visible_instance_indices.iter() {
                let instance = &instances[instance_index];
                let material_index = instance.material_index as usize;
                let mesh_description = &mesh_descriptions[instance.mesh_index as usize];
                let command_index = self.draw_offsets[material_index] + self.draw_counts[material_index];
                self.draw_command_data[command_index] = DrawCommand {
                    count: mesh_description.element_count(),
                    prim_count: 1,
                    first_index: mesh_description.element_offset(),
                    base_vertex: mesh_description.vertex_offset,
                    base_instance: instance_index as u32,
                };
                self.draw_counts[material_index] += 1;
            }

            unsafe {
                gl.named_buffer_data(
                    self.draw_command_buffer,
                    self.draw_command_data.vec_as_bytes(),
                    gl::DYNAMIC_DRAW,
                );
            }

            profiling_context.stop(gl, profiler_index);
        }
    }
}
