use crate::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Texture {
    id: u64,
    name: String,
    kind: String,
    file_path: PathBuf,
    properties: TextureProperties,
    uv_translation: [f64; 2],
    uv_scaling: [f64; 2],
    alpha_source: String,
}

#[derive(Debug)]
pub struct TextureProperties {
    blend_mode: i32,
    uv_set: String,
    use_material: i32,
}

impl Default for TextureProperties {
    fn default() -> Self {
        Self {
            blend_mode: 0,
            uv_set: String::new(),
            use_material: 0,
        }
    }
}

impl Texture {
    pub fn from_fbx(node: &Node, stack: &mut Vec<String>) -> Self {
        stack.push(node.name.clone());

        let id = node.properties[0].to_i64_exact() as u64;

        let name = {
            let name = node.properties[1].as_str();
            let postfix = "\u{0}\u{1}Texture";
            assert!(name.ends_with(postfix));
            String::from(&name[0..name.len() - postfix.len()])
        };

        assert_eq!("", node.properties[2].as_str());

        let mut kind = None;
        let mut file_path = None;
        let mut properties = TextureProperties::default();
        let mut uv_translation = None;
        let mut uv_scaling = None;
        let mut alpha_source = None;

        for node in node.children.iter() {
            stack.push(node.name.clone());
            match node.name.as_str() {
                "Version" | "Shading" | "Culling" => {
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
                    for node in node.children.iter() {
                        stack.push(node.name.clone());

                        assert_eq!(node.name.as_str(), "P");

                        match node.properties[0].as_str() {
                            "CurrentTextureBlendMode" => {
                                properties.blend_mode = node.properties[4].to_i32_exact();
                            }
                            "UVSet" => {
                                properties.uv_set = node.properties[4].as_str().to_string();
                            }
                            "UseMaterial" => {
                                properties.use_material = node.properties[4].to_i32_exact();
                            }
                            unknown => {
                                eprintln!("Unknown texture properties property: {:?}", unknown);
                            }
                        }

                        stack.pop();
                    }
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
            properties,
            uv_translation: uv_translation.unwrap(),
            uv_scaling: uv_scaling.unwrap(),
            alpha_source: alpha_source.unwrap(),
        }
    }
}

// P "AxisLen" "double" "Number" "" 10: f64,
// P "DefaultAttributeIndex" "int" "Integer" "" -1: i32,
// P "Freeze" "bool" "" "" 0: i32,
// P "GeometricRotation" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "GeometricRotation" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "GeometricScaling" "Vector3D" "Vector" "" 1: f64, 1: f64, 1: f64,
// P "GeometricScaling" "Vector3D" "Vector" "" 1: f64, 1: f64, 1: f64,
// P "GeometricTranslation" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "GeometricTranslation" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "InheritType" "enum" "" "" 0: i32,
// P "LODBox" "bool" "" "" 0: i32,
// P "Lcl Rotation" "Lcl Rotation" "" "A" 0: f64, 0: f64, 0: f64,
// P "Lcl Scaling" "Lcl Scaling" "" "A" 1: f64, 1: f64, 1: f64,
// P "Lcl Translation" "Lcl Translation" "" "A" 0: f64, 0: f64, 0: f64,
// P "LookAtProperty" "object" "" ""
// P "MaxDampRangeX" "double" "Number" "" 0: f64,
// P "MaxDampRangeY" "double" "Number" "" 0: f64,
// P "MaxDampRangeZ" "double" "Number" "" 0: f64,
// P "MaxDampStrengthX" "double" "Number" "" 0: f64,
// P "MaxDampStrengthY" "double" "Number" "" 0: f64,
// P "MaxDampStrengthZ" "double" "Number" "" 0: f64,
// P "MinDampRangeX" "double" "Number" "" 0: f64,
// P "MinDampRangeY" "double" "Number" "" 0: f64,
// P "MinDampRangeZ" "double" "Number" "" 0: f64,
// P "MinDampStrengthX" "double" "Number" "" 0: f64,
// P "MinDampStrengthY" "double" "Number" "" 0: f64,
// P "MinDampStrengthZ" "double" "Number" "" 0: f64,
// P "NegativePercentShapeSupport" "bool" "" "" 1: i32,
// P "PostRotation" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "PreRotation" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "PreferedAngleX" "double" "Number" "" 0: f64,
// P "PreferedAngleY" "double" "Number" "" 0: f64,
// P "PreferedAngleZ" "double" "Number" "" 0: f64,
// P "QuaternionInterpolate" "enum" "" "" 0: i32,
// P "RotationActive" "bool" "" "" 0: i32,
// P "RotationMax" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "RotationMaxX" "bool" "" "" 0: i32,
// P "RotationMaxY" "bool" "" "" 0: i32,
// P "RotationMaxZ" "bool" "" "" 0: i32,
// P "RotationMin" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "RotationMinX" "bool" "" "" 0: i32,
// P "RotationMinY" "bool" "" "" 0: i32,
// P "RotationMinZ" "bool" "" "" 0: i32,
// P "RotationOffset" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "RotationOrder" "enum" "" "" 0: i32,
// P "RotationPivot" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "RotationSpaceForLimitOnly" "bool" "" "" 0: i32,
// P "RotationStiffnessX" "double" "Number" "" 0: f64,
// P "RotationStiffnessY" "double" "Number" "" 0: f64,
// P "RotationStiffnessZ" "double" "Number" "" 0: f64,
// P "ScalingActive" "bool" "" "" 0: i32,
// P "ScalingMax" "Vector3D" "Vector" "" 1: f64, 1: f64, 1: f64,
// P "ScalingMaxX" "bool" "" "" 0: i32,
// P "ScalingMaxY" "bool" "" "" 0: i32,
// P "ScalingMaxZ" "bool" "" "" 0: i32,
// P "ScalingMin" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "ScalingMinX" "bool" "" "" 0: i32,
// P "ScalingMinY" "bool" "" "" 0: i32,
// P "ScalingMinZ" "bool" "" "" 0: i32,
// P "ScalingOffset" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "ScalingPivot" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "Show" "bool" "" "" 1: i32,
// P "TranslationActive" "bool" "" "" 0: i32,
// P "TranslationMax" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "TranslationMaxX" "bool" "" "" 0: i32,
// P "TranslationMaxY" "bool" "" "" 0: i32,
// P "TranslationMaxZ" "bool" "" "" 0: i32,
// P "TranslationMin" "Vector3D" "Vector" "" 0: f64, 0: f64, 0: f64,
// P "TranslationMinX" "bool" "" "" 0: i32,
// P "TranslationMinY" "bool" "" "" 0: i32,
// P "TranslationMinZ" "bool" "" "" 0: i32,
// P "UpVectorProperty" "object" "" ""
// P "Visibility Inheritance" "Visibility Inheritance" "" "" 1: i32,
// P "Visibility" "Visibility" "" "A" 1: f64,
