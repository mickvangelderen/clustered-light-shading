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
        if *count < 10 || &child.name == "P" || &child.name == "Material" || &child.name == "Texture" {
            visit(child, depth + 1)
        } else if *count == 10 {
            // First node thats skipped of this kind.
            println!("{}...", "  ".repeat(depth + 1));
        } else {
            // Skip this node.
        }
        *count += 1;
    }
}

#[derive(Debug)]
struct Material {
    id: u64,
    name: String,
    properties: MaterialProperties,
}

#[derive(Debug)]
struct MaterialProperties {
    transparent_color: [f64; 3],
    transparent_factor: f64,

    emissive_color: [f64; 3],
    emissive_factor: f64,

    ambient_color: [f64; 3],
    ambient_factor: f64,

    diffuse_color: [f64; 3],
    diffuse_factor: f64,

    specular_color: [f64; 3],
    specular_factor: f64,

    shininess: f64,
    opacity: f64,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            transparent_color: [0.0; 3],
            transparent_factor: 0.0,

            emissive_color: [0.0; 3],
            emissive_factor: 1.0,

            ambient_color: [0.2; 3],
            ambient_factor: 1.0,

            diffuse_color: [0.8; 3],
            diffuse_factor: 1.0,

            specular_color: [0.2; 3],
            specular_factor: 1.0,

            shininess: 20.0,
            opacity: 1.0,
        }
    }
}

#[derive(Debug)]
struct Objects {
    geometries: Vec<Geometry>,
    materials: Vec<Material>,
}

fn parse_objects(node: &Node, stack: &mut Vec<String>) -> Objects {
    stack.push(node.name.clone());

    let mut geometries = Vec::new();
    let mut materials = Vec::new();

    for child in node.children.iter() {
        match child.name.as_str() {
            "Geometry" => {
                geometries.push(parse_geometry(child, stack));
            }
            "Material" => {
                materials.push(parse_material(child, stack));
            }
            _ => {
                // Ignore.
            }
        }
    }

    stack.pop();

    Objects { geometries, materials }
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

#[derive(Debug)]
pub struct Geometry {
    pub id: u64,
    pub name: String,
    pub vertices: Vec<f64>,
    pub polygon_vertex_index: Vec<i32>,
    pub edges: Option<Vec<i32>>,
    pub layers: Vec<GeometryLayer>,
}

#[derive(Debug, Default)]
pub struct GeometryLayer {
    pub normals: Option<Attribute<f64, i32>>,
    pub binormals: Option<Attribute<f64, i32>>,
    pub tangents: Option<Attribute<f64, i32>>,
    pub uvs: Option<Attribute<f64, i32>>,
    pub materials: Option<Attribute<i32, i32>>,
}

#[derive(Debug)]
pub enum AttributeMapping {
    ByPolygon,
    ByVertex,
    ByPolygonVertex,
    ByEdge,
    AllSame,
}

impl AttributeMapping {
    pub fn from_str(s: &str) -> Self {
        match s {
            "ByPolygon" => Self::ByPolygon,
            "ByPolygonVertex" => Self::ByPolygonVertex,
            "ByVertex" | "ByVertice" => Self::ByVertex,
            "ByEdge" => Self::ByEdge,
            "AllSame" => Self::AllSame,
            other => panic!("Unknown MappingInformationType: {:?}", other),
        }
    }
}

enum ReferenceInformationType {
    Direct,
    IndexToDirect,
}

impl ReferenceInformationType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Direct" => Self::Direct,
            "IndexToDirect" | "Index" => Self::IndexToDirect,
            other => {
                panic!("Unknown ReferenceInformationType: {:?}", other);
            }
        }
    }
}

#[derive(Debug)]
pub struct Attribute<E, I> {
    elements: Vec<E>,
    indices: Option<Vec<I>>,
    mapping: AttributeMapping,
}

fn panic_wrong_property_kind() -> ! {
    panic!("Wrong property kind");
}

