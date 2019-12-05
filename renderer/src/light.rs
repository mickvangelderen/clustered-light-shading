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
    pub position: Point3<f32>,
    pub attenuation: AttenCoefs<f32>,
}

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

#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct LightBufferHeader {
    pub light_count: u32,
    pub _pad0: [u32; 15],
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct LightBufferLight {
    pub tint: [f32; 3],
    pub _pad0: f32,

    pub position: [f32; 3],
    pub _pad1: f32,

    pub attenuation: AttenCoefs<f32>,
}

impl LightBufferLight {
    pub fn from_point_light(point_light: PointLight) -> Self {
        let PointLight {
            tint,
            position,
            attenuation,
        } = point_light;
        Self {
            tint,
            _pad0: 0.0,

            position: position.into(),
            _pad1: 1.0,

            attenuation,
        }
    }
}

pub struct LightResources {
    pub buffer_ring: Ring3<StorageBufferWO>,
    pub sample_indices: LightSampleIndices,
    pub header: LightBufferHeader,
    pub body: Vec<LightBufferLight>,
}

impl LightResources {
    pub fn new(gl: &gl::Gl, profiling_context: &mut profiling::ProfilingContext) -> Self {
        Self {
            buffer_ring: Ring3::new(|| unsafe { StorageBuffer::new(gl) }),
            header: Default::default(),
            body: Default::default(),
            sample_indices: LightSampleIndices::new(profiling_context),
        }
    }

    pub unsafe fn recompute(
        &mut self,
        gl: &gl::Gl,
        profiling_context: &mut profiling::ProfilingContext,
        frame_index: FrameIndex,
        point_lights: &[PointLight],
    ) {
        let profiler_index = profiling_context.start(gl, self.sample_indices.total);

        {
            let profiler_index = profiling_context.start(gl, self.sample_indices.compute);

            self.header = light::LightBufferHeader {
                light_count: std::convert::TryFrom::try_from(point_lights.len()).unwrap(),
                _pad0: Default::default(),
            };

            self.body.clear();
            self.body
                .extend(point_lights.iter().copied().map(LightBufferLight::from_point_light));

            profiling_context.stop(gl, profiler_index);
        }

        {
            let profiler_index = profiling_context.start(gl, self.sample_indices.upload);

            let header_bytes = self.header.value_as_bytes();
            let body_bytes = self.body.vec_as_bytes();
            let total_byte_count = header_bytes.len() + body_bytes.len();

            let buffer = &mut self.buffer_ring[frame_index.to_usize()];

            buffer.reconcile(gl, total_byte_count);
            buffer.write_at(gl, 0, header_bytes);
            buffer.write_at(gl, header_bytes.len(), body_bytes);

            profiling_context.stop(gl, profiler_index);
        }

        profiling_context.stop(gl, profiler_index);
    }
}
