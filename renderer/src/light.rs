use cgmath::*;
use crate::rendering;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct RGB<S> {
    pub r: S,
    pub g: S,
    pub b: S,
}

impl<S> RGB<S> {
    pub fn new(r: S, g: S, b: S) -> Self {
        RGB { r, g, b }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct AttenCoefs<S> {
    pub constant: S,
    pub linear: S,
    pub quadratic: S,
}

#[derive(Debug, Copy, Clone)]
pub struct PointLight {
    pub ambient: RGB<f32>,
    pub diffuse: RGB<f32>,
    pub specular: RGB<f32>,
    pub pos_in_pnt: Point3<f32>,
    pub attenuation: AttenCoefs<f32>,
    pub radius: f32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct PointLightBufferEntry {
    ambient: RGB<f32>,
    _pad0: f32,
    diffuse: RGB<f32>,
    _pad1: f32,
    specular: RGB<f32>,
    _pad2: f32,
    pos_in_cam: Point3<f32>,
    _pad3: f32,
    attenuation: AttenCoefs<f32>,
    radius: f32,
}

impl PointLightBufferEntry {
    pub fn from_point_light(point_light: PointLight, pos_from_pnt_to_cam: Matrix4<f32>) -> Self {
        PointLightBufferEntry {
            ambient: point_light.ambient,
            _pad0: 0.0,
            diffuse: point_light.diffuse,
            _pad1: 0.0,
            specular: point_light.specular,
            _pad2: 0.0,
            pos_in_cam: pos_from_pnt_to_cam.transform_point(point_light.pos_in_pnt),
            _pad3: 0.0,
            attenuation: point_light.attenuation,
            radius: point_light.radius,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct LightingBuffer {
    pub point_lights: [PointLightBufferEntry; rendering::POINT_LIGHT_CAPACITY as usize],
}

impl AsRef<[u8; std::mem::size_of::<LightingBuffer>()]> for LightingBuffer {
    fn as_ref(&self) -> &[u8; std::mem::size_of::<LightingBuffer>()] {
        unsafe {
            &*(self as *const Self as *const _)
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct CLSBufferHeader {
    pub cluster_dims: Vector4<u32>,
}

impl AsRef<[u8; std::mem::size_of::<CLSBufferHeader>()]> for CLSBufferHeader {
    fn as_ref(&self) -> &[u8; std::mem::size_of::<CLSBufferHeader>()] {
        unsafe {
            &*(self as *const Self as *const _)
        }
    }
}
