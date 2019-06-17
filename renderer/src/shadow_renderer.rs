use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
}

pub struct Parameters {
    pub viewport: Viewport<i32>,
    pub framebuffer: gl::FramebufferName,
}

impl Renderer {
    pub fn render(&mut self, gl: &gl::Gl, params: &Parameters, world: &mut World, resources: &Resources) {
        unsafe {
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            params.viewport.set(gl);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);

            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            // Reverse-Z.
            gl.clear_depth(0.0);
            gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);

            self.update(gl, world);
            if let ProgramName::Linked(name) = self.program.name(&world.global) {
                gl.use_program(*name);

                gl.bind_vertex_array(resources.scene_vao);

                for (i, mesh_meta) in resources.mesh_metas.iter().enumerate() {
                    if let Some(loc) = self.pos_from_obj_to_wld_loc.into() {
                        let pos_from_obj_to_wld = Matrix4::from_translation(resources.meshes[i].translate);

                        gl.uniform_matrix4f(loc, gl::MajorAxis::Column, pos_from_obj_to_wld.as_ref());
                    }

                    gl.draw_elements_base_vertex(
                        gl::TRIANGLES,
                        mesh_meta.element_count,
                        gl::UNSIGNED_INT,
                        mesh_meta.element_offset,
                        mesh_meta.vertex_base,
                    );
                }

                gl.unbind_vertex_array();

                gl.unuse_program();
            }
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, world: &mut World) {
        let modified = self.program.modified();
        if modified < self.program.update(gl, world) {
            if let ProgramName::Linked(name) = self.program.name(&world.global) {
                unsafe {
                    self.pos_from_obj_to_wld_loc = get_uniform_location!(gl, *name, "pos_from_obj_to_wld");
                }
            }
        }
    }

    pub fn new(gl: &gl::Gl, vertex_source_indices: Vec<usize>, fragment_source_indices: Vec<usize>) -> Self {
        Renderer {
            program: Program::new(gl, vertex_source_indices, fragment_source_indices),
            pos_from_obj_to_wld_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
