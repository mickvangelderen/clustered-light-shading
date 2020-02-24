#![allow(unused)]

use crate::*;

pub struct MainFramebuffer {
    pub dimensions: Vector2<i32>,
    pub sample_count: u32,

    // Main frame resources.
    pub framebuffer_name: gl::NonDefaultFramebufferName,
    pub color_texture_name: gl::TextureName,
    pub depth_texture_name: gl::TextureName,
}

impl MainFramebuffer {
    const DEPTH_FORMAT: gl::symbols::DEPTH32F_STENCIL8 = gl::DEPTH32F_STENCIL8;
    const COLOR_FORMAT: gl::symbols::RGBA16F = gl::RGBA16F;

    pub fn new(gl: &gl::Gl, dimensions: Vector2<i32>, sample_count: u32) -> Self {
        unsafe {
            let framebuffer_name = gl.create_framebuffer();

            let depth_texture_name = create_texture(gl, Self::DEPTH_FORMAT, dimensions, sample_count);
            let color_texture_name = create_texture(gl, Self::COLOR_FORMAT, dimensions, sample_count);

            gl.named_framebuffer_texture(framebuffer_name, gl::DEPTH_STENCIL_ATTACHMENT, depth_texture_name, 0);
            gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT0, color_texture_name, 0);
            gl.named_framebuffer_draw_buffers(framebuffer_name, &[gl::COLOR_ATTACHMENT0.into()]);

            assert_eq!(
                gl::FramebufferStatus::from(gl::FRAMEBUFFER_COMPLETE),
                gl.check_named_framebuffer_status(framebuffer_name, gl::FRAMEBUFFER),
            );

            Self {
                dimensions,
                sample_count,

                framebuffer_name,
                color_texture_name,
                depth_texture_name,
            }
        }
    }

    pub fn reconcile(&mut self, gl: &gl::Gl, dimensions: Vector2<i32>, sample_count: u32) {
        if self.dimensions != dimensions || self.sample_count != sample_count {
            std::mem::replace(self, MainFramebuffer::new(gl, dimensions, sample_count)).drop(gl);
        }
    }

    pub fn drop(mut self, gl: &gl::Gl) {
        unsafe {
            gl.delete_framebuffer(self.framebuffer_name);
            gl.delete_texture(self.depth_texture_name);
            gl.delete_texture(self.color_texture_name);
        }
    }
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

pub struct MainProfilers {
    pub depth_profiler: SampleIndex,
    pub depth_opaque_profiler: SampleIndex,
    pub depth_masked_profiler: SampleIndex,
    pub basic_profiler: SampleIndex,
    pub basic_opaque_profiler: SampleIndex,
    pub basic_masked_profiler: SampleIndex,
    pub basic_transparent_profiler: SampleIndex,
}

impl MainProfilers {
    pub fn new(profiling_context: &mut profiling::ProfilingContext) -> Self {
        Self {
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

pub struct MainResourcesParameters<'a> {
    pub gl: &'a gl::Gl,
    pub profiling_context: &'a mut profiling::ProfilingContext,
    pub camera: CameraParameters,
    pub draw_resources_index: usize,
    pub cluster_resources_index: Option<ClusterResourcesIndex>,
    pub dimensions: Vector2<i32>,
    pub sample_count: u32,
    pub display_viewport: Option<Viewport<i32>>,
    pub should_render: bool,
}

pub struct MainResources {
    pub camera: CameraParameters,
    pub draw_resources_index: usize,
    pub cluster_resources_index: Option<ClusterResourcesIndex>,
    pub display_viewport: Option<Viewport<i32>>,
    pub depth_available: bool,
    pub should_render: bool,
    pub framebuffer: MainFramebuffer,
    pub profilers: MainProfilers,
}

impl MainResources {
    pub fn new(parameters: MainResourcesParameters) -> Self {
        let MainResourcesParameters {
            gl,
            profiling_context,
            camera,
            draw_resources_index,
            cluster_resources_index,
            dimensions,
            sample_count,
            display_viewport,
            should_render,
        } = parameters;
        Self {
            camera,
            draw_resources_index,
            cluster_resources_index,
            display_viewport,
            should_render,
            depth_available: false,
            framebuffer: MainFramebuffer::new(gl, dimensions, sample_count),
            profilers: MainProfilers::new(profiling_context),
        }
    }

    pub fn reconcile(&mut self, parameters: MainResourcesParameters) {
        let MainResourcesParameters {
            gl,
            profiling_context,
            camera,
            draw_resources_index,
            cluster_resources_index,
            dimensions,
            sample_count,
            display_viewport,
            should_render,
        } = parameters;
        self.camera = camera;
        self.draw_resources_index = draw_resources_index;
        self.cluster_resources_index = cluster_resources_index;
        self.display_viewport = display_viewport;
        self.depth_available = false;
        self.should_render = should_render;
        self.framebuffer.reconcile(gl, dimensions, sample_count);
    }
}
