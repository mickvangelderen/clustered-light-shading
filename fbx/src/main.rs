use std::convert::TryInto;
use std::fs;
use std::io;
use std::path;

use fbx::*;

mod model;
mod texture;
mod geometry;
mod material;

use model::*;
use texture::*;
use geometry::*;
use material::*;

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

#[derive(Debug)]
struct Objects {
    geometries: Vec<Geometry>,
    materials: Vec<Material>,
    models: Vec<Model>,
    textures: Vec<Texture>,
}

fn parse_objects(node: &Node, stack: &mut Vec<String>) -> Objects {
    stack.push(node.name.clone());

    let mut geometries = Vec::new();
    let mut materials = Vec::new();
    let mut models = Vec::new();
    let mut textures = Vec::new();

    for child in node.children.iter() {
        match child.name.as_str() {
            "Geometry" => {
                geometries.push(Geometry::from_fbx(child, stack));
            }
            "Material" => {
                materials.push(Material::from_fbx(child, stack));
            }
            "Model" => {
                models.push(Model::from_fbx(child, stack));
            }
            "Texture" => {
                textures.push(Texture::from_fbx(child, stack));
            }
            _ => {
                // Ignore.
            }
        }
    }

    stack.pop();

    Objects { geometries, materials, models, textures }
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

    for child in file.children.iter() {
        // visit(child, 0);

        match child.name.as_str() {
            "Objects" => {
                assert!(objects.is_none(), "Multiple \"Objects\" nodes.");
                objects = Some(parse_objects(child, stack));
            }
            _ => {
                // Don't care.
            }
        }
    }

    let objects = objects.expect("Missing \"Objects\" node.");
    // dbg!(&objects.materials);
    // dbg!(objects.materials.len());
    // dbg!(objects.geometries.len());
    dbg!(objects.textures);

    for geometry in objects.geometries.iter() {
        // dbg!(&geometry.name);
    }
}
