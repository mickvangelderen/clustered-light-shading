use cgmath::*;
use renderer::scene_file::*;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fs;
use std::io;
use std::path::Path;

pub trait ModelExt {
    fn transform_to_parent(&self) -> Matrix4<f64>;
}

impl ModelExt for fbx::dom::Model {
    fn transform_to_parent(&self) -> Matrix4<f64> {
        // NOTE(mickvangelderen): Others unsupported.
        assert_eq!(fbx::types::RotationOrder::XYZ, self.properties.rotation_order);

        // NOTE(mickvangelderen): Geometric transformations unsupported.
        assert_eq!([0.0; 3], self.properties.geometric_translation);
        assert_eq!([0.0; 3], self.properties.geometric_rotation);
        assert_eq!([1.0; 3], self.properties.geometric_scaling);

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

        let p = &self.properties;

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

        to_parent
    }
}

pub type Triangle = [u32; 3];

#[derive(Default)]
pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    vertex_to_index: HashMap<Vertex, u32>,
    triangles: Vec<Triangle>,
}

impl MeshBuilder {
    pub fn insert_vertex(&mut self, vertex: Vertex) -> u32 {
        match self.vertex_to_index.get(&vertex) {
            Some(&index) => index,
            None => {
                let index = u32::try_from(self.vertices.len()).unwrap();
                self.vertices.push(vertex);
                self.vertex_to_index.insert(vertex, index);
                index
            }
        }
    }

    pub fn push_triangle(&mut self, triangle: Triangle) {
        self.triangles.push(triangle);
    }
}

