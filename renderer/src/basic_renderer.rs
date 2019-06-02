use crate::keyboard_model;
use crate::rendering;
use crate::resources::Resources;
use crate::World;
use cgmath::*;
use gl_typed as gl;

pub struct Renderer {
    pub program: rendering::VSFSProgram,
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
    pub framebuffer: gl::FramebufferName,
    pub width: i32,
    pub height: i32,
    pub material_resources: rendering::MaterialResources,
    pub shadow_texture_name: gl::TextureName,
    pub shadow_texture_dimensions: [f32; 2],
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

    pub fn update(&mut self, gl: &gl::Gl, update: &rendering::VSFSProgramUpdate) {
        unsafe {
            if self.program.update(gl, update) {
                gl.use_program(self.program.name);

                self.pos_from_obj_to_wld_loc = get_uniform_location!(gl, self.program.name, "pos_from_obj_to_wld");
                self.highlight_loc = get_uniform_location!(gl, self.program.name, "highlight");

                self.shadow_sampler_loc = get_uniform_location!(gl, self.program.name, "shadow_sampler");
                self.diffuse_sampler_loc = get_uniform_location!(gl, self.program.name, "diffuse_sampler");
                self.normal_sampler_loc = get_uniform_location!(gl, self.program.name, "normal_sampler");
                self.specular_sampler_loc = get_uniform_location!(gl, self.program.name, "specular_sampler");

                self.shadow_dimensions_loc = get_uniform_location!(gl, self.program.name, "shadow_dimensions");
                self.diffuse_dimensions_loc = get_uniform_location!(gl, self.program.name, "diffuse_dimensions");
                self.normal_dimensions_loc = get_uniform_location!(gl, self.program.name, "normal_dimensions");
                self.specular_dimensions_loc = get_uniform_location!(gl, self.program.name, "specular_dimensions");

                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            let shadow_sampler = gl.create_sampler();

            gl.sampler_parameteri(shadow_sampler, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
            gl.sampler_parameteri(shadow_sampler, gl::TEXTURE_MAG_FILTER, gl::LINEAR);
            gl.sampler_parameteri(shadow_sampler, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
            gl.sampler_parameteri(shadow_sampler, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);

            Renderer {
                program: rendering::VSFSProgram::new(gl),
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
