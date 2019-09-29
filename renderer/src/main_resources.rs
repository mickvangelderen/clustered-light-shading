#![allow(unused)]

use crate::*;

pub struct MainResources {
    pub dims: Vector2<i32>,
    // Main frame resources.
    pub framebuffer_name: gl::NonDefaultFramebufferName,
    pub color_texture: Texture<gl::TEXTURE_2D, gl::RGBA16F>,
    pub depth_texture: Texture<gl::TEXTURE_2D, gl::DEPTH24_STENCIL8>,
    pub nor_in_cam_texture: Texture<gl::TEXTURE_2D, gl::R11F_G11F_B10F>,
    // Profiling
    pub depth_pass_profiler: SampleIndex,
    pub basic_pass_profiler: SampleIndex,
}

impl MainResources {
    pub fn new(gl: &gl::Gl, profiling_context: &mut ProfilingContext, dims: Vector2<i32>) -> Self {
        unsafe {
            // Textures.
            let texture_update = TextureUpdate::new()
                .data(dims.x, dims.y, None)
                .min_filter(gl::NEAREST.into())
                .mag_filter(gl::NEAREST.into())
                .max_level(0)
                .wrap_s(gl::CLAMP_TO_EDGE.into())
                .wrap_t(gl::CLAMP_TO_EDGE.into());

            let color_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RGBA16F);
            color_texture.update(gl, texture_update);

            let nor_in_cam_texture = Texture::new(gl, gl::TEXTURE_2D, gl::R11F_G11F_B10F);
            nor_in_cam_texture.update(gl, texture_update);

            let depth_texture = Texture::new(gl, gl::TEXTURE_2D, gl::DEPTH24_STENCIL8);
            depth_texture.update(gl, texture_update);

            // Framebuffers.

            let framebuffer_name = create_framebuffer!(
                gl,
                (gl::DEPTH_STENCIL_ATTACHMENT, depth_texture.name()),
                (gl::COLOR_ATTACHMENT0, color_texture.name()),
                (gl::COLOR_ATTACHMENT1, nor_in_cam_texture.name()),
            );

            // Uniform block buffers,

            MainResources {
                dims,
                framebuffer_name,
                color_texture,
                depth_texture,
                nor_in_cam_texture,
                depth_pass_profiler: profiling_context.add_sample("main_depth"),
                basic_pass_profiler: profiling_context.add_sample("main_basic"),
            }
        }
    }

    pub fn reset(&mut self, gl: &gl::Gl, _profiling_context: &mut ProfilingContext, dims: Vector2<i32>) {
        if self.dims != dims {
            self.dims = dims;

            let texture_update = TextureUpdate::new().data(dims.x, dims.y, None);
            self.color_texture.update(gl, texture_update);
            self.depth_texture.update(gl, texture_update);
            self.nor_in_cam_texture.update(gl, texture_update);
        }
    }

    pub fn drop(self, gl: &gl::Gl) {
        unsafe {
            gl.delete_framebuffer(self.framebuffer_name);
            self.color_texture.drop(gl);
            self.depth_texture.drop(gl);
            self.nor_in_cam_texture.drop(gl);
        }
    }
}

impl_frame_pool! {
    MainResourcesPool,
    MainResources,
    MainResourcesIndex,
    MainResourcesIndexIter,
    (gl: &gl::Gl, profiling_context: &mut ProfilingContext, dims: Vector2<i32>),
}
