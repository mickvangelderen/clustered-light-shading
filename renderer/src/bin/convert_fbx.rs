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
        vertex_buffer: Vec::new(),
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
    };

    for geometry in root.objects.geometries.iter() {
        // println!(
        //     "Geometry {:?} with {} vertices, {} indices.",
        //     &geometry.name,
        //     geometry.vertices.len(),
        //     geometry.polygon_vertex_index.len()
        // );

        // Deduplicate vertices.
        assert_eq!(0, geometry.vertices.len() % 3);

        let mut vertex_map: HashMap<Vertex, u32> = HashMap::new();
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut index_map: Vec<u32> = Vec::with_capacity(geometry.vertices.len() / 3);

        for chunk in geometry.vertices.chunks_exact(3) {
            let vertex = Vertex {
                pos_in_obj: [
                    FiniteF32::new(chunk[0] as f32).unwrap(),
                    FiniteF32::new(chunk[1] as f32).unwrap(),
                    FiniteF32::new(chunk[2] as f32).unwrap(),
                ],
            };

            let index = vertex_map.len() as u32;
            let index = *vertex_map.entry(vertex).or_insert_with(|| {
                vertices.push(vertex);
                index
            });
            index_map.push(index);
        }

        assert_eq!(geometry.vertices.len() / 3, index_map.len());
        assert_eq!(vertices.len(), vertex_map.len());

        // println!(
        //     "Deduplicated {} vertices",
        //     (geometry.vertices.len() / 3) - vertices.len()
        // );

        let mut triangles = Vec::<[u32; 3]>::new();

        let mut count = 0;
        let mut triangle = [0u32; 3];

        // Convert ngon indices to triangles.
        for &index in geometry.polygon_vertex_index.iter() {
            let (index, should_reset) = if index < 0 {
                ((index ^ -1) as u32, true)
            } else {
                (index as u32, false)
            };

            let index = index_map[index as usize];

            if count < 3 {
                triangle[count] = index;
                count += 1;
            } else {
                triangle[1] = triangle[2];
                triangle[2] = index;
            }

            if count == 3 {
                triangles.push(triangle);
            }

            if should_reset {
                assert_eq!(3, count); // panic when we only have 0, 1 or 2 indices.
                count = 0;
            }
        }

        // println!(
        //     "Triangulated {} indices to {} triangles",
        //     geometry.polygon_vertex_index.len(),
        //     triangles.len()
        // );

        file.mesh_descriptions.push(MeshDescription {
            index_byte_offset: std::mem::size_of_val(&file.triangle_buffer[..]) as u64,
            vertex_offset: file.vertex_buffer.len() as u32,
            element_count: (triangles.len() * 3) as u32,
        });
        file.vertex_buffer.extend(vertices);
        file.triangle_buffer.extend(triangles);
    }

    for oo in root.connections.oo {
        use fbx::dom::TypedIndex;

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
                    });
                }
                _ => {
                    panic!("Didn't expect this relation {:?}", oo);
                }
            },
            _ => {
                println!("Unhandled connection {:?}", oo);
            }
        }
    }

    // for op in root.connections.op {
    //     println!("{:?} {:?} {:?}", op.0, op.1, op.2);
    // }

    // for model in root.objects.models {
    //     println!("{:#?}", model);
    // }

    // println!("{:#?}", file.transforms);

    file.write(&mut std::io::BufWriter::new(std::fs::File::create(out_path).unwrap()))
        .unwrap();

    // let mut file = std::fs::File::open("out.bin").unwrap();
    // let scene_file = SceneFile::read(&mut file).unwrap();

    // dbg!(scene_file.mesh_descriptions);
    // dbg!(scene_file.vertex_buffer.len());
    // dbg!(scene_file.triangle_buffer.len());
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
