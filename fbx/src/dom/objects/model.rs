use crate::tree::*;

#[derive(Debug)]
pub struct Model {
    pub id: i64,
    pub name: String,
    pub properties: ModelProperties,
}

#[derive(Debug)]
pub struct ModelProperties {
    pub lcl_translation: [f64; 3],                           // LclTranslation
    pub lcl_rotation: [f64; 3],                              // LclRotation
    pub lcl_scaling: [f64; 3],                               // LclScaling
    pub visibility: f64,                                     // Visibility
    pub visibility_inheritance: u8,                          // VisibilityInheritance
    pub quaternion_interpolate: QuaternionInterpolationMode, // QuaternionInterpolate
    pub rotation_offset: [f64; 3],                           // RotationOffset
    pub rotation_pivot: [f64; 3],                            // RotationPivot
    pub scaling_offset: [f64; 3],                            // ScalingOffset
    pub scaling_pivot: [f64; 3],                             // ScalingPivot
    pub translation_active: u8,                              // TranslationActive
    pub translation_min: [f64; 3],                           // TranslationMin
    pub translation_max: [f64; 3],                           // TranslationMax
    pub translation_min_x: u8,                               // TranslationMinX
    pub translation_min_y: u8,                               // TranslationMinY
    pub translation_min_z: u8,                               // TranslationMinZ
    pub translation_max_x: u8,                               // TranslationMaxX
    pub translation_max_y: u8,                               // TranslationMaxY
    pub translation_max_z: u8,                               // TranslationMaxZ
    pub rotation_order: RotationOrder,                       // RotationOrder
    pub rotation_space_for_limit_only: u8,                   // RotationSpaceForLimitOnly
    pub rotation_stiffness_x: f64,                           // RotationStiffnessX
    pub rotation_stiffness_y: f64,                           // RotationStiffnessY
    pub rotation_stiffness_z: f64,                           // RotationStiffnessZ
    pub axis_len: f64,                                       // AxisLen
    pub pre_rotation: [f64; 3],                              // PreRotation
    pub post_rotation: [f64; 3],                             // PostRotation
    pub rotation_active: u8,                                 // RotationActive
    pub rotation_min: [f64; 3],                              // RotationMin
    pub rotation_max: [f64; 3],                              // RotationMax
    pub rotation_min_x: u8,                                  // RotationMinX
    pub rotation_min_y: u8,                                  // RotationMinY
    pub rotation_min_z: u8,                                  // RotationMinZ
    pub rotation_max_x: u8,                                  // RotationMaxX
    pub rotation_max_y: u8,                                  // RotationMaxY
    pub rotation_max_z: u8,                                  // RotationMaxZ
    pub scaling_active: u8,                                  // ScalingActive
    pub scaling_min: [f64; 3],                               // ScalingMin
    pub scaling_max: [f64; 3],                               // ScalingMax
    pub scaling_min_x: u8,                                   // ScalingMinX
    pub scaling_min_y: u8,                                   // ScalingMinY
    pub scaling_min_z: u8,                                   // ScalingMinZ
    pub scaling_max_x: u8,                                   // ScalingMaxX
    pub scaling_max_y: u8,                                   // ScalingMaxY
    pub scaling_max_z: u8,                                   // ScalingMaxZ
    pub geometric_translation: [f64; 3],                     // GeometricTranslation
    pub geometric_rotation: [f64; 3],                        // GeometricRotation
    pub geometric_scaling: [f64; 3],                         // GeometricScaling
    pub min_damp_range_x: f64,                               // MinDampRangeX
    pub min_damp_range_y: f64,                               // MinDampRangeY
    pub min_damp_range_z: f64,                               // MinDampRangeZ
    pub max_damp_range_x: f64,                               // MaxDampRangeX
    pub max_damp_range_y: f64,                               // MaxDampRangeY
    pub max_damp_range_z: f64,                               // MaxDampRangeZ
    pub min_damp_strength_x: f64,                            // MinDampStrengthX
    pub min_damp_strength_y: f64,                            // MinDampStrengthY
    pub min_damp_strength_z: f64,                            // MinDampStrengthZ
    pub max_damp_strength_x: f64,                            // MaxDampStrengthX
    pub max_damp_strength_y: f64,                            // MaxDampStrengthY
    pub max_damp_strength_z: f64,                            // MaxDampStrengthZ
    pub prefered_angle_x: f64,                               // PreferedAngleX
    pub prefered_angle_y: f64,                               // PreferedAngleY
    pub prefered_angle_z: f64,                               // PreferedAngleZ
    pub show: u8,                                            // Show
    pub negative_percent_shape_support: u8,                  // NegativePercentShapeSupport
    pub default_attribute_index: i32,                        // DefaultAttributeIndex
    pub freeze: u8,                                          // Freeze
    pub lod_box: u8,                                         // LODBox
}

// NOTE(mickvangelderen): Don't want to handle these right now.
// pub inherit_type: FbxTransform:EInheritType>, // InheritType
// pub look_at_property: FbxReference, // LookAtProperty
// pub up_vector_property: FbxReference, // UpVectorProperty

