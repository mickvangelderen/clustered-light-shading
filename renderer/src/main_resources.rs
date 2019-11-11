#![allow(unused)]

use crate::*;

pub struct MainResources {
    pub dims: Vector2<i32>,
    pub sample_count :u32,

    // Main frame resources.
    pub framebuffer_name: gl::NonDefaultFramebufferName,
    pub color_texture: gl::TextureName,
    pub depth_texture: gl::TextureName,

    // Profiling
    pub depth_pass_profiler: SampleIndex,
    pub basic_pass_profiler: SampleIndex,
}

impl MainResources {
    pub fn new(gl: &gl::Gl, profiling_context: &mut ProfilingContext, dims: Vector2<i32>, sample_count: u32) -> Self {
        unsafe {
            // Textures.
            let texture_update = TextureUpdate::new()
                .data(dims.x, dims.y, None)
                .min_filter(gl::NEAREST.into())
                .mag_filter(gl::NEAREST.into())
                .max_level(0)
                .wrap_s(gl::CLAMP_TO_EDGE.into())
                .wrap_t(gl::CLAMP_TO_EDGE.into());

            let color_texture = gl.create_texture(gl::TEXTURE_2D_MULTISAMPLE);
            gl.bind_texture(gl::TEXTURE_2D_MULTISAMPLE, color_texture);
            gl.tex_image_2d_multisample(gl::TEXTURE_2D_MULTISAMPLE, sample_count as i32, gl::RGBA16F, dims.x, dims.y, false);

            let depth_texture = gl.create_texture(gl::TEXTURE_2D_MULTISAMPLE);
            gl.bind_texture(gl::TEXTURE_2D_MULTISAMPLE, depth_texture);
            gl.tex_image_2d_multisample(gl::TEXTURE_2D_MULTISAMPLE, sample_count as i32, gl::DEPTH24_STENCIL8, dims.x, dims.y, false);

            // Framebuffers.

            let framebuffer_name = create_framebuffer!(
                gl,
                (gl::DEPTH_STENCIL_ATTACHMENT, depth_texture),
                (gl::COLOR_ATTACHMENT0, color_texture),
            );

            MainResources {
                dims,
                sample_count,

                framebuffer_name,
                color_texture,
                depth_texture,

                depth_pass_profiler: profiling_context.add_sample("main_depth"),
                basic_pass_profiler: profiling_context.add_sample("main_basic"),
            }
        }
    }

    pub fn reset(&mut self, gl: &gl::Gl, _profiling_context: &mut ProfilingContext, dims: Vector2<i32>, sample_count: u32) {
        if self.dims != dims || self.sample_count != sample_count {
            self.dims = dims;
            self.sample_count = sample_count;

            unsafe {
                gl.bind_texture(gl::TEXTURE_2D_MULTISAMPLE, self.color_texture);
                gl.tex_image_2d_multisample(gl::TEXTURE_2D_MULTISAMPLE, sample_count as i32, gl::RGBA16F, dims.x, dims.y, false);

                gl.bind_texture(gl::TEXTURE_2D_MULTISAMPLE, self.depth_texture);
                gl.tex_image_2d_multisample(gl::TEXTURE_2D_MULTISAMPLE, sample_count as i32, gl::DEPTH24_STENCIL8, dims.x, dims.y, false);
            }
        }
    }

    pub fn drop(self, gl: &gl::Gl) {
        unsafe {
            gl.delete_framebuffer(self.framebuffer_name);
            gl.delete_texture(self.color_texture);
            gl.delete_texture(self.depth_texture);
        }
    }
}

impl_frame_pool! {
    MainResourcesPool,
    MainResources,
    MainResourcesIndex,
    MainResourcesIndexIter,
    (gl: &gl::Gl, profiling_context: &mut ProfilingContext, dims: Vector2<i32>, sample_count: u32),
}
