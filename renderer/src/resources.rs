use crate::configuration;
use crate::convert::*;
use crate::keyboard_model;
use crate::light::*;
use crate::rendering;
use cgmath::*;
use gl_typed as gl;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::path::Path;
use std::path::PathBuf;

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

impl Vertex {
    fn set_format(gl: &gl::Gl, vao: gl::VertexArrayName, vb: gl::BufferName, eb: gl::BufferName) {
        macro_rules! set_attrib {
            ($vao: ident, $loc: expr, $component_count: expr, $component_type: expr, $Vertex: ident, $field: ident, $bbi: expr) => {
                gl.vertex_array_attrib_format(
                    $vao,
                    $loc,
                    $component_count,
                    $component_type,
                    false,
                    field_offset!($Vertex, $field) as u32,
                );
                gl.enable_vertex_array_attrib(vao, $loc);
                gl.vertex_array_attrib_binding(vao, $loc, $bbi);
            };
        }

        unsafe {
            set_attrib!(
                vao,
                rendering::VS_POS_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                Vertex,
                pos_in_obj,
                VERTEX_ARRAY_BUFFER_BINDING_INDEX
            );
            set_attrib!(
                vao,
                rendering::VS_POS_IN_TEX_LOC,
                2,
                gl::FLOAT,
                Vertex,
                pos_in_tex,
                VERTEX_ARRAY_BUFFER_BINDING_INDEX
            );
            set_attrib!(
                vao,
                rendering::VS_NOR_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                Vertex,
                nor_in_obj,
                VERTEX_ARRAY_BUFFER_BINDING_INDEX
            );
            set_attrib!(
                vao,
                rendering::VS_TAN_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                Vertex,
                tan_in_obj,
                VERTEX_ARRAY_BUFFER_BINDING_INDEX
            );

            // Bind buffers to vao.
            let stride = std::mem::size_of::<Vertex>() as u32;
            gl.vertex_array_vertex_buffer(vao, VERTEX_ARRAY_BUFFER_BINDING_INDEX, vb, 0, stride);
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
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub path_to_texture_index: HashMap<PathBuf, TextureIndex>,
    pub color_to_texture_index: HashMap<[u8; 4], TextureIndex>,
    pub scene_vao: gl::VertexArrayName,
    pub scene_vb: gl::BufferName,
    pub scene_eb: gl::BufferName,
    pub mesh_metas: Vec<MeshMeta>,
    pub key_indices: Vec<keyboard_model::UncheckedIndex>,
    pub point_lights: [PointLight; rendering::POINT_LIGHT_CAPACITY as usize],

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

#[inline]
fn create_texture(gl: &gl::Gl, img: image::RgbaImage) -> Texture {
    unsafe {
        let name = gl.create_texture(gl::TEXTURE_2D);

        gl.bind_texture(gl::TEXTURE_2D, name);
        gl.tex_image_2d(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8,
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

#[inline]
fn create_srgb_texture(gl: &gl::Gl, img: image::RgbaImage) -> Texture {
    unsafe {
        let name = gl.create_texture(gl::TEXTURE_2D);

        gl.bind_texture(gl::TEXTURE_2D, name);
        gl.tex_image_2d(
            gl::TEXTURE_2D,
            0,
            gl::SRGB8_ALPHA8,
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

const VERTEX_ARRAY_BUFFER_BINDING_INDEX: gl::VertexArrayBufferBindingIndex =
    gl::VertexArrayBufferBindingIndex::from_u32(0);

impl Resources {
    pub fn new<P: AsRef<Path>>(gl: &gl::Gl, resource_dir: P, configuration: &configuration::Root) -> Self {
        let resource_dir = resource_dir.as_ref();

        // ["two_planes.obj", "bunny.obj"]
        // ["shadow_test.obj"]
        let objs: Vec<(PathBuf, Vec<tobj::Model>, Vec<tobj::Material>)> = ["sponza/sponza.obj", "keyboard.obj"]
            .into_iter()
            .map(PathBuf::from)
            .map(|rel_file_path| {
                let file_path = resource_dir.join(&rel_file_path);
                println!("Loading {:?}.", file_path.display());
                let obj = tobj::load_obj(&file_path).unwrap();
                let vertex_count: usize = obj.0.iter().map(|model| model.mesh.indices.len()).sum();
                println!("Loaded {:?} with {} vertices.", file_path.display(), vertex_count);
                (rel_file_path, obj.0, obj.1)
            })
            .collect();

        let material_offsets: Vec<u32> = objs
            .iter()
            .scan(0, |sum, (_, _, ref materials)| {
                let offset = *sum;
                *sum += materials.len() as u32;
                Some(offset)
            })
            .collect();

        let mut meshes: Vec<Mesh> = Vec::with_capacity(objs.iter().map(|(_, ref models, _)| models.len()).sum());
        let mut materials: Vec<Material> =
            Vec::with_capacity(objs.iter().map(|(_, _, ref materials)| materials.len()).sum());

        let mut textures = Vec::new();
        let mut path_to_texture_index = HashMap::new();
        let mut color_to_texture_index = HashMap::new();

        for (i, (rel_file_path, obj_models, obj_materials)) in objs.into_iter().enumerate() {
            let material_offset = material_offsets[i];

            meshes.extend(obj_models.into_iter().map(|model| {
                let tobj::Model { mesh, name } = model;
                let tobj::Mesh {
                    positions,
                    normals: _normals,
                    texcoords,
                    indices,
                    material_id,
                } = mesh;

                let triangles: Vec<[u32; 3]> = indices.unflatten();
                let pos_in_obj: Vec<[f32; 3]> = positions.unflatten();

                let pos_in_tex: Vec<[f32; 2]> = texcoords.unflatten();
                let nor_in_obj = polygen::compute_normals(&triangles, &pos_in_obj);
                let tan_in_obj = if pos_in_tex.is_empty() {
                    Vec::new()
                } else {
                    polygen::compute_tangents(&triangles, &pos_in_obj, &pos_in_tex)
                };
                let material_index = material_id.map(|id| id as MaterialIndex + material_offset);

                Mesh {
                    name,
                    triangles,
                    pos_in_obj: if rel_file_path == Path::new("sponza/sponza.obj") {
                        pos_in_obj
                            .into_iter()
                            .map(|p| (Vector3::from(p) * 0.01).into())
                            .collect()
                    } else {
                        pos_in_obj
                    },
                    pos_in_tex,
                    nor_in_obj,
                    tan_in_obj,
                    translate: if rel_file_path == Path::new("keyboard.obj") {
                        Vector3::new(0.0, 0.3, 0.0)
                    } else {
                        Vector3::zero()
                    },
                    material_index,
                }
            }));

            let material_dir = resource_dir.join(rel_file_path.parent().unwrap());

            materials.extend(obj_materials.into_iter().map(|material| {
                // https://github.com/rust-lang/rfcs/pull/1769
                let diffuse = *if !material.diffuse_texture.is_empty() {
                    let file_path = material_dir.join(&material.diffuse_texture);
                    path_to_texture_index.entry(file_path.clone()).or_insert_with(|| {
                        println!("Loading diffuse texture {:?}", &file_path);
                        let img = image::open(&file_path)
                            .expect("Failed to load image.")
                            .flipv()
                            .to_rgba();
                        println!("Loaded diffuse texture {:?}", &file_path);

                        let texture = if configuration.global.diffuse_srgb {
                            create_srgb_texture(gl, img)
                        } else {
                            create_texture(gl, img)
                        };

                        let index = textures.len() as TextureIndex;
                        textures.push(texture);
                        index
                    })
                } else {
                    let rgba_u8 =
                        rgba_f32_to_rgba_u8([material.diffuse[0], material.diffuse[1], material.diffuse[2], 1.0]);
                    color_to_texture_index.entry(rgba_u8).or_insert_with(|| {
                        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba(rgba_u8));

                        let texture = if configuration.global.diffuse_srgb {
                            create_srgb_texture(gl, img)
                        } else {
                            create_texture(gl, img)
                        };

                        let index = textures.len() as TextureIndex;
                        textures.push(texture);
                        index
                    })
                };

                let normal = *if let Some(bump_path) = material.unknown_param.get("map_bump") {
                    let file_path = material_dir.join(bump_path);
                    path_to_texture_index.entry(file_path.clone()).or_insert_with(|| {
                        println!("Loading normal texture {:?}", &file_path);
                        let img = image::open(&file_path)
                            .expect("Failed to load image.")
                            .flipv()
                            .to_rgba();
                        println!("Loaded normal texture {:?}", &file_path);

                        let texture = create_texture(gl, img);
                        let index = textures.len() as TextureIndex;
                        textures.push(texture);
                        index
                    })
                } else {
                    let rgba_u8 = rgba_f32_to_rgba_u8([0.5, 0.5, 0.5, 1.0]);
                    color_to_texture_index.entry(rgba_u8).or_insert_with(|| {
                        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba(rgba_u8));

                        let texture = create_texture(gl, img);
                        let index = textures.len() as TextureIndex;
                        textures.push(texture);
                        index
                    })
                };

                let specular = *if !material.specular_texture.is_empty() {
                    let file_path = material_dir.join(&material.specular_texture);
                    path_to_texture_index.entry(file_path.clone()).or_insert_with(|| {
                        println!("Loading specular texture {:?}", &file_path);
                        let img = image::open(&file_path)
                            .expect("Failed to load image.")
                            .flipv()
                            .to_rgba();
                        println!("Loaded specular texture {:?}", &file_path);

                        let texture = create_texture(gl, img);
                        let index = textures.len() as TextureIndex;
                        textures.push(texture);
                        index
                    })
                } else {
                    let rgba_u8 =
                        rgba_f32_to_rgba_u8([material.specular[0], material.specular[1], material.specular[2], 1.0]);
                    color_to_texture_index.entry(rgba_u8).or_insert_with(|| {
                        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba(rgba_u8));

                        let texture = create_texture(gl, img);
                        let index = textures.len() as TextureIndex;
                        textures.push(texture);
                        index
                    })
                };

                Material {
                    diffuse,
                    normal,
                    specular,
                    shininess: material.shininess,
                }
            }));
        }

        let key_indices: Vec<keyboard_model::UncheckedIndex> = meshes
            .iter()
            .map(|mesh| model_name_to_keyboard_index(&mesh.name))
            .collect();

        let mut vertex_data: Vec<Vertex> = Vec::new();
        let mut element_data: Vec<[u32; 3]> = Vec::new();
        let mut mesh_metas: Vec<MeshMeta> = Vec::new();

        for mesh in meshes.iter() {
            let vertex_count = mesh.pos_in_obj.len();
            assert_eq!(vertex_count, mesh.pos_in_obj.len());
            assert_eq!(vertex_count, mesh.pos_in_tex.len());
            assert_eq!(vertex_count, mesh.nor_in_obj.len());
            assert_eq!(vertex_count, mesh.tan_in_obj.len());

            // Get this BEFORE appending.
            let vertex_base = u32::try_from(vertex_data.len()).unwrap();

            for i in 0..vertex_count {
                vertex_data.push(Vertex {
                    pos_in_obj: mesh.pos_in_obj[i],
                    pos_in_tex: mesh.pos_in_tex[i],
                    nor_in_obj: mesh.nor_in_obj[i],
                    tan_in_obj: mesh.tan_in_obj[i],
                });
            }

            // Get this BEFORE appending.
            let element_offset = std::mem::size_of_val(&element_data[..]);

            element_data.extend(mesh.triangles.iter());

            mesh_metas.push(MeshMeta {
                element_count: (mesh.triangles.len() * 3).try_into().unwrap(),
                element_offset,
                vertex_base,
                vertex_count: vertex_count.try_into().unwrap(),
            });
        }

        let (scene_vao, scene_vb, scene_eb) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            Vertex::set_format(gl, vao, vb, eb);

            // Upload data.
            gl.named_buffer_data(vb, vertex_data.vec_as_bytes(), gl::STATIC_DRAW);
            gl.named_buffer_data(eb, element_data.vec_as_bytes(), gl::STATIC_DRAW);

            (vao, vb, eb)
        };

        let (full_screen_vao, full_screen_vb, full_screen_eb) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            // Set up attributes.
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_TEX_LOC, 2, gl::FLOAT, false, 0);
            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_TEX_LOC);
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_TEX_LOC, VERTEX_ARRAY_BUFFER_BINDING_INDEX);

            // Bind buffers to vao.
            let stride = std::mem::size_of::<[f32; 2]>() as u32;
            gl.vertex_array_vertex_buffer(vao, VERTEX_ARRAY_BUFFER_BINDING_INDEX, vb, 0, stride);
            gl.vertex_array_element_buffer(vao, eb);

            // Upload data.
            gl.named_buffer_data(vb, FULL_SCREEN_VERTICES.slice_to_bytes(), gl::STATIC_DRAW);
            gl.named_buffer_data(eb, FULL_SCREEN_INDICES.slice_to_bytes(), gl::STATIC_DRAW);

            (vao, vb, eb)
        };

        let (cluster_vao, cluster_vb, cluster_eb, cluster_element_count) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            Vertex::set_format(gl, vao, vb, eb);

            // Upload data.
            let mesh = generate_cube_mesh((0.1, 0.9), (0.1, 0.9), (0.1, 0.9));
            gl.named_buffer_data(vb, mesh.vertices.value_as_bytes(), gl::STATIC_DRAW);
            gl.named_buffer_data(eb, mesh.indices.value_as_bytes(), gl::STATIC_DRAW);

            (vao, vb, eb, (mesh.indices.len() * 3) as u32)
        };

        Resources {
            meshes,
            materials,
            textures,
            path_to_texture_index,
            color_to_texture_index,
            scene_vao,
            scene_vb,
            scene_eb,
            mesh_metas,
            key_indices,
            full_screen_vao,
            full_screen_vb,
            full_screen_eb,
            cluster_vao,
            cluster_vb,
            cluster_eb,
            cluster_element_count,
            point_lights: [
                PointLight {
                    ambient: RGB::new(0.2000, 0.2000, 0.2000),
                    diffuse: RGB::new(4.0000, 4.0000, 4.0000),
                    specular: RGB::new(1.0000, 1.0000, 1.0000),
                    pos_in_pnt: Point3::new(-12.9671, 1.8846, -4.4980),
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
                    pos_in_pnt: Point3::new(-11.9563, 2.6292, 3.8412),
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
                    pos_in_pnt: Point3::new(13.6090, 2.6292, 3.3216),
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
                    pos_in_pnt: Point3::new(12.5982, 1.8846, -5.0176),
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
                    pos_in_pnt: Point3::new(3.3116, 4.3440, 5.1447),
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
                    pos_in_pnt: Point3::new(8.8820, 6.7391, -1.0279),
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
                    pos_in_pnt: Point3::new(-4.6988, 10.0393, 0.8667),
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
                    pos_in_pnt: Point3::new(-4.6816, 1.0259, -2.1767),
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

fn model_name_to_keyboard_index(name: &str) -> keyboard_model::UncheckedIndex {
    let code = match name {
        "Key_RIGHT_CONTROL_Key_LP.008" => Some(glutin::VirtualKeyCode::LControl),
        "Key_MENU_Key_LP.009" => Some(glutin::VirtualKeyCode::Apps),
        "Key_RIGHT_SUPER_Key_LP.010" => Some(glutin::VirtualKeyCode::RWin),
        "Key_RIGHT_ALT_Key_LP.011" => Some(glutin::VirtualKeyCode::RAlt),
        "Key_ESCAPE_Key_LP.012" => Some(glutin::VirtualKeyCode::Escape),
        "Key_LEFT_CONTROL_Key_LP.013" => Some(glutin::VirtualKeyCode::LControl),
        "Key_SUPER_Key_LP.014" => Some(glutin::VirtualKeyCode::LWin),
        "Key_ALT_Key_LP.015" => Some(glutin::VirtualKeyCode::LAlt),
        "Key_SPACE_Key_LP.003" => Some(glutin::VirtualKeyCode::Space),
        "Key_LEFT_SHIFT_Key_LP.004" => Some(glutin::VirtualKeyCode::LShift),
        "Key_CAPSLOCK_Key_LP.016" => Some(glutin::VirtualKeyCode::Capital),
        "Key_TAB_Key_LP.017" => Some(glutin::VirtualKeyCode::Tab),
        "Key_RSHIFT_Key_LP.005" => Some(glutin::VirtualKeyCode::RShift),
        "Key_ENTER_Key_LP.018" => Some(glutin::VirtualKeyCode::Return),
        "Key_\\_Key_LP.019" => Some(glutin::VirtualKeyCode::Backslash),
        "Key_BACKSPACE_Key_LP.020" => Some(glutin::VirtualKeyCode::Back),
        "Key_NUM_ENTER_Key_LP.021" => Some(glutin::VirtualKeyCode::NumpadEnter),
        "Key_NUM_ADD_Key_LP.006" => Some(glutin::VirtualKeyCode::Add),
        "Key_NUM_MIN_Key_LP.022" => Some(glutin::VirtualKeyCode::Subtract),
        "Key_NUM_0_Key_LP.007" => Some(glutin::VirtualKeyCode::Numpad0),
        "Key_NUM_DOT_Key_LP.023" => Some(glutin::VirtualKeyCode::NumpadComma),
        "Key_NUM_3_Key_LP.024" => Some(glutin::VirtualKeyCode::Numpad3),
        "Key_NUM_2_Key_LP.025" => Some(glutin::VirtualKeyCode::Numpad2),
        "Key_NUM_1_Key_LP.026" => Some(glutin::VirtualKeyCode::Numpad1),
        "Key_NUM_4_Key_LP.027" => Some(glutin::VirtualKeyCode::Numpad4),
        "Key_NUM_5_Key_LP.028" => Some(glutin::VirtualKeyCode::Numpad5),
        "Key_NUM_6_Key_LP.029" => Some(glutin::VirtualKeyCode::Numpad6),
        "Key_NUM_9_Key_LP.030" => Some(glutin::VirtualKeyCode::Numpad9),
        "Key_NUM_8_Key_LP.031" => Some(glutin::VirtualKeyCode::Numpad8),
        "Key_NUM_7_Key_LP.032" => Some(glutin::VirtualKeyCode::Numpad7),
        "Key_NUM_LCK_Key_LP.033" => Some(glutin::VirtualKeyCode::Numlock),
        "Key_NUM_DIV_Key_LP.034" => Some(glutin::VirtualKeyCode::Divide),
        "Key_NUM_MUL_Key_LP.035" => Some(glutin::VirtualKeyCode::Multiply),
        "Key_T4_Key_LP.036" => None, // TODO
        "Key_T3_Key_LP.037" => None, // TODO
        "Key_T2_Key_LP.038" => None, // TODO
        "Key_T1_Key_LP.039" => None, // TODO
        "Key_Up_Key_LP.040" => Some(glutin::VirtualKeyCode::Up),
        "Key_Left_Key_LP.041" => Some(glutin::VirtualKeyCode::Left),
        "Key_Down_Key_LP.042" => Some(glutin::VirtualKeyCode::Down),
        "Key_Left.001_Key_LP.043" => Some(glutin::VirtualKeyCode::Right), // FIXME: Inconsistent
        "Key_PGDN_Key_LP.044" => Some(glutin::VirtualKeyCode::PageDown),
        "Key_END_Key_LP.045" => Some(glutin::VirtualKeyCode::End),
        "Key_DEL_Key_LP.046" => Some(glutin::VirtualKeyCode::Delete),
        "Key_INS_Key_LP.047" => Some(glutin::VirtualKeyCode::Insert),
        "Key_Home_Key_LP.048" => Some(glutin::VirtualKeyCode::Home),
        "Key_PGUP_Key_LP.049" => Some(glutin::VirtualKeyCode::PageUp),
        "Key_PAUSE_Key_LP.050" => Some(glutin::VirtualKeyCode::Pause),
        "Key_SCRL_Key_LP.051" => Some(glutin::VirtualKeyCode::Scroll),
        "Key_PRNT_Key_LP.052" => Some(glutin::VirtualKeyCode::Snapshot),
        "Key_F12_Key_LP.053" => Some(glutin::VirtualKeyCode::F12),
        "Key_F12.001_Key_LP.054" => Some(glutin::VirtualKeyCode::F11),
        "Key_F10_Key_LP.055" => Some(glutin::VirtualKeyCode::F10),
        "Key_F9_Key_LP.056" => Some(glutin::VirtualKeyCode::F9),
        "Key_F8_Key_LP.057" => Some(glutin::VirtualKeyCode::F8),
        "Key_F7_Key_LP.058" => Some(glutin::VirtualKeyCode::F7),
        "Key_F6_Key_LP.059" => Some(glutin::VirtualKeyCode::F6),
        "Key_F5_Key_LP.060" => Some(glutin::VirtualKeyCode::F5),
        "Key_F4_Key_LP.061" => Some(glutin::VirtualKeyCode::F4),
        "Key_F3_Key_LP.062" => Some(glutin::VirtualKeyCode::F3),
        "Key_F2_Key_LP.063" => Some(glutin::VirtualKeyCode::F2),
        "Key_F1_Key_LP.064" => Some(glutin::VirtualKeyCode::F1),
        "Key_=_Key_LP.065" => Some(glutin::VirtualKeyCode::Equals),
        "Key_-_Key_LP.066" => Some(glutin::VirtualKeyCode::Minus),
        "Key_0_Key_LP.067" => Some(glutin::VirtualKeyCode::Key0),
        "Key_9_Key_LP.068" => Some(glutin::VirtualKeyCode::Key9),
        "Key_8_Key_LP.069" => Some(glutin::VirtualKeyCode::Key8),
        "Key_7_Key_LP.070" => Some(glutin::VirtualKeyCode::Key7),
        "Key_6_Key_LP.071" => Some(glutin::VirtualKeyCode::Key6),
        "Key_5_Key_LP.072" => Some(glutin::VirtualKeyCode::Key5),
        "Key_4_Key_LP.073" => Some(glutin::VirtualKeyCode::Key4),
        "Key_3_Key_LP.074" => Some(glutin::VirtualKeyCode::Key3),
        "Key_2_Key_LP.075" => Some(glutin::VirtualKeyCode::Key2),
        "Key_`_Key_LP.076" => Some(glutin::VirtualKeyCode::Apostrophe),
        "Key_1_Key_LP.077" => Some(glutin::VirtualKeyCode::Key1),
        "Key_/_Key_LP.002" => Some(glutin::VirtualKeyCode::Slash),
        "Key_._Key_LP.001" => Some(glutin::VirtualKeyCode::Period),
        "Key_,_Key_LP.078" => Some(glutin::VirtualKeyCode::Comma),
        "Key_M_Key_LP.079" => Some(glutin::VirtualKeyCode::M),
        "Key_N_Key_LP.080" => Some(glutin::VirtualKeyCode::N),
        "Key_B_Key_LP.081" => Some(glutin::VirtualKeyCode::B),
        "Key_V_Key_LP.082" => Some(glutin::VirtualKeyCode::V),
        "Key_C_Key_LP.083" => Some(glutin::VirtualKeyCode::C),
        "Key_X_Key_LP.084" => Some(glutin::VirtualKeyCode::X),
        "Key_Z_Key_LP.085" => Some(glutin::VirtualKeyCode::Z),
        "Key_'_Key_LP.086" => Some(glutin::VirtualKeyCode::Apostrophe),
        "Key_;_Key_LP.087" => Some(glutin::VirtualKeyCode::Semicolon),
        "Key_L_Key_LP.088" => Some(glutin::VirtualKeyCode::L),
        "Key_K_Key_LP.089" => Some(glutin::VirtualKeyCode::K),
        "Key_J_Key_LP.090" => Some(glutin::VirtualKeyCode::J),
        "Key_H_Key_LP.091" => Some(glutin::VirtualKeyCode::H),
        "Key_G_Key_LP.092" => Some(glutin::VirtualKeyCode::G),
        "Key_F_Key_LP.093" => Some(glutin::VirtualKeyCode::F),
        "Key_D_Key_LP.094" => Some(glutin::VirtualKeyCode::D),
        "Key_S_Key_LP.095" => Some(glutin::VirtualKeyCode::S),
        "Key_A_Key_LP.096" => Some(glutin::VirtualKeyCode::A),
        "Key_]_Key_LP.097" => Some(glutin::VirtualKeyCode::RBracket),
        "Key_[_Key_LP.098" => Some(glutin::VirtualKeyCode::LBracket),
        "Key_P_Key_LP.099" => Some(glutin::VirtualKeyCode::P),
        "Key_O_Key_LP.100" => Some(glutin::VirtualKeyCode::O),
        "Key_I_Key_LP.101" => Some(glutin::VirtualKeyCode::I),
        "Key_U_Key_LP.102" => Some(glutin::VirtualKeyCode::U),
        "Key_Y_Key_LP.103" => Some(glutin::VirtualKeyCode::Y),
        "Key_T_Key_LP.104" => Some(glutin::VirtualKeyCode::T),
        "Key_R_Key_LP.105" => Some(glutin::VirtualKeyCode::R),
        "Key_E_Key_LP.106" => Some(glutin::VirtualKeyCode::E),
        "Key_W_Key_LP.107" => Some(glutin::VirtualKeyCode::W),
        "Key_Q_Key_LP" => Some(glutin::VirtualKeyCode::Q),
        "Base_Cube.001" => None,
        _ => None, // Unknown model in obj file.
    };

    match code {
        Some(code) => keyboard_model::Index::from_code(code).into(),
        None => keyboard_model::Index::INVALID,
    }
}
