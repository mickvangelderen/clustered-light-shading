use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
}

glsl_defines! {
    fixed_header {
        bindings: {
            LIGHT_BUFFER_BINDING = 4;
        },
        uniforms: {
            WLD_TO_CLP_LOC = 0;
            OPACITY_LOC = 1;
        },
    }
}

pub struct Parameters {
    pub main_resources_index: usize,
}

impl Context<'_> {
    pub fn render_light_volumes(&mut self, params: Parameters) {
        unsafe {
            let program = &mut self.light_volume_renderer.program;
            program.update(&mut rendering_context!(self));
            if let ProgramName::Linked(program_name) = program.name {
                self.gl.use_program(program_name);

                self.gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    LIGHT_BUFFER_BINDING,
                    self.light_resources.buffer_ring[self.frame_index.to_usize()].name(),
                );

                let wld_to_clp = &self.main_resources_pool[params.main_resources_index].camera.wld_to_clp;
                self.gl.uniform_matrix4f(
                    WLD_TO_CLP_LOC,
                    gl::MajorAxis::Column,
                    wld_to_clp.cast::<f32>().unwrap().as_ref(),
                );

                self.gl.uniform_1f(
                    OPACITY_LOC,
                    self.configuration.light.volume_opacity,
                );

                self.gl.depth_mask(gl::FALSE);
                self.gl.enable(gl::BLEND);
                self.gl.blend_func(gl::SRC_ALPHA, gl::ONE);
                self.resources.icosphere1280.draw_instances(&self.gl, self.point_lights.len() as u32);
                self.gl.depth_mask(gl::TRUE);
                self.gl.disable(gl::BLEND);

                self.gl.unuse_program();
            }
        }
    }
}

impl Renderer {
    pub fn new(context: &mut RenderingContext) -> Self {
        Renderer {
            program: vs_fs_program(
                context,
                "light_volume_renderer.vert",
                "light_volume_renderer.frag",
                fixed_header(),
            ),
        }
    }
}
