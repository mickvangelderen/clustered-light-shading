use cgmath::*;
use renderer::scene_file::*;
use std::fs;
use std::io;
use std::path;
use std::path::Path;

pub trait ModelExt {
    fn transform_to_parent(&self) -> Matrix4<f64>;
}

impl ModelExt for fbx::dom::Model {
    fn transform_to_parent(&self) -> Matrix4<f64> {
        let t = Matrix4::from_translation(Vector3::from(self.properties.lcl_translation));

        t
    }
}

use std::collections::HashMap;

fn read(path: impl AsRef<path::Path>) -> io::Result<fbx::tree::File> {
    let mut reader = io::BufReader::new(fs::File::open(path)?);
    fbx::tree::File::parse(&mut reader)
}

#[allow(unused)]
fn visit(node: &fbx::tree::Node, depth: usize) {
    print!("{}{}", "  ".repeat(depth), node.name);
    for property in node.properties.iter() {
        use fbx::tree::Property;
        match property {
            Property::Bool(value) => print!(" {}: bool,", value),
            Property::I16(value) => print!(" {}: i16,", value),
            Property::I32(value) => print!(" {}: i32,", value),
            Property::I64(value) => print!(" {}: i64,", value),
            Property::F32(value) => print!(" {}: f32,", value),
            Property::F64(value) => print!(" {}: f64,", value),
            Property::BoolArray(value) => print!(" [bool; {}]", value.len()),
            Property::I32Array(value) => print!(" [i32; {}]", value.len()),
            Property::I64Array(value) => print!(" [i64; {}]", value.len()),
            Property::F32Array(value) => print!(" [f32; {}]", value.len()),
            Property::F64Array(value) => print!(" [f64; {}]", value.len()),
            Property::String(value) => print!(" {:?}", value),
            Property::Bytes(value) => print!(" [u8; {}]", value.len()),
        };
    }
    println!();

    let mut visited = std::collections::HashMap::<String, usize>::new();

    for child in node.children.iter() {
        // visit(child, depth + 1);
        let count = visited.entry(child.name.clone()).or_default();
        if *count < 100000 || &child.name == "P" || &child.name == "Material" || &child.name == "Texture" {
            visit(child, depth + 1)
        } else if *count == 100000 {
            // First node thats skipped of this kind.
            println!("{}...", "  ".repeat(depth + 1));
        } else {
            // Skip this node.
        }
        *count += 1;
    }
}

