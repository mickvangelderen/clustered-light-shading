use crate::tree::*;

#[derive(Debug)]
pub struct Geometry {
    pub id: i64,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    pub elements: Vec<E>,
    pub indices: Option<Vec<I>>,
    pub mapping: AttributeMapping,
}

impl<E, I> Attribute<E, I>
where
    I: std::convert::TryInto<usize> + Copy,
    <I as std::convert::TryInto<usize>>::Error: std::fmt::Debug,
{
    pub fn select_polygon_vertex_index(&self, indices: &PolygonVertexIndices) -> usize {
        let indirect_index = match self.mapping {
            AttributeMapping::ByPolygon => indices.polygon_index,
            AttributeMapping::ByVertex => indices.vertex_index,
            AttributeMapping::ByPolygonVertex => indices.polygon_vertex_index,
            AttributeMapping::ByEdge => panic!("Don't have edge index here"),
            AttributeMapping::AllSame => 0,
        };

        match self.indices {
            Some(ref indices) => indices[indirect_index].try_into().unwrap(),
            None => indirect_index,
        }
    }
}

pub struct PolygonVertexIndices {
    pub polygon_index: usize,
    pub vertex_index: usize,
    pub polygon_vertex_index: usize,
}

impl Geometry {
    pub fn from_fbx(node: &Node, stack: &mut Vec<String>) -> Self {
        stack.push(node.name.clone());

        let id = node.properties[0].to_i64_exact();

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
                            "NormalsW" => {
                                // TODO: Figure out what this is.
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

        Self {
            id,
            name,
            vertices: vertices.unwrap(),
            polygon_vertex_index: polygon_vertex_index.unwrap(),
            edges,
            layers,
        }
    }
}
