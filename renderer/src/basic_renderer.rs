use crate::gl_ext::*;
use crate::keyboard_model;
use crate::rendering;
use crate::resources::Resources;
use crate::World;
use cgmath::*;
use gl_typed as gl;

pub struct Renderer {
    pub program_name: gl::ProgramName,
    pub vertex_shader_name: gl::ShaderName,
    pub fragment_shader_name: gl::ShaderName,
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
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
    pub material_resources: rendering::MaterialResources,
    pub shadow_texture_name: gl::TextureName,
    pub shadow_texture_dimensions: [f32; 2],
}

#[derive(Default)]
pub struct Update<B: AsRef<[u8]>> {
    pub vertex_shader: Option<B>,
    pub fragment_shader: Option<B>,
}

impl<B: AsRef<[u8]>> Update<B> {
    pub fn should_update(&self) -> bool {
        self.vertex_shader.is_some() || self.fragment_shader.is_some()
    }
}

impl Renderer {
    pub fn render(&self, gl: &gl::Gl, params: &Parameters, world: &World, resources: &Resources) {
        unsafe {
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.viewport(0, 0, params.width, params.height);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);
            gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into(), gl::COLOR_ATTACHMENT1.into()]);

            gl.clear_color(world.clear_color[0], world.clear_color[1], world.clear_color[2], 1.0);
            // Reverse-Z projection.
            gl.clear_depth(0.0);
            gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);

            gl.use_program(self.program_name);

            if let Some(loc) = self.shadow_sampler_loc.into() {
                gl.uniform_1i(loc, 0);
                gl.bind_sampler(0, self.shadow_sampler);
                gl.active_texture(gl::TEXTURE0);
                gl.bind_texture(gl::TEXTURE_2D, params.shadow_texture_name);
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

            for i in 0..resources.vaos.len() {
                let maybe_material_index = resources.meshes[i].material_index;
                if bound_material != maybe_material_index {
                    bound_material = maybe_material_index;
                    if let Some(material_index) = maybe_material_index {
                        let material = &resources.materials[material_index as usize];

                        let diffuse_texture = &resources.textures[material.diffuse as usize];
                        let normal_texture = &resources.textures[material.normal as usize];
                        let specular_texture = &resources.textures[material.specular as usize];

                        gl.active_texture(gl::TEXTURE1);
                        gl.bind_texture(gl::TEXTURE_2D, diffuse_texture.name);

                        gl.active_texture(gl::TEXTURE2);
                        gl.bind_texture(gl::TEXTURE_2D, normal_texture.name);

                        gl.active_texture(gl::TEXTURE3);
                        gl.bind_texture(gl::TEXTURE_2D, specular_texture.name);

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

                gl.bind_vertex_array(resources.vaos[i]);

                gl.draw_elements(gl::TRIANGLES, resources.element_counts[i], gl::UNSIGNED_INT, 0);
            }

            if self.shadow_sampler_loc.is_some() {
                gl.unbind_sampler(1);
            }

            gl.unbind_vertex_array();

            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            gl.unuse_program();
        }
    }

    pub fn update<B: AsRef<[u8]>>(&mut self, gl: &gl::Gl, update: Update<B>) {
        unsafe {
            let mut should_link = false;

            if let Some(bytes) = update.vertex_shader {
                self.vertex_shader_name
                    .compile(
                        gl,
                        &[
                            rendering::COMMON_DECLARATION.as_bytes(),
                            rendering::GLOBAL_DATA_DECLARATION.as_bytes(),
                            rendering::VIEW_DATA_DECLARATION.as_bytes(),
                            "#line 1 1\n".as_bytes(),
                            bytes.as_ref(),
                        ],
                    )
                    .unwrap_or_else(|e| eprintln!("{} (vertex):\n{}", file!(), e));
                should_link = true;
            }

            if let Some(bytes) = update.fragment_shader {
                self.fragment_shader_name
                    .compile(
                        gl,
                        &[
                            rendering::COMMON_DECLARATION.as_bytes(),
                            rendering::GLOBAL_DATA_DECLARATION.as_bytes(),
                            rendering::VIEW_DATA_DECLARATION.as_bytes(),
                            rendering::MATERIAL_DATA_DECLARATION.as_bytes(),
                            "#line 1 1\n".as_bytes(),
                            bytes.as_ref(),
                        ],
                    )
                    .unwrap_or_else(|e| eprintln!("{} (fragment):\n{}", file!(), e));
                should_link = true;
            }

            if should_link {
                self.program_name
                    .link(gl)
                    .unwrap_or_else(|e| eprintln!("{} (program):\n{}", file!(), e));

                gl.use_program(self.program_name);

                self.pos_from_obj_to_wld_loc = get_uniform_location!(gl, self.program_name, "pos_from_obj_to_wld");
                self.highlight_loc = get_uniform_location!(gl, self.program_name, "highlight");

                self.shadow_sampler_loc = get_uniform_location!(gl, self.program_name, "shadow_sampler");
                self.diffuse_sampler_loc = get_uniform_location!(gl, self.program_name, "diffuse_sampler");
                self.normal_sampler_loc = get_uniform_location!(gl, self.program_name, "normal_sampler");
                self.specular_sampler_loc = get_uniform_location!(gl, self.program_name, "specular_sampler");

                self.shadow_dimensions_loc = get_uniform_location!(gl, self.program_name, "shadow_dimensions");
                self.diffuse_dimensions_loc = get_uniform_location!(gl, self.program_name, "diffuse_dimensions");
                self.normal_dimensions_loc = get_uniform_location!(gl, self.program_name, "normal_dimensions");
                self.specular_dimensions_loc = get_uniform_location!(gl, self.program_name, "specular_dimensions");

                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            let vertex_shader_name = gl.create_shader(gl::VERTEX_SHADER).expect("Failed to create shader.");

            let fragment_shader_name = gl.create_shader(gl::FRAGMENT_SHADER).expect("Failed to create shader.");

            let program_name = gl.create_program().expect("Failed to create program_name.");
            gl.attach_shader(program_name, vertex_shader_name);
            gl.attach_shader(program_name, fragment_shader_name);

            let shadow_sampler = {
                let mut names: [Option<gl::SamplerName>; 1] = std::mem::uninitialized();
                gl.gen_samplers(&mut names);
                names[0].expect("Failed to generate sampler.")
            };

            gl.sampler_parameter_i(shadow_sampler, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
            gl.sampler_parameter_i(shadow_sampler, gl::TEXTURE_MAG_FILTER, gl::LINEAR);
            gl.sampler_parameter_i(shadow_sampler, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
            gl.sampler_parameter_i(shadow_sampler, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);

            Renderer {
                program_name,
                vertex_shader_name,
                fragment_shader_name,
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