fn read(path: impl AsRef<Path>) -> io::Result<fbx::tree::File> {
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
        bin_in_obj_buffer: Vec::new(),
        tan_in_obj_buffer: Vec::new(),
        pos_in_tex_buffer: Vec::new(),
        triangle_buffer: Vec::new(),
        transforms: std::iter::once(renderer::scene_file::Transform {
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scaling: [1.0; 3],
        })
        .chain(root.objects.models.iter().map(|model| {
            let to_parent = model.transform_to_parent();

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

    let mut geometry_index_to_mesh_indices: Vec<Vec<u32>> = Vec::new();

    for geometry in root.objects.geometries.iter() {
        assert_eq!(0, geometry.vertices.len() % 3);

        let mut mesh_builders: Vec<MeshBuilder> = Vec::new();

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

            let polygon_vertex_indices = fbx::dom::PolygonVertexIndices {
                polygon_index,
                vertex_index: vertex_index as usize,
                polygon_vertex_index,
            };

            let nor_in_obj = match geometry.layers[0].normals.as_ref() {
                Some(attribute) => {
                    let index = attribute.select_polygon_vertex_index(&polygon_vertex_indices);

                    [
                        FiniteF32::new(attribute.elements[index * 3 + 0] as f32).unwrap(),
                        FiniteF32::new(attribute.elements[index * 3 + 1] as f32).unwrap(),
                        FiniteF32::new(attribute.elements[index * 3 + 2] as f32).unwrap(),
                    ]
                }
                None => Default::default(),
            };

            let bin_in_obj = match geometry.layers[0].binormals.as_ref() {
                Some(attribute) => {
                    let index = attribute.select_polygon_vertex_index(&polygon_vertex_indices);

                    [
                        FiniteF32::new(attribute.elements[index * 3 + 0] as f32).unwrap(),
                        FiniteF32::new(attribute.elements[index * 3 + 1] as f32).unwrap(),
                        FiniteF32::new(attribute.elements[index * 3 + 2] as f32).unwrap(),
                    ]
                }
                None => Default::default(),
            };

            let tan_in_obj = match geometry.layers[0].tangents.as_ref() {
                Some(attribute) => {
                    let index = attribute.select_polygon_vertex_index(&polygon_vertex_indices);

                    [
                        FiniteF32::new(attribute.elements[index * 3 + 0] as f32).unwrap(),
                        FiniteF32::new(attribute.elements[index * 3 + 1] as f32).unwrap(),
                        FiniteF32::new(attribute.elements[index * 3 + 2] as f32).unwrap(),
                    ]
                }
                None => Default::default(),
            };

            let pos_in_tex = match geometry.layers[0].uvs.as_ref() {
                Some(attribute) => {
                    let index = attribute.select_polygon_vertex_index(&polygon_vertex_indices);

                    [
                        FiniteF32::new(attribute.elements[index * 2 + 0] as f32).unwrap(),
                        FiniteF32::new(attribute.elements[index * 2 + 1] as f32).unwrap(),
                    ]
                }
                None => Default::default(),
            };

            let vertex = Vertex {
                pos_in_obj,
                nor_in_obj,
                bin_in_obj,
                tan_in_obj,
                pos_in_tex,
            };

            let material_layer: u32 = match geometry.layers[0].materials.as_ref() {
                Some(attribute) => {
                    let index = match attribute.mapping {
                        AttributeMapping::ByPolygon => polygon_index as usize,
                        AttributeMapping::ByVertex => vertex_index as usize,
                        AttributeMapping::ByPolygonVertex => polygon_vertex_index as usize,
                        AttributeMapping::AllSame => 0,
                        _ => unimplemented!(),
                    };

                    let index = match attribute.indices.as_ref() {
                        Some(indices) => indices[index] as usize,
                        None => index,
                    };

                    attribute.elements[index].try_into().unwrap()
                }
                None => panic!("No material assigned"),
            };

            assert!(
                material_layer < 16,
                "Hit artificial limit of 16 material layers per geometry"
            );

            while mesh_builders.len() <= material_layer as usize {
                mesh_builders.push(MeshBuilder::default());
            }

            let mesh_builder = &mut mesh_builders[material_layer as usize];

            let vertex_index = mesh_builder.insert_vertex(vertex);

            // Triangulate.
            if count < 3 {
                triangle[count] = vertex_index;
                count += 1;
            } else {
                triangle[1] = triangle[2];
                triangle[2] = vertex_index;
            }

            if count == 3 {
                mesh_builder.push_triangle(triangle);
            }

            if should_reset {
                assert_eq!(3, count); // panic when we only have 0, 1 or 2 indices.
                count = 0;
                polygon_index += 1;
            }
        }

        println!(
            "Geometry {} has {} material layers and {} geometry layers",
            geometry.id,
            mesh_builders.len(),
            geometry.layers.len()
        );

        let mut mesh_indices = Vec::new();

        for mesh_builder in mesh_builders {
            let mesh_index = file.mesh_descriptions.len() as u32;
            mesh_indices.push(mesh_index);

            file.mesh_descriptions.push(MeshDescription {
                triangle_offset: file.triangle_buffer.len() as u32,
                triangle_count: mesh_builder.triangles.len() as u32,
                vertex_offset: file.pos_in_obj_buffer.len() as u32,
                vertex_count: mesh_builder.vertices.len() as u32,
            });

            file.pos_in_obj_buffer
                .extend(mesh_builder.vertices.iter().map(|v| v.pos_in_obj));
            file.nor_in_obj_buffer
                .extend(mesh_builder.vertices.iter().map(|v| v.nor_in_obj));
            file.bin_in_obj_buffer
                .extend(mesh_builder.vertices.iter().map(|v| v.bin_in_obj));
            file.tan_in_obj_buffer
                .extend(mesh_builder.vertices.iter().map(|v| v.tan_in_obj));
            file.pos_in_tex_buffer
                .extend(mesh_builder.vertices.iter().map(|v| v.pos_in_tex));
            file.triangle_buffer.extend(mesh_builder.triangles);
        }

        geometry_index_to_mesh_indices.push(mesh_indices);
    }

    use fbx::dom::TypedIndex;

    let mut model_index_to_incomplete_instances: Vec<(Vec<IncompleteInstance>, usize)> = Vec::new();

    for &oo in root.connections.oo.iter() {
        match oo {
            (TypedIndex::Model(child_index), parent) => {
                let parent_index = match parent {
                    TypedIndex::Root => 0,
                    TypedIndex::Model(index) => u32::try_from(index + 1).unwrap(),
                    _ => {
                        panic!("Didn't expect this relation {:?}", oo);
                    }
                };
                file.transform_relations.push(TransformRelation {
                    parent_index,
                    child_index: child_index as u32,
                });
            }
            (TypedIndex::Geometry(geometry_index), TypedIndex::Model(model_index)) => {
                while model_index_to_incomplete_instances.len() <= model_index as usize {
                    model_index_to_incomplete_instances.push((Default::default(), 0));
                }
                let (ref mut instances, _) = model_index_to_incomplete_instances[model_index];

                for &mesh_index in geometry_index_to_mesh_indices[usize::try_from(geometry_index).unwrap()].iter() {
                    instances.push(IncompleteInstance {
                        mesh_index,
                        transform_index: model_index as u32 + 1,
                        material_index: None,
                    });
                }
            }
            (TypedIndex::Material(_), TypedIndex::Model(_)) => {
                // Will handle later.
            }
            _ => {
                println!("Unhandled connection {:?}", oo);
            }
        }
    }

    for &oo in root.connections.oo.iter() {
        match oo {
            (TypedIndex::Material(material_index), TypedIndex::Model(model_index)) => {
                let (ref mut instances, ref mut counter) = model_index_to_incomplete_instances[model_index as usize];
                instances[*counter].material_index =
                    Some(NonMaxU32::new(u32::try_from(material_index).unwrap()).unwrap());
                *counter += 1;
            }
            _ => {
                // Don't care.
            }
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

    file.instances.extend(
        model_index_to_incomplete_instances
            .into_iter()
            .flat_map(|(instances, _)| {
                instances.into_iter().map(|instance| Instance {
                    mesh_index: instance.mesh_index,
                    transform_index: instance.transform_index,
                    material_index: instance.material_index.unwrap().get(),
                })
            }),
    );

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
    let resource_dir = std::path::PathBuf::from("resources");
    for p in &[
        // "speedtree/Azalea/LowPoly/Azalea_LowPoly.fbx",
        // "speedtree/Azalea/HighPoly/Azalea.fbx",
        // "speedtree/Red Maple Young/LowPoly/Red_Maple_Young_LowPoly.fbx",
        // "speedtree/Red Maple Young/HighPoly/Red_Maple_Young.fbx",
        // "speedtree/Hedge/LowPoly/Hedge_LowPoly.fbx",
        // "speedtree/Hedge/HighPoly/Hedge.fbx",
        // "speedtree/Boston Fern/LowPoly/Boston_Fern_LowPoly.fbx",
        // "speedtree/Boston Fern/HighPoly/Boston_Fern.fbx",
        // "speedtree/Backyard Grass/LowPoly/Backyard_Grass_LowPoly.fbx",
        // "speedtree/Backyard Grass/HighPoly/Backyard_Grass.fbx",
        // "speedtree/European Linden/LowPoly/European_Linden_LowPoly.fbx",
        // "speedtree/European Linden/HighPoly/European_Linden.fbx",
        // "speedtree/Japanese Maple/LowPoly/Japanese_Maple_LowPoly.fbx",
        // "speedtree/Japanese Maple/HighPoly/Japanese_Maple.fbx",
        // "speedtree/White Oak/LowPoly/White_Oak_LowPoly.fbx",
        // "speedtree/White Oak/HighPoly/White_Oak.fbx",
        "emerald_square/Block_Sheleg_Tower.fbx",
        "emerald_square/Block_Park.fbx",
        "emerald_square/Block_Toy_Hotel.fbx",
        "emerald_square/End_Cap_Corner.fbx",
        "emerald_square/Block_KWOW_Coffee.fbx",
        "emerald_square/End_Cap.fbx",
        "bistro/Bistro_Interior_Binary.fbx",
        // "bistro/Bistro_Interior.fbx",
        "bistro/Bistro_Exterior.fbx",
        "sun_temple/SunTemple.fbx",
    ] {
        let i = resource_dir.join(p);
        let o = i.with_extension("bin");
        dbg!(&i, &o);
        convert(i, o);
    }
}
