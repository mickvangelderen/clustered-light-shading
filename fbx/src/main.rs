use std::fs;
use std::io;
use std::path;

use fbx::*;

fn read(path: impl AsRef<path::Path>) -> io::Result<File> {
    let mut reader = io::BufReader::new(fs::File::open(path)?);
    File::parse(&mut reader)
}

fn visit(node: &Node, depth: usize) {
    print!("{}ENTERING {}", "  ".repeat(depth), node.name);
    for property in node.properties.iter() {
        match property {
            Property::Bool(value) => print!(" {},", value),
            Property::I16(value) => print!(" {},", value),
            Property::I32(value) => print!(" {},", value),
            Property::I64(value) => print!(" {},", value),
            Property::F32(value) => print!(" {},", value),
            Property::F64(value) => print!(" {},", value),
            Property::BoolArray(value) => print!(" [bool; {}]", value.len()),
            Property::I32Array(value) => print!(" [i32; {}]", value.len()),
            Property::I64Array(value) => print!(" [i64; {}]", value.len()),
            Property::F32Array(value) => print!(" [f32; {}]", value.len()),
            Property::F64Array(value) => print!(" [f64; {}]", value.len()),
            Property::String(value) => print!(" {}", value),
            Property::Bytes(value) => print!(" [u8; {}]", value.len()),
        };
    }
    println!();

    for child in node.children.iter() {
        visit(child, depth + 1);
    }

    println!("{}LEAVING  {}", "  ".repeat(depth), node.name);
}

fn main() {
    let file = read("resources/bistro/Bistro_Exterior.fbx").unwrap();
    dbg!(&file.header, file.children.len());
    for child in file.children.iter() {
        visit(child, 0);
    }
}
