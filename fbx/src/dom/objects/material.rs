use crate::tree::*;

#[derive(Debug)]
pub struct Material {
    pub id: i64,
    pub name: String,
    pub properties: MaterialProperties,
}

impl_properties70!(MaterialProperties {
    "Transparent" | "TransparentColor" => transparent_color: [f64; 3] = [0.0; 3],
    "TransparencyFactor" | "TransparentFactor" => transparent_factor: f64 = 0.0,
    "Emissive" | "EmissiveColor" => emissive_color: [f64; 3] = [0.0; 3],
    "EmissiveFactor" => emissive_factor: f64 = 1.0,
    "Ambient" | "AmbientColor" => ambient_color: [f64; 3] = [0.2; 3],
    "AmbientFactor" => ambient_factor: f64 = 1.0,
    "Diffuse" | "DiffuseColor" => diffuse_color: [f64; 3] = [0.8; 3],
    "DiffuseFactor" => diffuse_factor: f64 = 1.0,
    "Specular" | "SpecularColor" => specular_color: [f64; 3] = [0.2; 3],
    "SpecularFactor" => specular_factor: f64 = 1.0,
    "Shininess" | "ShininessExponent" => shininess: f64 = 20.0,
    "Opacity" => opacity: f64 = 1.0,
});

impl Material {
    pub fn from_fbx(node: &Node, stack: &mut Vec<String>) -> Self {
        stack.push(node.name.clone());

        let id = node.properties[0].to_i64_exact();

        let name = {
            let name = node.properties[1].as_str();
            let postfix = "\u{0}\u{1}Material";
            assert!(name.ends_with(postfix));
            String::from(&name[0..name.len() - postfix.len()])
        };

        assert_eq!("", node.properties[2].as_str());

        let mut properties = None;

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
                    assert!(properties.is_none());
                    properties = Some(MaterialProperties::from_fbx(node, stack));
                }
                unknown => {
                    panic!("Unknown material property: {:?}", unknown);
                }
            }
            stack.pop();
        }

        stack.pop();

        Self {
            id,
            name,
            properties: properties.unwrap(),
        }
    }
}
