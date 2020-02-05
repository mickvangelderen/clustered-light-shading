#![allow(unused)]

use crate::*;

pub struct MainResources {
    pub dimensions: Vector2<i32>,
    pub sample_count: u32,

    // Main frame resources.
    pub framebuffer_name: gl::NonDefaultFramebufferName,
    pub color_texture: gl::TextureName,
    pub depth_texture: gl::TextureName,

    // Profiling
    pub depth_profiler: SampleIndex,
    pub depth_opaque_profiler: SampleIndex,
    pub depth_masked_profiler: SampleIndex,
    pub basic_profiler: SampleIndex,
    pub basic_opaque_profiler: SampleIndex,
    pub basic_masked_profiler: SampleIndex,
    pub basic_transparent_profiler: SampleIndex,
}

unsafe fn create_texture(
    gl: &gl::Gl,
    format: impl Into<gl::InternalFormat>,
    dimensions: Vector2<i32>,
    sample_count: u32,
) -> gl::TextureName {
    let name = if sample_count == 0 {
        let name = gl.create_texture(gl::TEXTURE_2D);
        gl.texture_storage_2d(name, 1, format, dimensions.x, dimensions.y);
        name
    } else {
        let name = gl.create_texture(gl::TEXTURE_2D_MULTISAMPLE);
        gl.bind_texture(gl::TEXTURE_2D_MULTISAMPLE, name);
        gl.tex_image_2d_multisample(
            gl::TEXTURE_2D_MULTISAMPLE,
            sample_count as i32,
            format,
            dimensions.x,
            dimensions.y,
            false,
        );
        name
    };

    gl.texture_parameteri(name, gl::TEXTURE_MAX_LEVEL, 0u32);
    gl.texture_parameteri(name, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
    gl.texture_parameteri(name, gl::TEXTURE_MAG_FILTER, gl::NEAREST);

    name
}

const DEPTH_FORMAT: gl::symbols::DEPTH32F_STENCIL8 = gl::DEPTH32F_STENCIL8;
const COLOR_FORMAT: gl::symbols::RGBA16F = gl::RGBA16F;

impl MainResources {
    pub fn new(
        gl: &gl::Gl,
        profiling_context: &mut ProfilingContext,
        dimensions: Vector2<i32>,
        sample_count: u32,
    ) -> Self {
        unsafe {
            let framebuffer_name = gl.create_framebuffer();

            let depth_texture = create_texture(gl, DEPTH_FORMAT, dimensions, sample_count);
            let color_texture = create_texture(gl, COLOR_FORMAT, dimensions, sample_count);

            gl.named_framebuffer_texture(framebuffer_name, gl::DEPTH_STENCIL_ATTACHMENT, depth_texture, 0);
            gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT0, color_texture, 0);
            gl.named_framebuffer_draw_buffers(framebuffer_name, &[gl::COLOR_ATTACHMENT0.into()]);

            MainResources {
                dimensions,
                sample_count,

                framebuffer_name,
                color_texture,
                depth_texture,

                depth_profiler: profiling_context.add_sample("depth"),
                depth_opaque_profiler: profiling_context.add_sample("opaque"),
                depth_masked_profiler: profiling_context.add_sample("masked"),

                basic_profiler: profiling_context.add_sample("basic"),
                basic_opaque_profiler: profiling_context.add_sample("opaque"),
                basic_masked_profiler: profiling_context.add_sample("masked"),
                basic_transparent_profiler: profiling_context.add_sample("transparent"),
            }
        }
    }

    pub fn reset(
        &mut self,
        gl: &gl::Gl,
        _profiling_context: &mut ProfilingContext,
        dimensions: Vector2<i32>,
        sample_count: u32,
    ) {
        if self.dimensions != dimensions || self.sample_count != sample_count {
            self.dimensions = dimensions;
            self.sample_count = sample_count;

            unsafe {
                let framebuffer_name = gl.create_framebuffer();
                let depth_texture = create_texture(gl, DEPTH_FORMAT, dimensions, sample_count);
                let color_texture = create_texture(gl, COLOR_FORMAT, dimensions, sample_count);

                gl.named_framebuffer_texture(framebuffer_name, gl::DEPTH_STENCIL_ATTACHMENT, depth_texture, 0);
                gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT0, color_texture, 0);

                gl.named_framebuffer_draw_buffers(
                    framebuffer_name,
                    &[gl::COLOR_ATTACHMENT0.into()],
                );

                gl.delete_framebuffer(std::mem::replace(&mut self.framebuffer_name, framebuffer_name));
                gl.delete_texture(std::mem::replace(&mut self.depth_texture, depth_texture));
                gl.delete_texture(std::mem::replace(&mut self.color_texture, color_texture));
            }
        }
    }

    pub fn drop(mut self, gl: &gl::Gl) {
        unsafe {
            gl.delete_framebuffer(self.framebuffer_name);
            gl.delete_texture(self.depth_texture);
            gl.delete_texture(self.color_texture);
        }
    }
}

impl_frame_pool! {
    MainResourcesPool,
    MainResources,
    MainResourcesIndex,
    MainResourcesIndexIter,
    (gl: &gl::Gl, profiling_context: &mut ProfilingContext, dimensions: Vector2<i32>, sample_count: u32),
}