fn convert(path: impl AsRef<Path>, out_path: impl AsRef<Path>) {
    let file = read(path).unwrap();
    dbg!(&file.header, file.children.len());

    let root = fbx::dom::Root::from_fbx_file(&file);

    let mut file = SceneFile {
        mesh_descriptions: Vec::new(),
        pos_in_obj_buffer: Vec::new(),
        nor_in_obj_buffer: Vec::new(),
        pos_in_tex_buffer: Vec::new(),
        triangle_buffer: Vec::new(),
        transforms: std::iter::once(renderer::scene_file::Transform {
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scaling: [1.0; 3],
        })
        .chain(root.objects.models.iter().map(|model| {
            assert_eq!(fbx::types::RotationOrder::XYZ, model.properties.rotation_order);
            assert_eq!([0.0; 3], model.properties.geometric_translation);
            assert_eq!([0.0; 3], model.properties.geometric_rotation);
            assert_eq!([1.0; 3], model.properties.geometric_scaling);

            fn rotation_matrix(xyz_deg: [f64; 3]) -> Matrix4<f64> {
                Matrix4::from(Euler {
                    x: Deg(xyz_deg[0]),
                    y: Deg(xyz_deg[1]),
                    z: Deg(xyz_deg[2]),
                })
            }

            fn translation_matrix(xyz: [f64; 3]) -> Matrix4<f64> {
                Matrix4::from_translation(xyz.into())
            }

            fn scaling_matrix(xyz: [f64; 3]) -> Matrix4<f64> {
                Matrix4::from_nonuniform_scale(xyz[0], xyz[1], xyz[2])
            }

            let p = &model.properties;

            let translation = translation_matrix(p.lcl_translation);
            let rotation = rotation_matrix(p.lcl_rotation);
            let scaling = scaling_matrix(p.lcl_scaling);

            let roff = translation_matrix(p.rotation_offset);
            let rpiv = translation_matrix(p.rotation_pivot);
            let rpre = rotation_matrix(p.pre_rotation);
            let rpost = rotation_matrix(p.post_rotation);

            let soff = translation_matrix(p.scaling_offset);
            let spiv = translation_matrix(p.scaling_pivot);

            let to_parent = translation
                * roff
                * rpiv
                * rpre
                * rotation
                * rpost.invert().unwrap()
                * rpiv.invert().unwrap()
                * soff
                * spiv
                * scaling
                * spiv.invert().unwrap();

            let t = to_parent.w.truncate();

            let s = [
                to_parent.x.truncate().magnitude(),
                to_parent.y.truncate().magnitude(),
                to_parent.z.truncate().magnitude(),
            ];

            let r = Euler::from(Quaternion::from(Matrix3::from_cols(
                to_parent.x.truncate() / s[0],
                to_parent.y.truncate() / s[1],
                to_parent.z.truncate() / s[2],
            )));

            renderer::scene_file::Transform {
                translation: [t[0] as f32, t[1] as f32, t[2] as f32],
                rotation: [
                    Deg::from(r.x).0 as f32,
                    Deg::from(r.y).0 as f32,
                    Deg::from(r.z).0 as f32,
                ],
                scaling: [s[0] as f32, s[1] as f32, s[2] as f32],
            }
        }))
        .collect(),
        transform_relations: Vec::new(),
        instances: Vec::new(),
        materials: root
            .objects
            .materials
            .iter()
            .map(|material| {
                fn cast_f64_3(a: [f64; 3]) -> [f32; 3] {
                    [a[0] as f32, a[1] as f32, a[2] as f32]
                }

                RawMaterial {
                    normal_texture_index: None,
                    emissive_color: cast_f64_3(material.properties.emissive_color),
                    emissive_texture_index: None,
                    ambient_color: cast_f64_3(material.properties.ambient_color),
                    ambient_texture_index: None,
                    diffuse_color: cast_f64_3(material.properties.diffuse_color),
                    diffuse_texture_index: None,
                    specular_color: cast_f64_3(material.properties.specular_color),
                    specular_texture_index: None,
                    shininess: material.properties.shininess as f32,
                    opacity: material.properties.opacity as f32,
                }
            })
            .collect(),
        textures: root
            .objects
            .textures
            .iter()
            .map(|texture| Texture {
                file_path: texture.file_path.clone(),
            })
            .collect(),
    };

    for geometry in root.objects.geometries.iter() {
        assert_eq!(0, geometry.vertices.len() % 3);

        // Initialize with best-guess capacity.
        let mut vertices = Vec::<Vertex>::with_capacity(geometry.vertices.len() / 3);
        let mut triangles = Vec::<[u32; 3]>::with_capacity(geometry.polygon_vertex_index.len() / 3);
        let mut vertex_map = HashMap::<Vertex, u32>::new();

        let mut count = 0;
        let mut triangle = [0u32; 3];
        let mut polygon_index = 0;

        // Go over all polygons.
        for (polygon_vertex_index, &vertex_index) in geometry.polygon_vertex_index.iter().enumerate() {
            let (vertex_index, should_reset) = if vertex_index < 0 {
                ((vertex_index ^ -1) as u32, true)
            } else {
                (vertex_index as u32, false)
            };

            // Load vertex.
            let pos_in_obj = [
                FiniteF32::new(geometry.vertices[vertex_index as usize * 3 + 0] as f32).unwrap(),
                FiniteF32::new(geometry.vertices[vertex_index as usize * 3 + 1] as f32).unwrap(),
                FiniteF32::new(geometry.vertices[vertex_index as usize * 3 + 2] as f32).unwrap(),
            ];

            use fbx::dom::AttributeMapping;

            let nor_in_obj = match geometry.layers[0].normals.as_ref() {
                Some(attribute) => {
                    let index = match attribute.mapping {
                        AttributeMapping::ByPolygon => polygon_index as usize,
                        AttributeMapping::ByVertex => vertex_index as usize,
                        AttributeMapping::ByPolygonVertex => polygon_vertex_index as usize,
                        _ => unimplemented!(),
                    };

                    let index = match attribute.indices.as_ref() {
                        Some(indices) => indices[index] as usize,
                        None => index,
                    };

                    [
                        FiniteF32::new(attribute.elements[index * 3 + 0] as f32).unwrap(),
                        FiniteF32::new(attribute.elements[index * 3 + 1] as f32).unwrap(),
                        FiniteF32::new(attribute.elements[index * 3 + 2] as f32).unwrap(),
                    ]
                }
                None => [
                    FiniteF32::new(0.0).unwrap(),
                    FiniteF32::new(0.0).unwrap(),
                    FiniteF32::new(0.0).unwrap(),
                ],
            };

            let pos_in_tex = match geometry.layers[0].uvs.as_ref() {
                Some(attribute) => {
                    let index = match attribute.mapping {
                        AttributeMapping::ByPolygon => polygon_index as usize,
                        AttributeMapping::ByVertex => vertex_index as usize,
                        AttributeMapping::ByPolygonVertex => polygon_vertex_index as usize,
                        _ => unimplemented!(),
                    };

                    let index = match attribute.indices.as_ref() {
                        Some(indices) => indices[index] as usize,
                        None => index,
                    };

                    [
                        FiniteF32::new(attribute.elements[index * 2 + 0] as f32).unwrap(),
                        FiniteF32::new(attribute.elements[index * 2 + 1] as f32).unwrap(),
                    ]
                }
                None => [FiniteF32::new(0.0).unwrap(), FiniteF32::new(0.0).unwrap()],
            };

            let vertex = Vertex {
                pos_in_obj,
                nor_in_obj,
                pos_in_tex,
            };

            let deduplicated_vertex_index = match vertex_map.get(&vertex) {
                Some(&index) => index,
                None => {
                    let index = vertices.len() as u32;
                    vertices.push(vertex);
                    vertex_map.insert(vertex, index);
                    index
                }
            };

            // Triangulate.
            if count < 3 {
                triangle[count] = deduplicated_vertex_index;
                count += 1;
            } else {
                triangle[1] = triangle[2];
                triangle[2] = deduplicated_vertex_index;
            }

            if count == 3 {
                triangles.push(triangle);
            }

            if should_reset {
                assert_eq!(3, count); // panic when we only have 0, 1 or 2 indices.
                count = 0;
                polygon_index += 1;
            }
        }

        file.mesh_descriptions.push(MeshDescription {
            index_byte_offset: std::mem::size_of_val(&file.triangle_buffer[..]) as u64,
            vertex_offset: file.pos_in_obj_buffer.len() as u32,
            element_count: (triangles.len() * 3) as u32,
        });

        file.pos_in_obj_buffer.extend(vertices.iter().map(|v| v.pos_in_obj));
        file.nor_in_obj_buffer.extend(vertices.iter().map(|v| v.nor_in_obj));
        file.pos_in_tex_buffer.extend(vertices.iter().map(|v| v.pos_in_tex));
        file.triangle_buffer.extend(triangles);
    }

    use fbx::dom::TypedIndex;

    for oo in root.connections.oo.iter() {
        match oo.0 {
            TypedIndex::Model(child_index) => {
                let parent_index = match oo.1 {
                    TypedIndex::Root => 0,
                    TypedIndex::Model(index) => index as u32 + 1,
                    _ => {
                        panic!("Didn't expect this relation {:?}", oo);
                    }
                };
                file.transform_relations.push(TransformRelation {
                    parent_index,
                    child_index: child_index as u32,
                });
            }
            TypedIndex::Geometry(geometry_index) => match oo.1 {
                TypedIndex::Model(model_index) => {
                    file.instances.push(Instance {
                        mesh_index: geometry_index as u32,
                        transform_index: model_index as u32 + 1,
                        material_index: None,
                    });
                }
                _ => {
                    panic!("Didn't expect this relation {:?}", oo);
                }
            },
            TypedIndex::Material(_) => match oo.1 {
                TypedIndex::Model(_) => {
                    // Will handle later.
                }
                _ => {
                    panic!("Didn't expect this relation {:?}", oo);
                }
            }
            _ => {
                println!("Unhandled connection {:?}", oo);
            }
        }
    }

    for oo in root.connections.oo.iter() {
        if let (TypedIndex::Material(material_index), TypedIndex::Model(model_index)) = *oo {
            let instance = file
                .instances
                .iter_mut()
                .find(|instance| instance.transform_index == model_index as u32 + 1)
                .unwrap();
            // FIXME: Handle multiple materials.
            // assert!(instance.material_index.is_none());
            instance.material_index = Some(NonMaxU32::new(material_index as u32).unwrap());
        }
    }

    for op in root.connections.op.iter() {
        match (op.0, op.1) {
            (TypedIndex::Texture(texture_index), TypedIndex::Material(material_index)) => {
                let material = &mut file.materials[material_index as usize];
                let texture_index = Some(NonMaxU32::new(texture_index as u32).unwrap());
                match op.2.as_ref() {
                    "DiffuseColor" => {
                        material.diffuse_texture_index = texture_index;
                    }
                    "NormalMap" => {
                        material.normal_texture_index = texture_index;
                    }
                    "SpecularColor" => {
                        material.specular_texture_index = texture_index;
                    }
                    "EmissiveColor" => {
                        material.emissive_texture_index = texture_index;
                    }
                    _ => {
                        eprintln!("Unhandled connection property {:?}", op);
                    }
                }
            }
            _ => {
                eprintln!("Unhandled connection {:?}", op);
            }
        }
    }

    file.write(&mut std::io::BufWriter::new(std::fs::File::create(&out_path).unwrap()))
        .unwrap();

    // let mut file = std::fs::File::open(&out_path).unwrap();
    // let scene_file = SceneFile::read(&mut file).unwrap();

    // dbg!(scene_file.mesh_descriptions);
    // dbg!(scene_file.vertex_buffer.len());
    // dbg!(scene_file.triangle_buffer.len());
    // dbg!(scene_file.materials);
    // dbg!(scene_file.textures);
}

fn main() {
    convert(
        "resources/bistro/Bistro_Exterior.fbx",
        "resources/bistro/Bistro_Exterior.bin",
    );
    convert(
        "resources/sun_temple/SunTemple.fbx",
        "resources/sun_temple/SunTemple.bin",
    );
}