fn parse_material(node: &Node, stack: &mut Vec<String>) -> Material {
    stack.push(node.name.clone());

    let id = node.properties[0].to_i64_exact() as u64;

    let name = {
        let name = node.properties[1].as_str();
        let postfix = "\u{0}\u{1}Material";
        assert!(name.ends_with(postfix));
        String::from(&name[0..name.len() - postfix.len()])
    };

    assert_eq!("", node.properties[2].as_str());

    let mut properties = MaterialProperties::default();

    for node in node.children.iter() {
        stack.push(node.name.clone());
        match node.name.as_str() {
            "Version" => {
                // Don't care.
            }
            "ShadingModel" => {
                assert_eq!("phong", node.properties[0].as_str());
            }
            "MultiLayer" => {
                assert_eq!(0, node.properties[0].to_i32_exact());
            }
            "Properties70" => {
                for node in node.children.iter() {
                    stack.push(node.name.clone());

                    assert_eq!(node.name.as_str(), "P");

                    match node.properties[0].as_str() {
                        "Transparent" | "TransparentColor" => {
                            properties.transparent_color = [
                                node.properties[4].to_f64_exact(),
                                node.properties[5].to_f64_exact(),
                                node.properties[6].to_f64_exact(),
                            ];
                        }
                        "TransparencyFactor" | "TransparentFactor" => {
                            properties.transparent_factor = node.properties[4].to_f64_exact();
                        }
                        "Emissive" | "EmissiveColor" => {
                            properties.emissive_color = [
                                node.properties[4].to_f64_exact(),
                                node.properties[5].to_f64_exact(),
                                node.properties[6].to_f64_exact(),
                            ];
                        }
                        "EmissiveFactor" => {
                            properties.emissive_factor = node.properties[4].to_f64_exact();
                        }
                        "Ambient" | "AmbientColor" => {
                            properties.ambient_color = [
                                node.properties[4].to_f64_exact(),
                                node.properties[5].to_f64_exact(),
                                node.properties[6].to_f64_exact(),
                            ];
                        }
                        "AmbientFactor" => {
                            properties.ambient_factor = node.properties[4].to_f64_exact();
                        }
                        "Diffuse" | "DiffuseColor" => {
                            properties.diffuse_color = [
                                node.properties[4].to_f64_exact(),
                                node.properties[5].to_f64_exact(),
                                node.properties[6].to_f64_exact(),
                            ];
                        }
                        "Specular" | "SpecularColor" => {
                            properties.specular_color = [
                                node.properties[4].to_f64_exact(),
                                node.properties[5].to_f64_exact(),
                                node.properties[6].to_f64_exact(),
                            ];
                        }
                        "Shininess" | "ShininessExponent" => {
                            properties.shininess = node.properties[4].to_f64_exact();
                        }
                        "Opacity" => {
                            properties.opacity = node.properties[4].to_f64_exact();
                        }
                        "Bump"
                        | "NormalMap"
                        | "BumpFactor"
                        | "ReflectionColor"
                        | "ReflectionFactor"
                        | "Reflectivity"
                        | "DisplacementColor"
                        | "DisplacementFactor"
                        | "VectorDisplacementColor"
                        | "VectorDisplacementFactor" => {
                            // Don't care.
                        }
                        unknown => {
                            panic!("Unknown material property: {:?}", unknown);
                        }
                    }

                    stack.pop();
                }
            }
            unknown => {
                panic!("Unknown material property: {:?}", unknown);
            }
        }
        stack.pop();
    }

    stack.pop();

    Material { id, name, properties }
}

