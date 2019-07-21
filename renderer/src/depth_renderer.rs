use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
    pub obj_to_wld_loc: gl::OptionUniformLocation,
}

impl Renderer {
    pub fn render(&mut self, gl: &gl::Gl, world: &mut World, resources: &Resources) {
        unsafe {
            self.update(gl, world);
            if let ProgramName::Linked(name) = self.program.name {
                gl.use_program(name);
                gl.bind_vertex_array(resources.scene_pos_vao);

                for (i, mesh_meta) in resources.mesh_metas.iter().enumerate() {
                    if let Some(loc) = self.obj_to_wld_loc.into() {
                        let obj_to_wld = Matrix4::from_translation(resources.meshes[i].translate);

                        gl.uniform_matrix4f(loc, gl::MajorAxis::Column, obj_to_wld.as_ref());
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
        if self.program.update(gl, world) {
            if let ProgramName::Linked(name) = self.program.name {
                unsafe {
                    self.obj_to_wld_loc = get_uniform_location!(gl, name, "obj_to_wld");
                }
            }
        }
    }

    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        Renderer {
            program: vs_fs_program(gl, world, "depth_renderer.vert", "depth_renderer.frag"),
            obj_to_wld_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
