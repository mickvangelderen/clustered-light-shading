use crate::*;

pub struct MainResources {
    pub dims: Vector2<i32>,
    // Main frame resources.
    pub framebuffer_name: gl::NonDefaultFramebufferName,
    pub color_texture: Texture<gl::TEXTURE_2D, gl::RGBA16F>,
    pub depth_texture: Texture<gl::TEXTURE_2D, gl::DEPTH24_STENCIL8>,
    pub nor_in_cam_texture: Texture<gl::TEXTURE_2D, gl::R11F_G11F_B10F>,
    // Profiling
    pub depth_pass_profiler: Profiler,
    pub basic_pass_profiler: Profiler,
}

impl MainResources {
    pub fn new(gl: &gl::Gl, dims: Vector2<i32>) -> Self {
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
                depth_pass_profiler: Profiler::new(&gl),
                basic_pass_profiler: Profiler::new(&gl),
            }
        }
    }

    pub fn resize(&mut self, gl: &gl::Gl, dims: Vector2<i32>) {
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

pub struct MainResourcesPool {
    resources: Vec<MainResources>,
    used: usize,
}

impl MainResourcesPool {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            used: 0,
        }
    }

    pub fn next_unused(&mut self, gl: &gl::Gl, dims: Vector2<i32>) -> MainResourcesIndex {
        let index = self.used;
        self.used += 1;

        if self.resources.len() < index + 1 {
            self.resources.push(MainResources::new(&gl, dims));
        }

        let resources = &mut self.resources[index];
        resources.resize(&gl, dims);

        MainResourcesIndex(index)
    }

    pub fn used_slice(&self) -> &[MainResources] {
        &self.resources[0..self.used]
    }

    pub fn used_count(&self) -> usize {
        self.used
    }

    pub fn reset(&mut self) {
        self.used = 0;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MainResourcesIndex(pub usize);

impl std::ops::Index<MainResourcesIndex> for MainResourcesPool {
    type Output = MainResources;

    fn index(&self, index: MainResourcesIndex) -> &Self::Output {
        &self.resources[index.0]
    }
}

impl std::ops::IndexMut<MainResourcesIndex> for MainResourcesPool {
    fn index_mut(&mut self, index: MainResourcesIndex) -> &mut Self::Output {
        &mut self.resources[index.0]
    }
}
