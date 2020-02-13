use crate::tree::*;
use crate::types::*;

#[derive(Debug)]
pub struct Model {
    pub id: i64,
    pub name: String,
    pub properties: ModelProperties,
}

impl_properties70!(ModelProperties {
    "LclTranslation" | "Lcl Translation" => lcl_translation: [f64; 3] = [0.0; 3],
    "LclRotation" | "Lcl Rotation" => lcl_rotation: [f64; 3] = [0.0; 3],
    "LclScaling" | "Lcl Scaling" => lcl_scaling: [f64; 3] = [1.0; 3],
    "Visibility" => visibility: f64 = 0.0,
    "VisibilityInheritance" => visibility_inheritance: u8 = 0,
    "QuaternionInterpolate" => quaternion_interpolate: QuaternionInterpolationMode = QuaternionInterpolationMode::Off,
    "RotationOffset" => rotation_offset: [f64; 3] = [0.0; 3],
    "RotationPivot" => rotation_pivot: [f64; 3] = [0.0; 3],
    "ScalingOffset" => scaling_offset: [f64; 3] = [0.0; 3],
    "ScalingPivot" => scaling_pivot: [f64; 3] = [0.0; 3],
    "TranslationActive" => translation_active: u8 = 0,
    "TranslationMin" => translation_min: [f64; 3] = [0.0; 3],
    "TranslationMax" => translation_max: [f64; 3] = [0.0; 3],
    "TranslationMinX" => translation_min_x: u8 = 0,
    "TranslationMinY" => translation_min_y: u8 = 0,
    "TranslationMinZ" => translation_min_z: u8 = 0,
    "TranslationMaxX" => translation_max_x: u8 = 0,
    "TranslationMaxY" => translation_max_y: u8 = 0,
    "TranslationMaxZ" => translation_max_z: u8 = 0,
    "RotationOrder" => rotation_order: RotationOrder = RotationOrder::XYZ,
    "RotationSpaceForLimitOnly" => rotation_space_for_limit_only: u8 = 0,
    "RotationStiffnessX" => rotation_stiffness_x: f64 = 0.0,
    "RotationStiffnessY" => rotation_stiffness_y: f64 = 0.0,
    "RotationStiffnessZ" => rotation_stiffness_z: f64 = 0.0,
    "AxisLen" => axis_len: f64 = 0.0,
    "PreRotation" => pre_rotation: [f64; 3] = [0.0; 3],
    "PostRotation" => post_rotation: [f64; 3] = [0.0; 3],
    "RotationActive" => rotation_active: u8 = 0,
    "RotationMin" => rotation_min: [f64; 3] = [0.0; 3],
    "RotationMax" => rotation_max: [f64; 3] = [0.0; 3],
    "RotationMinX" => rotation_min_x: u8 = 0,
    "RotationMinY" => rotation_min_y: u8 = 0,
    "RotationMinZ" => rotation_min_z: u8 = 0,
    "RotationMaxX" => rotation_max_x: u8 = 0,
    "RotationMaxY" => rotation_max_y: u8 = 0,
    "RotationMaxZ" => rotation_max_z: u8 = 0,
    "ScalingActive" => scaling_active: u8 = 0,
    "ScalingMin" => scaling_min: [f64; 3] = [0.0; 3],
    "ScalingMax" => scaling_max: [f64; 3] = [0.0; 3],
    "ScalingMinX" => scaling_min_x: u8 = 0,
    "ScalingMinY" => scaling_min_y: u8 = 0,
    "ScalingMinZ" => scaling_min_z: u8 = 0,
    "ScalingMaxX" => scaling_max_x: u8 = 0,
    "ScalingMaxY" => scaling_max_y: u8 = 0,
    "ScalingMaxZ" => scaling_max_z: u8 = 0,
    "GeometricTranslation" => geometric_translation: [f64; 3] = [0.0; 3],
    "GeometricRotation" => geometric_rotation: [f64; 3] = [0.0; 3],
    "GeometricScaling" => geometric_scaling: [f64; 3] = [1.0; 3],
    "MinDampRangeX" => min_damp_range_x: f64 = 0.0,
    "MinDampRangeY" => min_damp_range_y: f64 = 0.0,
    "MinDampRangeZ" => min_damp_range_z: f64 = 0.0,
    "MaxDampRangeX" => max_damp_range_x: f64 = 0.0,
    "MaxDampRangeY" => max_damp_range_y: f64 = 0.0,
    "MaxDampRangeZ" => max_damp_range_z: f64 = 0.0,
    "MinDampStrengthX" => min_damp_strength_x: f64 = 0.0,
    "MinDampStrengthY" => min_damp_strength_y: f64 = 0.0,
    "MinDampStrengthZ" => min_damp_strength_z: f64 = 0.0,
    "MaxDampStrengthX" => max_damp_strength_x: f64 = 0.0,
    "MaxDampStrengthY" => max_damp_strength_y: f64 = 0.0,
    "MaxDampStrengthZ" => max_damp_strength_z: f64 = 0.0,
    "PreferedAngleX" => prefered_angle_x: f64 = 0.0,
    "PreferedAngleY" => prefered_angle_y: f64 = 0.0,
    "PreferedAngleZ" => prefered_angle_z: f64 = 0.0,
    "Show" => show: u8 = 0,
    "NegativePercentShapeSupport" => negative_percent_shape_support: u8 = 0,
    "DefaultAttributeIndex" => default_attribute_index: i32 = 0,
    "Freeze" => freeze: u8 = 0,
    "LODBox" => lod_box: u8 = 0,
});

// NOTE(mickvangelderen): Don't want to handle these right now.
// pub inherit_type: FbxTransform:EInheritType>, // InheritType
// pub look_at_property: FbxReference, // LookAtProperty
// pub up_vector_property: FbxReference, // UpVectorProperty

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

        let mut properties = None;

        for node in node.children.iter() {
            stack.push(node.name.clone());
            match node.name.as_str() {
                "Properties70" => {
                    assert!(properties.is_none());
                    properties = Some(ModelProperties::from_fbx(node, stack))
                }
                unknown => {
                    eprintln!("Unhandled Model property {:?}", unknown);
                }
            }
            stack.pop();
        }

        stack.pop();

        Model {
            id,
            name,
            properties: properties.unwrap(),
        }
    }
}
