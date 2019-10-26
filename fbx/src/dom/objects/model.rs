use crate::tree::*;

#[derive(Debug)]
pub struct Model {
    pub id: i64,
    pub name: String,
    pub properties: ModelProperties,
}

#[derive(Debug)]
pub struct ModelProperties {
    pub rotation_offset: [f64; 3],
    pub rotation_pivot: [f64; 3],
    pub scaling_offset: [f64; 3],
    pub scaling_pivot: [f64; 3],
    pub pre_rotation: [f64; 3],
    pub post_rotation: [f64; 3],
    pub geometric_translation: [f64; 3],
    pub geometric_rotation: [f64; 3],
    pub geometric_scaling: [f64; 3],
    pub lcl_translation: [f64; 3],
    pub lcl_rotation: [f64; 3],
    pub lcl_scaling: [f64; 3],
}

impl Default for ModelProperties {
    fn default() -> Self {
        Self {
            rotation_offset: [0.0; 3],
            rotation_pivot: [0.0; 3],
            scaling_offset: [0.0; 3],
            scaling_pivot: [0.0; 3],
            pre_rotation: [0.0; 3],
            post_rotation: [0.0; 3],
            geometric_translation: [0.0; 3],
            geometric_rotation: [0.0; 3],
            geometric_scaling: [1.0; 3],
            lcl_translation: [0.0; 3],
            lcl_rotation: [0.0; 3],
            lcl_scaling: [1.0; 3],
        }
    }
}

impl Model {
    pub fn from_fbx(node: &Node, stack: &mut Vec<String>) -> Self {
        stack.push(node.name.clone());

        let id = node.properties[0].to_i64_exact();

        let name = {
            let name = node.properties[1].as_str();
            let postfix = "\u{0}\u{1}Model";
            assert!(name.ends_with(postfix));
            String::from(&name[0..name.len() - postfix.len()])
        };

        assert_eq!("Mesh", node.properties[2].as_str());

        let mut properties = ModelProperties::default();

        for node in node.children.iter() {
            stack.push(node.name.clone());
            match node.name.as_str() {
                "Version" | "Shading" | "Culling" => {
                    // Don't care.
                }
                "Properties70" => {
                    for node in node.children.iter() {
                        stack.push(node.name.clone());

                        assert_eq!(node.name.as_str(), "P");

                        match node.properties[0].as_str() {
                            "RotationOffset" => {
                                properties.rotation_offset = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "RotationPivot" => {
                                properties.rotation_pivot = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "ScalingOffset" => {
                                properties.scaling_offset = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "ScalingPivot" => {
                                properties.rotation_pivot = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "PreRotation" => {
                                properties.pre_rotation = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "PostRotation" => {
                                properties.post_rotation = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "GeometricTranslation" => {
                                properties.geometric_translation = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "GeometricRotation" => {
                                properties.geometric_rotation = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "GeometricScaling" => {
                                properties.geometric_scaling = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "Lcl Translation" => {
                                properties.lcl_translation = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "Lcl Rotation" => {
                                properties.lcl_rotation = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            "Lcl Scaling" => {
                                properties.lcl_scaling = [
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ];
                            }
                            _unknown => {
                                // Don't care.
                                // eprintln!("Unknown model properties property: {:?}", unknown);
                            }
                        }

                        stack.pop();
                    }
                }
                unknown => {
                    panic!("Unknown model property: {:?}", unknown);
                }
            }
            stack.pop();
        }

        stack.pop();

        Model { id, name, properties }
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
