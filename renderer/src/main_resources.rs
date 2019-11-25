#![allow(unused)]

use crate::*;

pub struct MainResources {
    pub dimensions: Vector2<i32>,
    pub sample_count: u32,

    // Main frame resources.
    pub framebuffer_name: gl::NonDefaultFramebufferName,
    pub color_texture: gl::TextureName,
    pub depth_texture: gl::TextureName,
    pub cluster_depth_texture: Option<gl::TextureName>,

    // Profiling
    pub depth_pass_profiler: SampleIndex,
    pub basic_pass_profiler: SampleIndex,
}

unsafe fn create_texture(
    gl: &gl::Gl,
    format: impl Into<gl::InternalFormat>,
    dimensions: Vector2<i32>,
    sample_count: u32,
) -> gl::TextureName {
    if sample_count == 0 {
        let color_texture = gl.create_texture(gl::TEXTURE_2D);
        gl.texture_storage_2d(color_texture, 1, format, dimensions.x, dimensions.y);
        color_texture
    } else {
        let color_texture = gl.create_texture(gl::TEXTURE_2D_MULTISAMPLE);
        gl.bind_texture(gl::TEXTURE_2D_MULTISAMPLE, color_texture);
        gl.tex_image_2d_multisample(
            gl::TEXTURE_2D_MULTISAMPLE,
            sample_count as i32,
            format,
            dimensions.x,
            dimensions.y,
            true,
        );
        color_texture
    }
}

impl MainResources {
    pub fn new(
        gl: &gl::Gl,
        profiling_context: &mut ProfilingContext,
        dimensions: Vector2<i32>,
        sample_count: u32,
    ) -> Self {
        unsafe {
            let framebuffer_name = gl.create_framebuffer();

            let depth_texture = create_texture(gl, gl::DEPTH_COMPONENT32F, dimensions, sample_count);
            let color_texture = create_texture(gl, gl::RGBA16F, dimensions, sample_count);
            let cluster_depth_texture = if sample_count > 0 {
                Some(create_texture(gl, gl::R32F, dimensions, sample_count))
            } else {
                None
            };

            gl.named_framebuffer_texture(framebuffer_name, gl::DEPTH_STENCIL_ATTACHMENT, depth_texture, 0);
            gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT0, color_texture, 0);
            if let Some(cluster_depth_texture) = cluster_depth_texture {
                gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT1, cluster_depth_texture, 0);
            }

            let c01 = [gl::COLOR_ATTACHMENT0.into(), gl::COLOR_ATTACHMENT1.into()];
            let c0 = [gl::COLOR_ATTACHMENT0.into()];
            gl.named_framebuffer_draw_buffers(
                framebuffer_name,
                match cluster_depth_texture {
                    Some(_) => &c01,
                    None => &c0,
                },
            );

            MainResources {
                dimensions,
                sample_count,

                framebuffer_name,
                color_texture,
                depth_texture,
                cluster_depth_texture,

                depth_pass_profiler: profiling_context.add_sample("main_depth"),
                basic_pass_profiler: profiling_context.add_sample("main_basic"),
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
                let depth_texture = create_texture(gl, gl::DEPTH_COMPONENT32F, dimensions, sample_count);
                let color_texture = create_texture(gl, gl::RGBA16F, dimensions, sample_count);
                let cluster_depth_texture = if sample_count > 0 {
                    Some(create_texture(gl, gl::R32F, dimensions, sample_count))
                } else {
                    None
                };

                gl.named_framebuffer_texture(framebuffer_name, gl::DEPTH_STENCIL_ATTACHMENT, depth_texture, 0);
                gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT0, color_texture, 0);
                if let Some(cluster_depth_texture) = cluster_depth_texture {
                    gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT1, cluster_depth_texture, 0);
                }

                let c01 = [gl::COLOR_ATTACHMENT0.into(), gl::COLOR_ATTACHMENT1.into()];
                let c0 = [gl::COLOR_ATTACHMENT0.into()];
                gl.named_framebuffer_draw_buffers(
                    framebuffer_name,
                    match cluster_depth_texture {
                        Some(_) => &c01,
                        None => &c0,
                    },
                );

                gl.delete_framebuffer(std::mem::replace(&mut self.framebuffer_name, framebuffer_name));
                gl.delete_texture(std::mem::replace(&mut self.depth_texture, depth_texture));
                gl.delete_texture(std::mem::replace(&mut self.color_texture, color_texture));
                if let Some(cluster_depth_texture) =
                    std::mem::replace(&mut self.cluster_depth_texture, cluster_depth_texture)
                {
                    gl.delete_texture(cluster_depth_texture);
                }
            }
        }
    }

    pub fn drop(mut self, gl: &gl::Gl) {
        unsafe {
            gl.delete_framebuffer(self.framebuffer_name);
            gl.delete_texture(self.depth_texture);
            gl.delete_texture(self.color_texture);
            if let Some(cluster_depth_texture) = std::mem::replace(&mut self.cluster_depth_texture, None) {
                gl.delete_texture(cluster_depth_texture);
            }
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
