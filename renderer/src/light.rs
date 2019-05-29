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

pub struct AttenParams<S> {
    pub intensity: S,
    pub clip_near: S,
    pub cutoff: S,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct AttenCoefs<S> {
    pub inv_quadratic: S,
    pub constant: S,
    pub linear: S,
    pub clip_near: S,
    pub clip_far: S,
    pub _pad: [S; 3],
}

impl<S> From<AttenParams<S>> for AttenCoefs<S> where S: num_traits::Float {
    fn from(value: AttenParams<S>) -> Self {
        let AttenParams { intensity, clip_near, cutoff } = value;

        let n3 = S::from(-3.0).unwrap();
        let p2 = S::from(2.0).unwrap();

        AttenCoefs {
            inv_quadratic: intensity,
            constant: n3 * cutoff,
            linear: p2 * S::sqrt(S::powi(cutoff, 3)/intensity),
            clip_near,
            clip_far: S::sqrt(intensity/cutoff),
            _pad: [S::zero(), S::zero(), S::zero()],
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PointLight {
    pub ambient: RGB<f32>,
    pub diffuse: RGB<f32>,
    pub specular: RGB<f32>,
    pub pos_in_pnt: Point3<f32>,
    pub attenuation: AttenCoefs<f32>,
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
        }
    }
}


// FIXME: Align only necessary on uniform blocks?
#[derive(Debug)]
#[repr(C, align(256))]
pub struct LightingBuffer {
    pub point_lights: [PointLightBufferEntry; rendering::POINT_LIGHT_CAPACITY as usize],
}
