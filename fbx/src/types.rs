/// https://help.autodesk.com/view/FBX/2017/ENU/?guid=__cpp_ref_fbxmath_8h_html
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::X),
            1 => Some(Self::Y),
            2 => Some(Self::Z),
            _ => None,
        }
    }
}


/// https://help.autodesk.com/view/FBX/2017/ENU/?guid=__cpp_ref_fbxmath_8h_html
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RotationOrder {
    XYZ,
    XZY,
    YZX,
    YXZ,
    ZXY,
    ZYX,
    SphericXYZ,
}

impl RotationOrder {
    pub fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::XYZ),
            1 => Some(Self::XZY),
            2 => Some(Self::YZX),
            3 => Some(Self::YXZ),
            4 => Some(Self::ZXY),
            5 => Some(Self::ZYX),
            6 => Some(Self::SphericXYZ),
            _ => None,
        }
    }
}

/// https://help.autodesk.com/view/FBX/2017/ENU/?guid=__cpp_ref_fbxmath_8h_html
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum QuaternionInterpolationMode {
    Off,
    Classic,
    Slerp,
    Cubic,
    TangentDependent,
}

impl QuaternionInterpolationMode {
    pub fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Off),
            1 => Some(Self::Classic),
            2 => Some(Self::Slerp),
            3 => Some(Self::Cubic),
            4 => Some(Self::TangentDependent),
            _ => None,
        }
    }
}
