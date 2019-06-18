use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
}

impl Renderer {
    pub fn render(&mut self, gl: &gl::Gl, world: &mut World, resources: &Resources) {
        unsafe {
            self.update(gl, world);
            if let ProgramName::Linked(name) = self.program.name(&world.global) {
                gl.use_program(*name);
                gl.bind_vertex_array(resources.scene_pos_vao);

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

    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        Renderer {
            program: Program::new(
                gl,
                vec![world.add_source("depth_renderer.vert")],
                vec![world.add_source("depth_renderer.frag")],
            ),
            pos_from_obj_to_wld_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
