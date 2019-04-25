use crate::frustrum::Frustrum;
use crate::keyboard_model;
use crate::resources::Resources;
use crate::shader_defines;
use crate::World;
use cgmath::*;
use gl_typed as gl;

unsafe fn recompile_shader(gl: &gl::Gl, name: gl::ShaderName, source: &[u8]) -> Result<(), String> {
    gl.shader_source(
        name,
        &[shader_defines::VERSION, shader_defines::DEFINES, source],
    );
    gl.compile_shader(name);
    let status = gl.get_shaderiv_move(name, gl::COMPILE_STATUS);
    if status == gl::ShaderCompileStatus::Compiled.into() {
        Ok(())
    } else {
        let log = gl.get_shader_info_log_move(name);
        Err(String::from_utf8(log).unwrap())
    }
}

pub struct Renderer {
    pub program_name: gl::ProgramName,
    pub vertex_shader_name: gl::ShaderName,
    pub fragment_shader_name: gl::ShaderName,
    //
    pub time_loc: gl::OptionUniformLocation,
    pub diffuse_loc: gl::OptionUniformLocation,
    pub ambient_loc: gl::OptionUniformLocation,
    pub specular_loc: gl::OptionUniformLocation,
    pub shininess_loc: gl::OptionUniformLocation,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
    pub pos_from_wld_to_cam_loc: gl::OptionUniformLocation,
    pub pos_from_wld_to_clp_loc: gl::OptionUniformLocation,
    pub pos_from_wld_to_lgt_loc: gl::OptionUniformLocation,
    pub highlight_loc: gl::OptionUniformLocation,
    pub diffuse_sampler_loc: gl::OptionUniformLocation,
    pub shadow_sampler_loc: gl::OptionUniformLocation,
    pub shadow_sampler: gl::SamplerName,
}

