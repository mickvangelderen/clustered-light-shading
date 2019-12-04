use crate::*;

pub struct AttenParams<S> {
    pub i: S,
    pub i0: S,
    pub r0: S,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct AttenCoefs<S> {
    pub i: S,
    pub i0: S,
    pub r0: S,
    pub r1: S,
}

impl<S> From<AttenParams<S>> for AttenCoefs<S>
where
    S: num_traits::Float,
{
    fn from(value: AttenParams<S>) -> Self {
        let AttenParams { i, i0, r0 } = value;

        AttenCoefs {
            i,
            i0,
            r0,
            r1: S::sqrt(i / i0),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PointLight {
    pub tint: [f32; 3],
    pub pos_in_wld: Point3<f32>,
    pub attenuation: AttenCoefs<f32>,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct LightBufferLight {
    pub tint: [f32; 3],
    pub _pad0: f32,

    pub pos_in_lgt: [f32; 3],
    pub _pad1: f32,

    pub attenuation: AttenCoefs<f32>,
}

impl LightBufferLight {
    pub fn from_point_light(point_light: PointLight, pos_from_wld_to_lgt: Matrix4<f64>) -> Self {
        Self {
            tint: point_light.tint,
            _pad0: 0.0,

            pos_in_lgt: pos_from_wld_to_lgt
                .transform_point(point_light.pos_in_wld.cast().unwrap())
                .cast()
                .unwrap()
                .into(),
            _pad1: 1.0,

            attenuation: point_light.attenuation,
        }
    }
}

#[repr(C)]
pub struct LightBufferHeader {
    pub wld_to_lgt: Matrix4<f32>,
    pub lgt_to_wld: Matrix4<f32>,

    pub light_count: Vector4<u32>,
}

pub const LIGHT_BUFFER_DECLARATION: &'static str = r"
";

pub struct LightSampleIndices {
    pub total: profiling::SampleIndex,
    pub compute: profiling::SampleIndex,
    pub upload: profiling::SampleIndex,
}

impl LightSampleIndices {
    pub fn new(profiling_context: &mut profiling::ProfilingContext) -> Self {
        Self {
            total: profiling_context.add_sample("lights"),
            compute: profiling_context.add_sample("compute"),
            upload: profiling_context.add_sample("upload"),
        }
    }
}

pub struct LightResources {
    pub buffer_ring: Ring3<StorageBuffer<StorageBufferWrite>>,
    pub lights: Vec<LightBufferLight>,
    pub dirty: bool,
    pub sample_indices: LightSampleIndices,
}

impl LightResources {
    pub fn new(gl: &gl::Gl, profiling_context: &mut profiling::ProfilingContext) -> Self {
        Self {
            buffer_ring: Ring3::new(|| StorageBuffer::new(gl, StorageBufferWrite)),
            lights: Vec::new(),
            dirty: true,
            sample_indices: LightSampleIndices::new(profiling_context),
        }
    }
}

pub struct LightParameters {
    pub wld_to_lgt: Matrix4<f64>,
    pub lgt_to_wld: Matrix4<f64>,
}
