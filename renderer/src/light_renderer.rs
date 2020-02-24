use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
}

pub struct Parameters {
    pub main_resources_index: usize,
}

glsl_defines!(fixed_header {
    bindings: {
        LIGHT_BUFFER_BINDING = 4;
    },
    uniforms: {
        WLD_TO_CAM_LOC = 0;
        CAM_TO_CLP_LOC = 1;
    },
});

impl Context<'_> {
    pub fn render_lights(&mut self, params: &Parameters) {
        let Context {
            ref gl,
            ref mut light_renderer,
            ..
        } = *self;

        unsafe {
            light_renderer.program.update(&mut rendering_context!(self));
            if let ProgramName::Linked(program) = light_renderer.program.name {
                let light_resources = &mut self.light_resources;

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    LIGHT_BUFFER_BINDING,
                    light_resources.buffer_ring[self.frame_index.to_usize()].name(),
                );

                let camera = &self.main_resources_pool[params.main_resources_index].camera;

                gl.bind_vertex_array(self.resources.quad_vao);
                gl.use_program(program);
                gl.uniform_matrix4f(
                    WLD_TO_CAM_LOC,
                    gl::MajorAxis::Column,
                    camera.wld_to_cam.cast().unwrap().as_ref(),
                );
                gl.uniform_matrix4f(
                    CAM_TO_CLP_LOC,
                    gl::MajorAxis::Column,
                    camera.cam_to_clp.cast().unwrap().as_ref(),
                );

                gl.depth_mask(gl::FALSE);
                gl.enable(gl::BLEND);
                gl.blend_func(gl::SRC_ALPHA, gl::ONE);

                gl.draw_elements_instanced_base_vertex(
                    gl::TRIANGLES,
                    6,
                    gl::UNSIGNED_INT,
                    0,
                    light_resources.header.light_count as u32,
                    0,
                );

                gl.depth_mask(gl::TRUE);
                gl.disable(gl::BLEND);

                gl.unbind_vertex_array();
                gl.unuse_program();
            }
        }
    }
}

impl Renderer {
    pub fn new(context: &mut RenderingContext) -> Self {
        Renderer {
            program: vs_fs_program(context, "light_renderer.vert", "light_renderer.frag", fixed_header()),
        }
    }
}
