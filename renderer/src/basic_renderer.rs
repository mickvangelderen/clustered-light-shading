use crate::keyboard_model;
use crate::World;
use crate::resources::Resources;
use cgmath::*;
use gl_typed as gl;

unsafe fn recompile_shader(gl: &gl::Gl, name: gl::ShaderName, source: &[u8]) -> Result<(), String> {
    gl.shader_source(name, &[source]);
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
    pub pos_from_wld_to_clp_loc: gl::OptionUniformLocation,
    pub highlight_loc: gl::OptionUniformLocation,
    pub diffuse_sampler_loc: gl::OptionUniformLocation,
    pub vs_ver_pos_loc: gl::OptionAttributeLocation,
    pub vs_tex_pos_loc: gl::OptionAttributeLocation,
}

pub struct Parameters {
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
    pub pos_from_cam_to_clp: Matrix4<f32>,
}

pub struct Update<'a> {
    pub vertex_shader: Option<&'a [u8]>,
    pub fragment_shader: Option<&'a [u8]>,
}

impl Renderer {
    pub unsafe fn render(&self, gl: &gl::Gl, params: &Parameters, world: &World, resources: &Resources) {
        gl.enable(gl::DEPTH_TEST);
        gl.enable(gl::CULL_FACE);
        gl.cull_face(gl::BACK);
        gl.viewport(0, 0, params.width, params.height);
        gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);

        gl.clear_color(
            world.clear_color[0],
            world.clear_color[1],
            world.clear_color[2],
            1.0,
        );
        gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);

        gl.use_program(self.program_name);

        if let Some(loc) = self.diffuse_sampler_loc.into() {
            gl.active_texture(gl::TEXTURE0);
            gl.uniform_1i(loc, 0);
        };

        if let Some(loc) = self.pos_from_wld_to_clp_loc.into() {
            let pos_from_wld_to_clp =
                params.pos_from_cam_to_clp * world.camera.pos_from_wld_to_cam();

            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, pos_from_wld_to_clp.as_ref());
        }

        // Cache texture binding.
        let mut bound_diffuse_texture: u32 = 0;

        for i in 0..resources.vaos.len() {
            if let Some(material_id) = resources.models[i].mesh.material_id {
                let diffuse_texture = resources.diffuse_textures[material_id];
                if diffuse_texture.into_u32() != bound_diffuse_texture {
                    gl.bind_texture(gl::TEXTURE_2D, diffuse_texture);
                    bound_diffuse_texture = diffuse_texture.into_u32();
                }
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

        gl.unbind_vertex_array();

        gl.bind_framebuffer(gl::FRAMEBUFFER, None);
    }

    pub unsafe fn update(&mut self, gl: &gl::Gl, update: Update) {
        let mut should_link = false;

        if let Some(bytes) = update.vertex_shader {
            recompile_shader(gl, self.vertex_shader_name, bytes)
                .unwrap_or_else(|e| eprintln!("{}", e));
            should_link = true;
        }

        if let Some(bytes) = update.fragment_shader {
            recompile_shader(gl, self.fragment_shader_name, bytes)
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
                        eprintln!(
                            "basic_renderer.rs: Could not get uniform location {:?}.",
                            $s
                        );
                    }
                    loc
                }};
            }

            self.pos_from_wld_to_clp_loc =
                get_uniform_location!(gl, self.program_name, "pos_from_wld_to_clp");
            self.highlight_loc = get_uniform_location!(gl, self.program_name, "highlight");
            self.diffuse_sampler_loc =
                get_uniform_location!(gl, self.program_name, "pos_from_wld_to_clp");

            macro_rules! get_attribute_location {
                ($gl: ident, $program: expr, $s: expr) => {{
                    let loc = $gl.get_attrib_location($program, gl::static_cstr!($s));
                    if loc.is_none() {
                        eprintln!(
                            "basic_renderer.rs: Could not get attribute location {:?}.",
                            $s
                        );
                    }
                    loc
                }};
            }

            self.vs_ver_pos_loc = get_attribute_location!(gl, self.program_name, "vs_ver_pos");
            self.vs_tex_pos_loc = get_attribute_location!(gl, self.program_name, "vs_tex_pos");

            // FIXME: Have to update the vaos!
        }
    }

    pub unsafe fn new(gl: &gl::Gl, world: &World) -> Self {
        let vertex_shader_name = gl
            .create_shader(gl::VERTEX_SHADER)
            .expect("Failed to create shader.");

        let fragment_shader_name = gl
            .create_shader(gl::FRAGMENT_SHADER)
            .expect("Failed to create shader.");

        let program_name = gl.create_program().expect("Failed to create program_name.");
        gl.attach_shader(program_name, vertex_shader_name);
        gl.attach_shader(program_name, fragment_shader_name);

        Renderer {
            program_name,
            vertex_shader_name,
            fragment_shader_name,
            pos_from_wld_to_clp_loc: gl::OptionUniformLocation::NONE,
            highlight_loc: gl::OptionUniformLocation::NONE,
            diffuse_sampler_loc: gl::OptionUniformLocation::NONE,
            vs_ver_pos_loc: gl::OptionAttributeLocation::NONE,
            vs_tex_pos_loc: gl::OptionAttributeLocation::NONE,
        }
    }
}
