use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
    //
    pub highlight_loc: gl::OptionUniformLocation,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,

    pub shadow_sampler_loc: gl::OptionUniformLocation,
    pub diffuse_sampler_loc: gl::OptionUniformLocation,
    pub normal_sampler_loc: gl::OptionUniformLocation,
    pub specular_sampler_loc: gl::OptionUniformLocation,

    pub shadow_dimensions_loc: gl::OptionUniformLocation,
    pub diffuse_dimensions_loc: gl::OptionUniformLocation,
    pub normal_dimensions_loc: gl::OptionUniformLocation,
    pub specular_dimensions_loc: gl::OptionUniformLocation,

    pub shadow_sampler: gl::SamplerName,
}

pub struct Parameters {
    pub material_resources: rendering::MaterialResources,
    pub shadow_texture_name: gl::TextureName,
    pub shadow_texture_dimensions: [f32; 2],
}

impl Renderer {
    pub fn render(&mut self, gl: &gl::Gl, params: &Parameters, world: &mut World, resources: &Resources) {
        unsafe {
            self.update(gl, world);
            if let ProgramName::Linked(program_name) = self.program.name(&world.global) {
                gl.use_program(*program_name);

                if let Some(loc) = self.shadow_sampler_loc.into() {
                    gl.uniform_1i(loc, 0);
                    gl.bind_sampler(0, self.shadow_sampler);
                    gl.bind_texture_unit(0, params.shadow_texture_name);
                }

                if let Some(loc) = self.diffuse_sampler_loc.into() {
                    gl.uniform_1i(loc, 1);
                }

                if let Some(loc) = self.normal_sampler_loc.into() {
                    gl.uniform_1i(loc, 2);
                }

                if let Some(loc) = self.specular_sampler_loc.into() {
                    gl.uniform_1i(loc, 3);
                }

                if let Some(loc) = self.shadow_dimensions_loc.into() {
                    gl.uniform_2f(loc, params.shadow_texture_dimensions);
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

                            params.material_resources.bind_index(gl, material_index as usize);
                        } else {
                            // TODO SET DEFAULTS
                        }
                    }

                    if let Some(loc) = self.pos_from_obj_to_wld_loc.into() {
                        let pos_from_obj_to_wld = Matrix4::from_translation(resources.meshes[i].translate);

                        gl.uniform_matrix4f(loc, gl::MajorAxis::Column, pos_from_obj_to_wld.as_ref());
                    }

                    if let Some(loc) = self.highlight_loc.into() {
                        let highlight: f32 = keyboard_model::Index::new(resources.key_indices[i])
                            .map(|i| world.keyboard_model.pressure(i))
                            .unwrap_or(0.0);
                        gl.uniform_1f(loc, highlight);
                    }

                    gl.draw_elements_base_vertex(
                        gl::TRIANGLES,
                        mesh_meta.element_count,
                        gl::UNSIGNED_INT,
                        mesh_meta.element_offset,
                        mesh_meta.vertex_base,
                    );
                }

                if self.shadow_sampler_loc.is_some() {
                    gl.unbind_sampler(1);
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
                    self.highlight_loc = get_uniform_location!(gl, *name, "highlight");

                    self.shadow_sampler_loc = get_uniform_location!(gl, *name, "shadow_sampler");
                    self.diffuse_sampler_loc = get_uniform_location!(gl, *name, "diffuse_sampler");
                    self.normal_sampler_loc = get_uniform_location!(gl, *name, "normal_sampler");
                    self.specular_sampler_loc = get_uniform_location!(gl, *name, "specular_sampler");

                    self.shadow_dimensions_loc = get_uniform_location!(gl, *name, "shadow_dimensions");
                    self.diffuse_dimensions_loc = get_uniform_location!(gl, *name, "diffuse_dimensions");
                    self.normal_dimensions_loc = get_uniform_location!(gl, *name, "normal_dimensions");
                    self.specular_dimensions_loc = get_uniform_location!(gl, *name, "specular_dimensions");
                }
            }
        }
    }

    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        unsafe {
            let shadow_sampler = gl.create_sampler();

            gl.sampler_parameteri(shadow_sampler, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
            gl.sampler_parameteri(shadow_sampler, gl::TEXTURE_MAG_FILTER, gl::LINEAR);
            gl.sampler_parameteri(shadow_sampler, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
            gl.sampler_parameteri(shadow_sampler, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);

            Renderer {
                program: rendering::Program::new(
                    gl,
                    vec![world.add_source("basic_renderer.vert")],
                    vec![world.add_source("basic_renderer.frag")],
                ),
                pos_from_obj_to_wld_loc: gl::OptionUniformLocation::NONE,
                highlight_loc: gl::OptionUniformLocation::NONE,

                shadow_sampler_loc: gl::OptionUniformLocation::NONE,
                diffuse_sampler_loc: gl::OptionUniformLocation::NONE,
                normal_sampler_loc: gl::OptionUniformLocation::NONE,
                specular_sampler_loc: gl::OptionUniformLocation::NONE,

                shadow_dimensions_loc: gl::OptionUniformLocation::NONE,
                diffuse_dimensions_loc: gl::OptionUniformLocation::NONE,
                normal_dimensions_loc: gl::OptionUniformLocation::NONE,
                specular_dimensions_loc: gl::OptionUniformLocation::NONE,

                shadow_sampler,
            }
        }
    }
}