pub struct Parameters<'a> {
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
    pub pos_from_cam_to_clp: Matrix4<f32>,
    pub pos_from_wld_to_lgt: Matrix4<f32>,
    pub shadow_texture_name: gl::TextureName,
    pub frustrum: &'a Frustrum<f32>,
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
    pub unsafe fn render<'a>(
        &self,
        gl: &gl::Gl,
        params: &Parameters<'a>,
        world: &World,
        resources: &Resources,
    ) {
        gl.enable(gl::DEPTH_TEST);
        gl.enable(gl::CULL_FACE);
        gl.cull_face(gl::BACK);
        gl.viewport(0, 0, params.width, params.height);
        gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);
        gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into(), gl::COLOR_ATTACHMENT1.into()]);

        gl.clear_color(
            world.clear_color[0],
            world.clear_color[1],
            world.clear_color[2],
            1.0,
        );
        // Infinite far perspective projection.
        gl.clear_depth(1.0 - params.frustrum.z0 as f64 / params.frustrum.z1 as f64);
        gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);

        gl.use_program(self.program_name);

        if let Some(loc) = self.time_loc.into() {
            gl.uniform_1f(loc, world.time);
        }

        if let Some(loc) = self.diffuse_sampler_loc.into() {
            gl.uniform_1i(loc, 0);
        }

        if let Some(loc) = self.shadow_sampler_loc.into() {
            gl.uniform_1i(loc, 1);
            gl.bind_sampler(1, self.shadow_sampler);
            gl.active_texture(gl::TEXTURE1);
            gl.bind_texture(gl::TEXTURE_2D, params.shadow_texture_name);
        }

        let pos_from_wld_to_cam = if world.smooth_camera {
            world.camera.smooth_pos_from_wld_to_cam()
        } else {
            world.camera.pos_from_wld_to_cam()
        };

        if let Some(loc) = self.pos_from_wld_to_cam_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, pos_from_wld_to_cam.as_ref());
        }

        if let Some(loc) = self.pos_from_wld_to_clp_loc.into() {
            let pos_from_wld_to_clp = params.pos_from_cam_to_clp * pos_from_wld_to_cam;
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, pos_from_wld_to_clp.as_ref());
        }

        if let Some(loc) = self.pos_from_wld_to_lgt_loc.into() {
            gl.uniform_matrix4f(
                loc,
                gl::MajorAxis::Column,
                params.pos_from_wld_to_lgt.as_ref(),
            );
        }

        // Cache texture binding.
        let mut bound_material = None;

        for i in 0..resources.vaos.len() {
            let maybe_material_id = resources.meshes[i].material_id;
            if bound_material != maybe_material_id {
                bound_material = maybe_material_id;
                if let Some(material_id) = maybe_material_id {
                    let material = &resources.materials[material_id as usize];
                    let diffuse_texture = resources.diffuse_textures[material_id as usize];

                    gl.active_texture(gl::TEXTURE0);
                    gl.bind_texture(gl::TEXTURE_2D, diffuse_texture);

                    if let Some(loc) = self.ambient_loc.into() {
                        gl.uniform_3f(loc, material.ambient);
                    }

                    if let Some(loc) = self.diffuse_loc.into() {
                        gl.uniform_3f(loc, material.diffuse);
                    }

                    if let Some(loc) = self.specular_loc.into() {
                        gl.uniform_3f(loc, material.specular);
                    }

                    if let Some(loc) = self.shininess_loc.into() {
                        gl.uniform_1f(loc, material.shininess);
                    }
                } else {
                    gl.active_texture(gl::TEXTURE0);
                    gl.unbind_texture(gl::TEXTURE_2D);

                    if let Some(loc) = self.ambient_loc.into() {
                        gl.uniform_3f(loc, [1.0, 1.0, 1.0]);
                    }

                    if let Some(loc) = self.diffuse_loc.into() {
                        gl.uniform_3f(loc, [1.0, 1.0, 1.0]);
                    }

                    if let Some(loc) = self.specular_loc.into() {
                        gl.uniform_3f(loc, [1.0, 1.0, 1.0]);
                    }

                    if let Some(loc) = self.shininess_loc.into() {
                        gl.uniform_1f(loc, 64.0);
                    }
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
            // // NOTE: Help renderdoc.
            // gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, resources.ebs[i]);
            gl.draw_elements(
                gl::TRIANGLES,
                resources.element_counts[i],
                gl::UNSIGNED_INT,
                0,
            );
        }

        if self.shadow_sampler_loc.is_some() {
            gl.unbind_sampler(1);
        }

        gl.unbind_vertex_array();

        gl.bind_framebuffer(gl::FRAMEBUFFER, None);
        gl.unuse_program();
    }

    pub unsafe fn update<B: AsRef<[u8]>>(&mut self, gl: &gl::Gl, update: Update<B>) {
        let mut should_link = false;

        if let Some(bytes) = update.vertex_shader {
            recompile_shader(gl, self.vertex_shader_name, bytes.as_ref())
                .unwrap_or_else(|e| eprintln!("{}", e));
            should_link = true;
        }

        if let Some(bytes) = update.fragment_shader {
            recompile_shader(gl, self.fragment_shader_name, bytes.as_ref())
                .unwrap_or_else(|e| eprintln!("{}", e));
            should_link = true;
        }

        if should_link {
            gl.link_program(self.program_name);
            gl.use_program(self.program_name);

            macro_rules! get_uniform_location {
                ($gl: ident, $program: expr, $s: expr) => {{
                    let loc = $gl.get_uniform_location($program, gl::static_cstr!($s));
                    if loc.is_none() {
                        eprintln!("{}: Could not get uniform location {:?}.", file!(), $s);
                    }
                    loc
                }};
            }

            self.time_loc = get_uniform_location!(gl, self.program_name, "time");
            self.diffuse_loc = get_uniform_location!(gl, self.program_name, "diffuse");
            self.ambient_loc = get_uniform_location!(gl, self.program_name, "ambient");
            self.specular_loc = get_uniform_location!(gl, self.program_name, "specular");
            self.shininess_loc = get_uniform_location!(gl, self.program_name, "shininess");
            self.pos_from_obj_to_wld_loc =
                get_uniform_location!(gl, self.program_name, "pos_from_obj_to_wld");
            self.pos_from_wld_to_cam_loc =
                get_uniform_location!(gl, self.program_name, "pos_from_wld_to_cam");
            self.pos_from_wld_to_clp_loc =
                get_uniform_location!(gl, self.program_name, "pos_from_wld_to_clp");
            self.pos_from_wld_to_lgt_loc =
                get_uniform_location!(gl, self.program_name, "pos_from_wld_to_lgt");
            self.highlight_loc = get_uniform_location!(gl, self.program_name, "highlight");
            self.diffuse_sampler_loc =
                get_uniform_location!(gl, self.program_name, "diffuse_sampler");
            self.shadow_sampler_loc =
                get_uniform_location!(gl, self.program_name, "shadow_sampler");

            gl.unuse_program();
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            let vertex_shader_name = gl
                .create_shader(gl::VERTEX_SHADER)
                .expect("Failed to create shader.");

            let fragment_shader_name = gl
                .create_shader(gl::FRAGMENT_SHADER)
                .expect("Failed to create shader.");

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

            Renderer {
                program_name,
                vertex_shader_name,
                fragment_shader_name,
                time_loc: gl::OptionUniformLocation::NONE,
                diffuse_loc: gl::OptionUniformLocation::NONE,
                ambient_loc: gl::OptionUniformLocation::NONE,
                specular_loc: gl::OptionUniformLocation::NONE,
                shininess_loc: gl::OptionUniformLocation::NONE,
                pos_from_obj_to_wld_loc: gl::OptionUniformLocation::NONE,
                pos_from_wld_to_cam_loc: gl::OptionUniformLocation::NONE,
                pos_from_wld_to_clp_loc: gl::OptionUniformLocation::NONE,
                pos_from_wld_to_lgt_loc: gl::OptionUniformLocation::NONE,
                highlight_loc: gl::OptionUniformLocation::NONE,
                diffuse_sampler_loc: gl::OptionUniformLocation::NONE,
                shadow_sampler_loc: gl::OptionUniformLocation::NONE,
                shadow_sampler,
            }
        }
    }
}