impl Default for ModelProperties {
    fn default() -> Self {
        Self {
            lcl_translation: [0.0; 3],                                // LclTranslation
            lcl_rotation: [0.0; 3],                                   // LclRotation
            lcl_scaling: [1.0; 3],                                    // LclScaling
            visibility: 0.0,                                          // Visibility
            visibility_inheritance: 0,                                // VisibilityInheritance
            quaternion_interpolate: QuaternionInterpolationMode::Off, // QuaternionInterpolate
            rotation_offset: [0.0; 3],                                // RotationOffset
            rotation_pivot: [0.0; 3],                                 // RotationPivot
            scaling_offset: [0.0; 3],                                 // ScalingOffset
            scaling_pivot: [0.0; 3],                                  // ScalingPivot
            translation_active: 0,                                    // TranslationActive
            translation_min: [0.0; 3],                                // TranslationMin
            translation_max: [0.0; 3],                                // TranslationMax
            translation_min_x: 0,                                     // TranslationMinX
            translation_min_y: 0,                                     // TranslationMinY
            translation_min_z: 0,                                     // TranslationMinZ
            translation_max_x: 0,                                     // TranslationMaxX
            translation_max_y: 0,                                     // TranslationMaxY
            translation_max_z: 0,                                     // TranslationMaxZ
            rotation_order: RotationOrder::XYZ,                       // RotationOrder
            rotation_space_for_limit_only: 0,                         // RotationSpaceForLimitOnly
            rotation_stiffness_x: 0.0,                                // RotationStiffnessX
            rotation_stiffness_y: 0.0,                                // RotationStiffnessY
            rotation_stiffness_z: 0.0,                                // RotationStiffnessZ
            axis_len: 0.0,                                            // AxisLen
            pre_rotation: [0.0; 3],                                   // PreRotation
            post_rotation: [0.0; 3],                                  // PostRotation
            rotation_active: 0,                                       // RotationActive
            rotation_min: [0.0; 3],                                   // RotationMin
            rotation_max: [0.0; 3],                                   // RotationMax
            rotation_min_x: 0,                                        // RotationMinX
            rotation_min_y: 0,                                        // RotationMinY
            rotation_min_z: 0,                                        // RotationMinZ
            rotation_max_x: 0,                                        // RotationMaxX
            rotation_max_y: 0,                                        // RotationMaxY
            rotation_max_z: 0,                                        // RotationMaxZ
            scaling_active: 0,                                        // ScalingActive
            scaling_min: [0.0; 3],                                    // ScalingMin
            scaling_max: [0.0; 3],                                    // ScalingMax
            scaling_min_x: 0,                                         // ScalingMinX
            scaling_min_y: 0,                                         // ScalingMinY
            scaling_min_z: 0,                                         // ScalingMinZ
            scaling_max_x: 0,                                         // ScalingMaxX
            scaling_max_y: 0,                                         // ScalingMaxY
            scaling_max_z: 0,                                         // ScalingMaxZ
            geometric_translation: [0.0; 3],                          // GeometricTranslation
            geometric_rotation: [0.0; 3],                             // GeometricRotation
            geometric_scaling: [1.0; 3],                              // GeometricScaling
            min_damp_range_x: 0.0,                                    // MinDampRangeX
            min_damp_range_y: 0.0,                                    // MinDampRangeY
            min_damp_range_z: 0.0,                                    // MinDampRangeZ
            max_damp_range_x: 0.0,                                    // MaxDampRangeX
            max_damp_range_y: 0.0,                                    // MaxDampRangeY
            max_damp_range_z: 0.0,                                    // MaxDampRangeZ
            min_damp_strength_x: 0.0,                                 // MinDampStrengthX
            min_damp_strength_y: 0.0,                                 // MinDampStrengthY
            min_damp_strength_z: 0.0,                                 // MinDampStrengthZ
            max_damp_strength_x: 0.0,                                 // MaxDampStrengthX
            max_damp_strength_y: 0.0,                                 // MaxDampStrengthY
            max_damp_strength_z: 0.0,                                 // MaxDampStrengthZ
            prefered_angle_x: 0.0,                                    // PreferedAngleX
            prefered_angle_y: 0.0,                                    // PreferedAngleY
            prefered_angle_z: 0.0,                                    // PreferedAngleZ
            show: 0,                                                  // Show
            negative_percent_shape_support: 0,                        // NegativePercentShapeSupport
            default_attribute_index: 0,                               // DefaultAttributeIndex
            freeze: 0,                                                // Freeze
            lod_box: 0,                                               // LODBox
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
                                // FIXME(mickvangelderen): Ignoring many of the
                                // properties right now. Even though there is a
                                // field for it. Need to generate Properties70
                                // structs with a macro because its too much
                                // typing.

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

/// https://help.autodesk.com/view/FBX/2017/ENU/?guid=__cpp_ref_fbxmath_8h_html
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

/// https://help.autodesk.com/view/FBX/2017/ENU/?guid=__cpp_ref_fbxmath_8h_html
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum RotationOrder {
    XYZ = 0,
    XZY = 1,
    YZX = 2,
    YXZ = 3,
    ZXY = 4,
    ZYX = 5,
    SphericXYZ = 6,
}

/// https://help.autodesk.com/view/FBX/2017/ENU/?guid=__cpp_ref_fbxmath_8h_html
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum QuaternionInterpolationMode {
    Off = 0,
    Classic = 1,
    Slerp = 2,
    Cubic = 3,
    TangentDependent = 4,
}
