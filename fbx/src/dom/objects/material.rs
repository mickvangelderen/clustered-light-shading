use crate::tree::*;

#[derive(Debug)]
pub struct Material {
    pub id: u64,
    pub name: String,
    pub properties: MaterialProperties,
}

#[derive(Debug)]
pub struct MaterialProperties {
    pub transparent_color: [f64; 3],
    pub transparent_factor: f64,

    pub emissive_color: [f64; 3],
    pub emissive_factor: f64,

    pub ambient_color: [f64; 3],
    pub ambient_factor: f64,

    pub diffuse_color: [f64; 3],
    pub diffuse_factor: f64,

    pub specular_color: [f64; 3],
    pub specular_factor: f64,

    pub shininess: f64,
    pub opacity: f64,
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

impl Material {
    pub fn from_fbx(node: &Node, stack: &mut Vec<String>) -> Self {
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

        Self { id, name, properties }
    }
}
