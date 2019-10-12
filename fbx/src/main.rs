use std::convert::TryInto;
use std::fs;
use std::io;
use std::path;

use fbx::*;

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
        if *count < 5 {
            visit(child, depth + 1)
        } else if *count == 5 {
            // First node thats skipped of this kind.
            println!("{}...", "  ".repeat(depth + 1));
        } else {
            // Skip this node.
        }
        *count += 1;
    }
}

#[derive(Debug)]
struct Objects {
    geometries: Vec<Geometry>,
}

fn parse_objects(node: &Node, stack: &mut Vec<String>) -> Objects {
    stack.push(node.name.clone());

    let mut geometries = Vec::new();

    for child in node.children.iter() {
        match child.name.as_str() {
            "Geometry" => {
                geometries.push(parse_geometry(child, stack));
            }
            other => {
                // Ignore.
            }
        }
    }

    stack.pop();

    Objects { geometries }
}

#[derive(Debug)]
struct Geometry {
    id: u64,
    name: String,
}

fn parse_geometry(node: &Node, stack: &mut Vec<String>) -> Geometry {
    stack.push(node.name.clone());

    let id = match node.properties.get(0) {
        Some(&Property::I64(id)) => id.try_into().unwrap(),
        _ => panic!("Geometry doesn't have id."),
    };

    let name = match node.properties.get(1) {
        Some(Property::String(name)) => name.clone(),
        _ => panic!("Geometry doesn't have name."),
    };

    stack.pop();

    Geometry { id, name }
}

fn main() {
    let file = read("resources/bistro/Bistro_Exterior.fbx").unwrap();
    dbg!(&file.header, file.children.len());

    let mut stack = &mut Vec::new();

    let mut objects = None;

    for child in file.children.iter() {
        match child.name.as_str() {
            "Objects" => {
                assert!(objects.is_none(), "Multiple \"Objects\" nodes.");
                objects = Some(parse_objects(child, stack));
            }
            other => {
                // Don't care.
            }
        }
        // visit(child, 0);
    }

    let objects = objects.expect("Missing \"Objects\" node.");

    dbg!(&objects);
}
