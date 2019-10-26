use fbx::dom::*;
use fbx::tree::{File, Node, Property};
use renderer::scene_file::*;
use std::fs;
use std::io;
use std::path;

fn read(path: impl AsRef<path::Path>) -> io::Result<File> {
    let mut reader = io::BufReader::new(fs::File::open(path)?);
    File::parse(&mut reader)
}

fn visit(node: &Node, depth: usize) {
    print!("{}{}", "  ".repeat(depth), node.name);
    for property in node.properties.iter() {
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

fn main() {
    let file = read("resources/sun_temple/SunTemple.fbx").unwrap();
    // let file = read("resources/bistro/Bistro_Exterior.fbx").unwrap();
    dbg!(&file.header, file.children.len());

    let stack = &mut Vec::<String>::new();

    let mut objects: Option<Objects> = None;
    let mut connections: Option<Connections> = None;
    let mut global_settings: Option<GlobalSettings> = None;

    for node in file.children.iter() {
        stack.push(node.name.to_string());

        // visit(node, 0);

        match node.name.as_str() {
            "GlobalSettings" => {
                assert!(global_settings.is_none());
                global_settings = Some(GlobalSettings::from_fbx(node, stack));
            }
            "Objects" => {
                assert!(objects.is_none());
                objects = Some(Objects::from_fbx(node, stack));
            }
            "Connections" => {
                assert!(connections.is_none());
                connections = Some(Connections::from_fbx(node, stack));
            }
            _ => {
                // Don't care.
            }
        }

        stack.pop();
    }

    let objects = objects.expect("Missing \"Objects\" node.");
    // dbg!(&objects.materials);
    // dbg!(objects.materials.len());
    // dbg!(objects.geometries.len());
    // dbg!(objects.textures);

    // dbg!(connections);
    // dbg!(global_settings.unwrap());

    let mut file = SceneFile {
        mesh_descriptions: Vec::new(),
        vertex_buffer: Vec::new(),
        triangle_buffer: Vec::new(),
        transforms: Vec::new(),
        transform_relations: Vec::new(),
        instances: Vec::new(),
    };

    for geometry in objects.geometries.iter() {
        println!(
            "Geometry {:?} with {} vertices, {} indices.",
            &geometry.name,
            geometry.vertices.len(),
            geometry.polygon_vertex_index.len()
        );

        // Deduplicate vertices.
        use std::collections::HashMap;

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

        println!(
            "Deduplicated {} vertices",
            (geometry.vertices.len() / 3) - vertices.len()
        );

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

        println!(
            "Triangulated {} indices to {} triangles",
            geometry.polygon_vertex_index.len(),
            triangles.len()
        );

        file.mesh_descriptions.push(MeshDescription {
            index_byte_offset: std::mem::size_of_val(&file.triangle_buffer[..]) as u64,
            vertex_offset: file.vertex_buffer.len() as u32,
            element_count: (triangles.len() * 3) as u32,
        });
        file.vertex_buffer.extend(vertices);
        file.triangle_buffer.extend(triangles);

        // dbg!(triangles);
        // break;
    }

    file.write(&mut std::io::BufWriter::new(std::fs::File::create("out.bin").unwrap()))
        .unwrap();

    let mut file = std::fs::File::open("out.bin").unwrap();
    let scene_file = SceneFile::read(&mut file).unwrap();

    dbg!(scene_file.mesh_descriptions);
    dbg!(scene_file.vertex_buffer.len());
    dbg!(scene_file.triangle_buffer.len());
}
