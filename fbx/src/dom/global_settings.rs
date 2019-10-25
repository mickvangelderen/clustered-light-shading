use crate::tree::*;

#[derive(Debug)]
pub struct GlobalSettings {
    up_axis: i32,
    up_axis_sign: i32,
    front_axis: i32,
    front_axis_sign: i32,
    coord_axis: i32,
    coord_axis_sign: i32,
    original_up_axis: i32,
    original_up_axis_sign: i32,
    unit_scale_factor: f64,
    original_unit_scale_factor: f64,
    ambient_color: [f64; 3],
}

impl GlobalSettings {
    pub fn from_fbx(node: &Node, stack: &mut Vec<String>) -> Self {
        stack.push(node.name.clone());

        let mut up_axis = None;
        let mut up_axis_sign = None;
        let mut front_axis = None;
        let mut front_axis_sign = None;
        let mut coord_axis = None;
        let mut coord_axis_sign = None;
        let mut original_up_axis = None;
        let mut original_up_axis_sign = None;
        let mut unit_scale_factor = None;
        let mut original_unit_scale_factor = None;
        let mut ambient_color = None;

        for node in node.children.iter() {
            stack.push(node.name.clone());

            match node.name.as_str() {
                "Version" => {
                    // Don't care.
                }
                "Properties70" => {
                    for node in node.children.iter() {
                        stack.push(node.name.clone());

                        assert_eq!("P", node.name.as_str());

                        match node.properties[0].as_str() {
                            "DefaultCamera" | "TimeMode" | "TimeSpanStart" | "TimeSpanStop" | "CustomFrameRate"
                            | "TimeProtocol" | "SnapOnFrameMode" | "TimeMarker" | "CurrentTimeMarker" => {
                                // Don't care.
                            }

                            "UpAxis" => {
                                assert!(up_axis.is_none());
                                up_axis = Some(node.properties[4].to_i32_exact())
                            }
                            "UpAxisSign" => {
                                assert!(up_axis_sign.is_none());
                                up_axis_sign = Some(node.properties[4].to_i32_exact())
                            }
                            "FrontAxis" => {
                                assert!(front_axis.is_none());
                                front_axis = Some(node.properties[4].to_i32_exact())
                            }
                            "FrontAxisSign" => {
                                assert!(front_axis_sign.is_none());
                                front_axis_sign = Some(node.properties[4].to_i32_exact())
                            }
                            "CoordAxis" => {
                                assert!(coord_axis.is_none());
                                coord_axis = Some(node.properties[4].to_i32_exact())
                            }
                            "CoordAxisSign" => {
                                assert!(coord_axis_sign.is_none());
                                coord_axis_sign = Some(node.properties[4].to_i32_exact())
                            }
                            "OriginalUpAxis" => {
                                assert!(original_up_axis.is_none());
                                original_up_axis = Some(node.properties[4].to_i32_exact())
                            }
                            "OriginalUpAxisSign" => {
                                assert!(original_up_axis_sign.is_none());
                                original_up_axis_sign = Some(node.properties[4].to_i32_exact())
                            }
                            "UnitScaleFactor" => {
                                assert!(unit_scale_factor.is_none());
                                unit_scale_factor = Some(node.properties[4].to_f64_exact())
                            }
                            "OriginalUnitScaleFactor" => {
                                assert!(original_unit_scale_factor.is_none());
                                original_unit_scale_factor = Some(node.properties[4].to_f64_exact())
                            }
                            "AmbientColor" => {
                                assert!(ambient_color.is_none());
                                ambient_color = Some([
                                    node.properties[4].to_f64_exact(),
                                    node.properties[5].to_f64_exact(),
                                    node.properties[6].to_f64_exact(),
                                ])
                            }
                            unknown => {
                                eprintln!("Unknown GlobalSettings Properties70 property kind {:?}", unknown);
                            }
                        }

                        stack.pop();
                    }
                }
                unknown => {
                    panic!("Unknown GlobalSettings property {:?}", unknown);
                }
            }

            stack.pop();
        }

        stack.pop();

        Self {
            up_axis: up_axis.unwrap(),
            up_axis_sign: up_axis_sign.unwrap(),
            front_axis: front_axis.unwrap(),
            front_axis_sign: front_axis_sign.unwrap(),
            coord_axis: coord_axis.unwrap(),
            coord_axis_sign: coord_axis_sign.unwrap(),
            original_up_axis: original_up_axis.unwrap(),
            original_up_axis_sign: original_up_axis_sign.unwrap(),
            unit_scale_factor: unit_scale_factor.unwrap(),
            original_unit_scale_factor: original_unit_scale_factor.unwrap(),
            ambient_color: ambient_color.unwrap(),
        }
    }
}