fn parse_geometry(node: &Node, stack: &mut Vec<String>) -> Geometry {
    stack.push(node.name.clone());

    let id = node.properties[0].to_i64_exact() as u64;

    let name = {
        let name = node.properties[1].as_str();
        let postfix = "\u{0}\u{1}Geometry";
        assert!(name.ends_with(postfix));
        String::from(&name[0..name.len() - postfix.len()])
    };

    assert_eq!("Mesh", node.properties[2].as_str());

    let mut vertices = None;
    let mut polygon_vertex_index: Option<Vec<i32>> = None;
    let mut edges: Option<Vec<i32>> = None;
    let mut layers = Vec::<GeometryLayer>::new();

    for node in node.children.iter() {
        stack.push(node.name.clone());
        match node.name.as_str() {
            "GeometryVersion" => {
                // Don't care.
            }
            "Layer" => {
                // NOTE(mickvangelderen): Just going to not deal with this.
            }
            "Vertices" => {
                assert!(vertices.is_none());
                vertices = Some(node.properties[0].as_f64_array_exact().to_vec());
            }
            "PolygonVertexIndex" => {
                assert!(polygon_vertex_index.is_none());
                polygon_vertex_index = Some(node.properties[0].as_i32_array_exact().to_vec());
            }
            "Edges" => {
                assert!(edges.is_none());
                edges = Some(node.properties[0].as_i32_array_exact().to_vec());
            }
            "LayerElementNormal" => {
                let layer_index = node.properties[0].to_i32_exact() as usize;
                while layers.len() < layer_index + 1 {
                    layers.push(GeometryLayer::default());
                }
                let layer = &mut layers[layer_index];

                let mut mapping = None;
                let mut reference = None;
                let mut elements = None;
                let mut indices = None;

                for node in node.children.iter() {
                    stack.push(node.name.clone());
                    match node.name.as_str() {
                        "Version" => {
                            // Don't care.
                        }
                        "Name" => {
                            // Don't care.
                        }
                        "MappingInformationType" => {
                            assert!(mapping.is_none());
                            mapping = Some(AttributeMapping::from_str(node.properties[0].as_str()));
                        }
                        "ReferenceInformationType" => {
                            assert!(reference.is_none());
                            reference = Some(ReferenceInformationType::from_str(node.properties[0].as_str()));
                        }
                        "Normals" => {
                            assert!(elements.is_none());
                            elements = Some(node.properties[0].as_f64_array_exact().to_vec());
                        }
                        "NormalsIndex" => {
                            assert!(indices.is_none());
                            indices = Some(node.properties[0].as_i32_array_exact().to_vec());
                        }
                        other => {
                            panic!("Unexpected layer elements property {:?}", other);
                        }
                    }
                    stack.pop();
                }

                let mapping = mapping.unwrap();

                assert!(layer.normals.is_none());
                layer.normals = Some(Attribute {
                    elements: elements.unwrap(),
                    indices: match mapping {
                        AttributeMapping::AllSame => {
                            // Deal with all same not having indices but ref is sometimes index to direct...
                            None
                        }
                        _ => match reference.unwrap() {
                            ReferenceInformationType::Direct => {
                                assert!(indices.is_none());
                                None
                            }
                            ReferenceInformationType::IndexToDirect => Some(indices.unwrap()),
                        },
                    },
                    mapping,
                })
            }
            "LayerElementBinormal" => {
                let layer_index = node.properties[0].to_i32_exact() as usize;
                while layers.len() < layer_index + 1 {
                    layers.push(GeometryLayer::default());
                }
                let layer = &mut layers[layer_index];

                let mut mapping = None;
                let mut reference = None;
                let mut elements = None;
                let mut indices = None;

                for node in node.children.iter() {
                    stack.push(node.name.clone());
                    match node.name.as_str() {
                        "Version" => {
                            // Don't care.
                        }
                        "Name" => {
                            // Don't care.
                        }
                        "MappingInformationType" => {
                            assert!(mapping.is_none());
                            mapping = Some(AttributeMapping::from_str(node.properties[0].as_str()));
                        }
                        "ReferenceInformationType" => {
                            assert!(reference.is_none());
                            reference = Some(ReferenceInformationType::from_str(node.properties[0].as_str()));
                        }
                        "Binormals" => {
                            assert!(elements.is_none());
                            elements = Some(node.properties[0].as_f64_array_exact().to_vec());
                        }
                        "BinormalsIndex" => {
                            assert!(indices.is_none());
                            indices = Some(node.properties[0].as_i32_array_exact().to_vec());
                        }
                        other => {
                            panic!("Unexpected layer elements property {:?}", other);
                        }
                    }
                    stack.pop();
                }

                let mapping = mapping.unwrap();

                assert!(layer.binormals.is_none());
                layer.binormals = Some(Attribute {
                    elements: elements.unwrap(),
                    indices: match mapping {
                        AttributeMapping::AllSame => {
                            // Deal with all same not having indices but ref is sometimes index to direct...
                            None
                        }
                        _ => match reference.unwrap() {
                            ReferenceInformationType::Direct => {
                                assert!(indices.is_none());
                                None
                            }
                            ReferenceInformationType::IndexToDirect => Some(indices.unwrap()),
                        },
                    },
                    mapping,
                })
            }
            "LayerElementTangent" => {
                let layer_index = node.properties[0].to_i32_exact() as usize;
                while layers.len() < layer_index + 1 {
                    layers.push(GeometryLayer::default());
                }
                let layer = &mut layers[layer_index];

                let mut mapping = None;
                let mut reference = None;
                let mut elements = None;
                let mut indices = None;

                for node in node.children.iter() {
                    stack.push(node.name.clone());
                    match node.name.as_str() {
                        "Version" => {
                            // Don't care.
                        }
                        "Name" => {
                            // Don't care.
                        }
                        "MappingInformationType" => {
                            assert!(mapping.is_none());
                            mapping = Some(AttributeMapping::from_str(node.properties[0].as_str()));
                        }
                        "ReferenceInformationType" => {
                            assert!(reference.is_none());
                            reference = Some(ReferenceInformationType::from_str(node.properties[0].as_str()));
                        }
                        "Tangents" => {
                            assert!(elements.is_none());
                            elements = Some(node.properties[0].as_f64_array_exact().to_vec());
                        }
                        "TangentsIndex" => {
                            assert!(indices.is_none());
                            indices = Some(node.properties[0].as_i32_array_exact().to_vec());
                        }
                        other => {
                            panic!("Unexpected layer elements property {:?}", other);
                        }
                    }
                    stack.pop();
                }

                let mapping = mapping.unwrap();

                assert!(layer.tangents.is_none());
                layer.tangents = Some(Attribute {
                    elements: elements.unwrap(),
                    indices: match mapping {
                        AttributeMapping::AllSame => {
                            // Deal with all same not having indices but ref is sometimes index to direct...
                            None
                        }
                        _ => match reference.unwrap() {
                            ReferenceInformationType::Direct => {
                                assert!(indices.is_none());
                                None
                            }
                            ReferenceInformationType::IndexToDirect => Some(indices.unwrap()),
                        },
                    },
                    mapping,
                })
            }
            "LayerElementUV" => {
                let layer_index = node.properties[0].to_i32_exact() as usize;
                while layers.len() < layer_index + 1 {
                    layers.push(GeometryLayer::default());
                }
                let layer = &mut layers[layer_index];

                let mut mapping = None;
                let mut reference = None;
                let mut elements = None;
                let mut indices = None;

                for node in node.children.iter() {
                    stack.push(node.name.clone());
                    match node.name.as_str() {
                        "Version" => {
                            // Don't care.
                        }
                        "Name" => {
                            // Don't care.
                        }
                        "MappingInformationType" => {
                            assert!(mapping.is_none());
                            mapping = Some(AttributeMapping::from_str(node.properties[0].as_str()));
                        }
                        "ReferenceInformationType" => {
                            assert!(reference.is_none());
                            reference = Some(ReferenceInformationType::from_str(node.properties[0].as_str()));
                        }
                        "UV" => {
                            assert!(elements.is_none());
                            elements = Some(node.properties[0].as_f64_array_exact().to_vec());
                        }
                        "UVIndex" => {
                            assert!(indices.is_none());
                            indices = Some(node.properties[0].as_i32_array_exact().to_vec());
                        }
                        other => {
                            panic!("Unexpected layer elements property {:?}", other);
                        }
                    }
                    stack.pop();
                }

                let mapping = mapping.unwrap();

                assert!(layer.uvs.is_none());
                layer.uvs = Some(Attribute {
                    elements: elements.unwrap(),
                    indices: match mapping {
                        AttributeMapping::AllSame => {
                            // Deal with all same not having indices but ref is sometimes index to direct...
                            None
                        }
                        _ => match reference.unwrap() {
                            ReferenceInformationType::Direct => {
                                assert!(indices.is_none());
                                None
                            }
                            ReferenceInformationType::IndexToDirect => Some(indices.unwrap()),
                        },
                    },
                    mapping,
                })
            }
            "LayerElementMaterial" => {
                // NOTE(mickvangelderen): Deviates from the rest.
                let layer_index = node.properties[0].to_i32_exact() as usize;
                while layers.len() < layer_index + 1 {
                    layers.push(GeometryLayer::default());
                }
                let layer = &mut layers[layer_index];

                let mut mapping = None;
                let mut reference = None;
                let mut elements = None;

                for node in node.children.iter() {
                    stack.push(node.name.clone());
                    match node.name.as_str() {
                        "Version" => {
                            // Don't care.
                        }
                        "Name" => {
                            // Don't care.
                        }
                        "MappingInformationType" => {
                            assert!(mapping.is_none());
                            mapping = Some(AttributeMapping::from_str(node.properties[0].as_str()));
                        }
                        "ReferenceInformationType" => {
                            assert!(reference.is_none());
                            reference = Some(ReferenceInformationType::from_str(node.properties[0].as_str()));
                        }
                        "Materials" => {
                            assert!(elements.is_none());
                            elements = Some(node.properties[0].as_i32_array_exact().to_vec());
                        }
                        other => {
                            panic!("Unexpected layer elements property {:?}", other);
                        }
                    }
                    stack.pop();
                }

                let mapping = mapping.unwrap();

                assert!(layer.materials.is_none());
                layer.materials = Some(Attribute {
                    elements: elements.unwrap(),
                    indices: None,
                    mapping,
                })
            }
            other => {
                panic!("Unexpected geometry property {:?}", other);
            }
        }
        stack.pop();
    }

    stack.pop();

    Geometry {
        id,
        name,
        vertices: vertices.unwrap(),
        polygon_vertex_index: polygon_vertex_index.unwrap(),
        edges,
        layers,
    }
}

fn main() {
    let file = read("resources/sun_temple/SunTemple.fbx").unwrap();
    // let file = read("resources/bistro/Bistro_Exterior.fbx").unwrap();
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

    let _objects = objects.expect("Missing \"Objects\" node.");

    dbg!(&_objects.materials);
    for geometry in _objects.geometries.iter() {
        dbg!(geometry.

    }

    // dbg!(&objects);
}
