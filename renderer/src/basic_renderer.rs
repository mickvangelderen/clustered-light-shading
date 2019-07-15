use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
    //
    pub obj_to_wld_loc: gl::OptionUniformLocation,

    pub diffuse_sampler_loc: gl::OptionUniformLocation,
    pub normal_sampler_loc: gl::OptionUniformLocation,
    pub specular_sampler_loc: gl::OptionUniformLocation,

    pub diffuse_dimensions_loc: gl::OptionUniformLocation,
    pub normal_dimensions_loc: gl::OptionUniformLocation,
    pub specular_dimensions_loc: gl::OptionUniformLocation,

    pub display_mode_loc: gl::OptionUniformLocation,
}

pub struct Parameters {
    pub mode: u32,
}

impl Renderer {
    pub fn render(&mut self, gl: &gl::Gl, params: &Parameters, world: &mut World, resources: &Resources) {
        unsafe {
            self.update(gl, world);
            if let ProgramName::Linked(program_name) = self.program.name(&world.global) {
                gl.use_program(*program_name);

                if let Some(loc) = self.diffuse_sampler_loc.into() {
                    gl.uniform_1i(loc, 1);
                }

                if let Some(loc) = self.normal_sampler_loc.into() {
                    gl.uniform_1i(loc, 2);
                }

                if let Some(loc) = self.specular_sampler_loc.into() {
                    gl.uniform_1i(loc, 3);
                }

                if let Some(loc) = self.display_mode_loc.into() {
                    gl.uniform_1ui(loc, params.mode);
                }

                // Cache texture binding.
                let mut bound_material = None;

                gl.bind_vertex_array(resources.scene_vao);

                for (i, mesh_meta) in resources.mesh_metas.iter().enumerate() {
                    let maybe_material_index = resources.meshes[i].material_index;
                    if bound_material != maybe_material_index {
                        bound_material = maybe_material_index;
                        if let Some(material_index) = maybe_material_index {
                            let material = &resources.materials[material_index as usize];

                            let diffuse_texture = &resources.textures[material.diffuse as usize];
                            let normal_texture = &resources.textures[material.normal as usize];
                            let specular_texture = &resources.textures[material.specular as usize];

                            gl.bind_texture_unit(1, diffuse_texture.name);
                            gl.bind_texture_unit(2, normal_texture.name);
                            gl.bind_texture_unit(3, specular_texture.name);

                            if let Some(loc) = self.diffuse_dimensions_loc.into() {
                                gl.uniform_2f(loc, diffuse_texture.dimensions);
                            }

                            if let Some(loc) = self.normal_dimensions_loc.into() {
                                gl.uniform_2f(loc, normal_texture.dimensions);
                            }

                            if let Some(loc) = self.specular_dimensions_loc.into() {
                                gl.uniform_2f(loc, specular_texture.dimensions);
                            }

                        // params.material_resources.bind_index(gl, material_index as usize);
                        } else {
                            // TODO SET DEFAULTS
                        }
                    }

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
        let modified = self.program.modified();
        if modified < self.program.update(gl, world) {
            if let ProgramName::Linked(name) = self.program.name(&world.global) {
                unsafe {
                    self.obj_to_wld_loc = get_uniform_location!(gl, *name, "obj_to_wld");

                    self.diffuse_sampler_loc = get_uniform_location!(gl, *name, "diffuse_sampler");
                    self.normal_sampler_loc = get_uniform_location!(gl, *name, "normal_sampler");
                    self.specular_sampler_loc = get_uniform_location!(gl, *name, "specular_sampler");

                    self.diffuse_dimensions_loc = get_uniform_location!(gl, *name, "diffuse_dimensions");
                    self.normal_dimensions_loc = get_uniform_location!(gl, *name, "normal_dimensions");
                    self.specular_dimensions_loc = get_uniform_location!(gl, *name, "specular_dimensions");

                    self.display_mode_loc = get_uniform_location!(gl, *name, "display_mode");
                }
            }
        }
    }

    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        Renderer {
            program: rendering::Program::new(
                gl,
                vec![
                    rendering::Shader::new(gl, gl::VERTEX_SHADER, vec![world.add_source("basic_renderer.vert")]),
                    rendering::Shader::new(gl, gl::FRAGMENT_SHADER, vec![world.add_source("basic_renderer.frag")]),
                ],
            ),
            obj_to_wld_loc: gl::OptionUniformLocation::NONE,

            diffuse_sampler_loc: gl::OptionUniformLocation::NONE,
            normal_sampler_loc: gl::OptionUniformLocation::NONE,
            specular_sampler_loc: gl::OptionUniformLocation::NONE,

            diffuse_dimensions_loc: gl::OptionUniformLocation::NONE,
            normal_dimensions_loc: gl::OptionUniformLocation::NONE,
            specular_dimensions_loc: gl::OptionUniformLocation::NONE,

            display_mode_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
