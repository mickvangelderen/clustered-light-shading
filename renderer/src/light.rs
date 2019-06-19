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
    pub cutoff: S,
    pub clip_near: S,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct AttenCoefs<S> {
    pub intensity: S,
    pub cutoff: S,
    pub clip_near: S,
    pub clip_far: S,
}

impl<S> From<AttenParams<S>> for AttenCoefs<S> where S: num_traits::Float {
    fn from(value: AttenParams<S>) -> Self {
        let AttenParams { intensity, cutoff, clip_near } = value;

        AttenCoefs {
            intensity,
            cutoff,
            clip_near,
            clip_far: S::sqrt(intensity/cutoff),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PointLight {
    pub ambient: RGB<f32>,
    pub diffuse: RGB<f32>,
    pub specular: RGB<f32>,
    pub pos_in_wld: Point3<f32>,
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
    pos_in_lgt: Point3<f32>,
    _pad3: f32,
    attenuation: AttenCoefs<f32>,
}

impl PointLightBufferEntry {
    pub fn from_point_light(point_light: PointLight, pos_from_wld_to_lgt: Matrix4<f32>) -> Self {
        PointLightBufferEntry {
            ambient: point_light.ambient,
            _pad0: 0.0,
            diffuse: point_light.diffuse,
            _pad1: 0.0,
            specular: point_light.specular,
            _pad2: 0.0,
            pos_in_lgt: pos_from_wld_to_lgt.transform_point(point_light.pos_in_wld),
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
