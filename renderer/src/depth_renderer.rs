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

                let scene_file = &self.resources.scene_file;

                for instance in scene_file.instances.iter() {
                    let transform = &scene_file.transforms[instance.transform_index as usize];
                    let mesh_description = &scene_file.mesh_descriptions[instance.mesh_index as usize];

                    if let Some(loc) = depth_renderer.obj_to_wld_loc.into() {
                        let obj_to_wld = Matrix4::from_translation(transform.translation.into())
                            * Matrix4::from(Euler {
                                x: Deg(transform.rotation[0]),
                                y: Deg(transform.rotation[1]),
                                z: Deg(transform.rotation[2]),
                            })
                            * Matrix4::from_nonuniform_scale(
                                transform.scaling[0],
                                transform.scaling[1],
                                transform.scaling[2],
                            );
                        gl.uniform_matrix4f(loc, gl::MajorAxis::Column, obj_to_wld.cast().unwrap().as_ref());
                    }

                    gl.draw_elements_base_vertex(
                        gl::TRIANGLES,
                        mesh_description.element_count(),
                        gl::UNSIGNED_INT,
                        mesh_description.element_byte_offset(),
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
