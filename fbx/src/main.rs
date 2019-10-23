use std::fs;
use std::io;
use std::path;

mod objects;
mod connections;
mod global_settings;

use objects::*;
use connections::*;
use global_settings::*;

use fbx::*;

fn panic_wrong_property_kind() -> ! {
    panic!("Wrong property kind");
}

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

#[repr(C)]
pub struct OpaqueDepthVertex {
    pub pos_in_obj: [f32; 3],
}

#[repr(C)]
pub struct MaskedDepthVertex {
    pub pos_in_obj: [f32; 3],
    pub pos_in_tex: [f32; 2],
}

#[repr(C)]
pub struct FullVertex {
    pub pos_in_obj: [f32; 3],
    pub nor_in_obj: [f32; 3],
    pub pos_in_tex: [f32; 2],
}

fn main() {
    // let file = read("resources/sun_temple/SunTemple.fbx").unwrap();
    let file = read("resources/bistro/Bistro_Exterior.fbx").unwrap();
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
    dbg!(global_settings.unwrap());

    for geometry in objects.geometries.iter() {
        // dbg!(&geometry.name);
    }
}
