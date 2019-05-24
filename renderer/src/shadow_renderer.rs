use crate::rendering;
use crate::resources::Resources;
use cgmath::*;
use gl_typed as gl;

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
}

pub struct Parameters {
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
}

impl Renderer {
    pub fn render(&self, gl: &gl::Gl, params: &Parameters, resources: &Resources) {
        unsafe {
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.viewport(0, 0, params.width, params.height);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);
            gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into()]);

            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            // Reverse-Z.
            gl.clear_depth(0.0);
            gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);

            gl.use_program(self.program.name);

            for i in 0..resources.vaos.len() {
                if let Some(loc) = self.pos_from_obj_to_wld_loc.into() {
                    let pos_from_obj_to_wld = Matrix4::from_translation(resources.meshes[i].translate);

                    gl.uniform_matrix4f(loc, gl::MajorAxis::Column, pos_from_obj_to_wld.as_ref());
                }

                gl.bind_vertex_array(resources.vaos[i]);
                gl.draw_elements(gl::TRIANGLES, resources.element_counts[i], gl::UNSIGNED_INT, 0);
            }

            gl.unbind_vertex_array();

            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            gl.unuse_program();
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, update: &rendering::VSFSProgramUpdate) {
        unsafe {
            if self.program.update(gl, update) {
                gl.use_program(self.program.name);

                self.pos_from_obj_to_wld_loc = get_uniform_location!(gl, self.program.name, "pos_from_obj_to_wld");

                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            Renderer {
                program: rendering::VSFSProgram::new(gl),
                pos_from_obj_to_wld_loc: gl::OptionUniformLocation::NONE,
            }
        }
    }
}
