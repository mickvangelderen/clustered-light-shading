use crate::*;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct AttenCoefs<S> {
    pub i: S,
    pub i0: S,
    pub r0: S,
    pub r1: S,
}

impl<S> AttenCoefs<S>
where
    S: num_traits::Float,
{
    pub fn cast<U>(self) -> Option<AttenCoefs<U>>
    where
        U: num_traits::Float,
    {
        Some(AttenCoefs {
            i: num_traits::cast(self.i)?,
            i0: num_traits::cast(self.i0)?,
            r0: num_traits::cast(self.r0)?,
            r1: num_traits::cast(self.r1)?,
        })
    }
}

impl From<configuration::Attenuation> for AttenCoefs<f64> {
    fn from(value: configuration::Attenuation) -> Self {
        let configuration::Attenuation { i, i0, r0 } = value;

        AttenCoefs {
            i,
            i0,
            r0,
            r1: f64::sqrt(i / i0),
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
    pub framebuffer: gl::NonDefaultFramebufferName,
    pub depth_texture: gl::TextureName,
    pub distance_texture: gl::TextureName,
}

impl LightResources {
    pub fn new(gl: &gl::Gl, profiling_context: &mut profiling::ProfilingContext) -> Self {
        unsafe {
            let framebuffer = gl.create_framebuffer();

            let depth_texture = {
                let name = gl.create_texture(gl::TEXTURE_CUBE_MAP);
                gl.texture_storage_2d(name, 1, gl::DEPTH_COMPONENT32, 256, 256);
                gl.texture_parameteri(name, gl::TEXTURE_MAX_LEVEL, 0u32);
                gl.texture_parameteri(name, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
                gl.texture_parameteri(name, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
                name
            };

            let distance_texture = {
                let name = gl.create_texture(gl::TEXTURE_CUBE_MAP);
                gl.texture_storage_2d(name, 1, gl::R16, 256, 256);
                gl.texture_parameteri(name, gl::TEXTURE_MAX_LEVEL, 0u32);
                gl.texture_parameteri(name, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
                gl.texture_parameteri(name, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
                name
            };

            gl.named_framebuffer_texture(framebuffer, gl::DEPTH_ATTACHMENT, depth_texture, 0);
            gl.named_framebuffer_texture(framebuffer, gl::COLOR_ATTACHMENT0, distance_texture, 0);
            gl.named_framebuffer_draw_buffers(framebuffer, &[gl::COLOR_ATTACHMENT0.into()]);

            Self {
                buffer_ring: Ring3::new(|| StorageBuffer::new(gl)),
                header: Default::default(),
                body: Default::default(),
                sample_indices: LightSampleIndices::new(profiling_context),
                framebuffer,
                depth_texture,
                distance_texture,
            }
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
