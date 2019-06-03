use crate::resources::Resources;
use crate::*;
use cgmath::*;
use gl_typed as gl;

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub light_count_loc: gl::OptionUniformLocation,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
}

pub struct Parameters<'a> {
    pub framebuffer: gl::FramebufferName,
    pub width: i32,
    pub height: i32,
    pub cls_buffer: &'a rendering::CLSBuffer,
    pub min_light_count: u32,
    pub animate_z: Option<f32>,
}

impl Renderer {
    pub fn render(&self, gl: &gl::Gl, params: &Parameters, world: &World, resources: &Resources) {
        unsafe {
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.viewport(0, 0, params.width, params.height);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);

            gl.clear_color(world.clear_color[0], world.clear_color[1], world.clear_color[2], 1.0);
            // Reverse-Z projection.
            gl.clear_depth(0.0);
            gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);

            gl.use_program(self.program.name);
            gl.bind_vertex_array(resources.cluster_vao);

            let [xn, yn, zn, _wn]: [u32; 4] = params.cls_buffer.header.dimensions.into();
            for zi in 0..zn {
                if let Some(animate_z) = params.animate_z {
                    if zi != (((world.tick as f64 / DESIRED_UPS) * animate_z as f64) as u64 % zn as u64) as u32 {
                        continue;
                    }
                }
                for yi in 0..yn {
                    for xi in 0..xn {
                        let i = ((zi * yn) + yi) * xn + xi;
                        let cluster = &params.cls_buffer.body[i as usize];
                        let light_count = cluster[0];
                        if light_count >= params.min_light_count {
                            if let Some(loc) = self.light_count_loc.into() {
                                gl.uniform_1ui(loc, light_count);
                            }

                            if let Some(loc) = self.pos_from_obj_to_wld_loc.into() {
                                let pos_from_obj_to_cls =
                                    Matrix4::from_translation(Vector3::new(xi as f32, yi as f32, zi as f32));

                                let pos_from_obj_to_wld =
                                    params.cls_buffer.header.pos_from_cls_to_wld * pos_from_obj_to_cls;
                                gl.uniform_matrix4f(loc, gl::MajorAxis::Column, pos_from_obj_to_wld.as_ref());
                            }

                            gl.draw_elements(gl::TRIANGLES, resources.cluster_element_count, gl::UNSIGNED_INT, 0);
                        }
                    }
                }
            }

            gl.unbind_vertex_array();
            gl.unuse_program();
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, update: &rendering::VSFSProgramUpdate) {
        unsafe {
            if self.program.update(gl, update) {
                gl.use_program(self.program.name);

                self.light_count_loc = get_uniform_location!(gl, self.program.name, "light_count");
                self.pos_from_obj_to_wld_loc = get_uniform_location!(gl, self.program.name, "pos_from_obj_to_wld");

                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        Renderer {
            program: rendering::VSFSProgram::new(gl),
            light_count_loc: gl::OptionUniformLocation::NONE,
            pos_from_obj_to_wld_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
