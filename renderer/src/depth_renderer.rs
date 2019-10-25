use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
    pub obj_to_wld_loc: gl::OptionUniformLocation,
}

impl Context<'_> {
    pub fn render_depth(&mut self) {
        let Context {
            ref gl,
            ref resources,
            ref mut depth_renderer,
            ..
        } = *self;
        unsafe {
            depth_renderer.update(&mut rendering_context!(self));
            if let ProgramName::Linked(name) = depth_renderer.program.name {
                gl.use_program(name);
                gl.bind_vertex_array(resources.scene_vao);

                for mesh_description in resources.scene_file.mesh_descriptions.iter() {
                    if let Some(loc) = depth_renderer.obj_to_wld_loc.into() {
                        let obj_to_wld = Matrix4::identity();
                        gl.uniform_matrix4f(loc, gl::MajorAxis::Column, obj_to_wld.as_ref());
                    }

                    gl.draw_elements_base_vertex(
                        gl::TRIANGLES,
                        mesh_description.element_count,
                        gl::UNSIGNED_INT,
                        mesh_description.index_byte_offset as usize,
                        mesh_description.vertex_offset,
                    );
                }

                gl.unbind_vertex_array();
                gl.unuse_program();
            }
        }
    }
}

impl Renderer {
    pub fn update(&mut self, context: &mut RenderingContext) {
        if self.program.update(context) {
            let gl = &context.gl;
            if let ProgramName::Linked(name) = self.program.name {
                unsafe {
                    self.obj_to_wld_loc = get_uniform_location!(gl, name, "obj_to_wld");
                }
            }
        }
    }

    pub fn new(context: &mut RenderingContext) -> Self {
        Renderer {
            program: vs_fs_program(context, "depth_renderer.vert", "depth_renderer.frag", String::from("// TODO: Pass locations and bindings")),
            obj_to_wld_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
