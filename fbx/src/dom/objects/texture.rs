use crate::tree::*;

use std::path::PathBuf;

#[derive(Debug)]
pub struct Texture {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub file_path: PathBuf,
    pub properties: TextureProperties,
    pub uv_translation: [f64; 2],
    pub uv_scaling: [f64; 2],
    pub alpha_source: String,
}

impl_properties70!(TextureProperties {
    "CurrentTextureBlendMode" => blend_mode: i32 = 0,
    "UVSet" => uv_set: String = String::new(),
    "UseMaterial" => use_material: i32 = 0,
});

impl Texture {
    pub fn from_fbx(node: &Node, stack: &mut Vec<String>) -> Self {
        stack.push(node.name.clone());

        let id = node.properties[0].to_i64_exact();

        let name = {
            let name = node.properties[1].as_str();
            let postfix = "\u{0}\u{1}Texture";
            assert!(name.ends_with(postfix));
            String::from(&name[0..name.len() - postfix.len()])
        };

        assert_eq!("", node.properties[2].as_str());

        let mut kind = None;
        let mut file_path = None;
        let mut properties = None;
        let mut uv_translation = None;
        let mut uv_scaling = None;
        let mut alpha_source = None;

        for node in node.children.iter() {
            stack.push(node.name.clone());
            match node.name.as_str() {
                "Version" | "TextureName" | "FileName" | "Cropping" | "Media" => {
                    // Don't care.
                }
                "Type" => {
                    assert!(kind.is_none());
                    kind = Some(node.properties[0].as_str().to_string());
                }
                "RelativeFilename" => {
                    assert!(file_path.is_none());
                    file_path = Some(PathBuf::from(node.properties[0].as_str()));
                }
                "ModelUVTranslation" => {
                    assert!(uv_translation.is_none());
                    uv_translation = Some([node.properties[0].to_f64_exact(), node.properties[1].to_f64_exact()]);
                }
                "ModelUVScaling" => {
                    assert!(uv_scaling.is_none());
                    uv_scaling = Some([node.properties[0].to_f64_exact(), node.properties[1].to_f64_exact()]);
                }
                "Texture_Alpha_Source" => {
                    assert!(alpha_source.is_none());
                    alpha_source = Some(node.properties[0].as_str().to_string());
                }
                "Properties70" => {
                    assert!(properties.is_none());
                    properties = Some(TextureProperties::from_fbx(node, stack));
                }
                unknown => {
                    panic!("Unknown texture property: {:?}", unknown);
                }
            }
            stack.pop();
        }

        stack.pop();

        Texture {
            id,
            name,
            kind: kind.unwrap(),
            file_path: file_path.unwrap(),
            properties: properties.unwrap(),
            uv_translation: uv_translation.unwrap(),
            uv_scaling: uv_scaling.unwrap(),
            alpha_source: alpha_source.unwrap(),
        }
    }
}
