use super::World;
use cgmath::*;
use gl_typed as gl;
use std::mem;

const VS_SRC: &'static [u8] = b"
#version 400 core

uniform mat4 pos_from_wld_to_clp;

in vec3 vs_ver_pos;
in vec2 vs_tex_pos;
out vec2 fs_tex_pos;

void main() {
    mat4 pos_from_obj_to_wld = mat4(
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0
    );

    gl_Position = pos_from_wld_to_clp*pos_from_obj_to_wld*vec4(vs_ver_pos, 1.0);
    fs_tex_pos = vs_tex_pos;
}\0";

const FS_SRC: &'static [u8] = b"
#version 400

uniform sampler2D diffuse_sampler;
// uniform sampler2D normal_sampler;

in vec2 fs_tex_pos;
out vec4 frag_color;

void main() {
    vec4 d = texture(diffuse_sampler, fs_tex_pos);
    frag_color = vec4(d);
}
\0";

unsafe fn recompile_shader(
    gl: &gl::Gl,
    name: &mut gl::ShaderName,
    source: &[u8],
) -> Result<(), String> {
    gl.shader_source(name, &[source]);
    gl.compile_shader(name);
    let status = gl.get_shaderiv_move(name, gl::COMPILE_STATUS);
    if status == gl::ShaderCompileStatus::Compiled.into() {
        Ok(())
    } else {
        let log = gl.get_shader_info_log_move(&name);
        Err(String::from_utf8(log).unwrap())
    }
}

pub struct Renderer {
    program: gl::ProgramName,
    vao: gl::VertexArrayName,
    pos_from_wld_to_clp_loc: gl::UniformLocation<[[f32; 4]; 4]>,
    diffuse_sampler_loc: gl::UniformLocation<i32>,
    // #[allow(unused)]
    // normal_sampler_loc: gl::UniformLocation<i32>,
    diffuse_texture: gl::TextureName,
    #[allow(unused)]
    normal_texture: gl::TextureName,
}

pub struct Parameters<'a, N: 'a>
where
    N: gl::MaybeDefaultFramebufferName,
{
    pub framebuffer: &'a N,
    pub width: i32,
    pub height: i32,
    pub pos_from_cam_to_clp: Matrix4<f32>,
}

impl Renderer {
    pub unsafe fn render<'a, N>(&self, gl: &gl::Gl, params: &Parameters<'a, N>, world: &World)
    where
        N: gl::MaybeDefaultFramebufferName,
    {
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

        gl.use_program(&self.program);
        gl.bind_vertex_array(&self.vao);

        gl.active_texture(gl::TEXTURE0);
        gl.bind_texture(gl::TEXTURE_2D, &self.diffuse_texture);
        gl.uniform_1i(&self.diffuse_sampler_loc, 0);

        // gl.active_texture(gl::TEXTURE1);
        // gl.bind_texture(gl::TEXTURE_2D, &self.normal_texture);
        // gl.uniform_1i(&self.normal_sampler_loc, 1);

        let pos_from_wld_to_clp = params.pos_from_cam_to_clp * world.camera.pos_from_wld_to_cam();

        gl.uniform_matrix4f(
            &self.pos_from_wld_to_clp_loc,
            gl::MajorAxis::Column,
            pos_from_wld_to_clp.as_ref(),
        );

        gl.draw_elements(
            gl::TRIANGLES,
            world.model.mesh.indices.len(),
            gl::UNSIGNED_INT,
            0,
        );

        gl.bind_framebuffer(gl::FRAMEBUFFER, &gl::DefaultFramebufferName);
    }

    pub unsafe fn new(gl: &gl::Gl, world: &World) -> Self {
        let mut vs = gl
            .create_shader(gl::VERTEX_SHADER)
            .expect("Failed to create shader.");
        recompile_shader(&gl, &mut vs, VS_SRC).unwrap_or_else(|e| panic!("{}", e));

        let mut fs = gl
            .create_shader(gl::FRAGMENT_SHADER)
            .expect("Failed to create shader.");
        recompile_shader(&gl, &mut fs, FS_SRC).unwrap_or_else(|e| panic!("{}", e));

        let mut program = gl.create_program().expect("Failed to create program.");
        gl.attach_shader(&mut program, &vs);
        gl.attach_shader(&mut program, &fs);
        gl.link_program(&mut program);
        gl.use_program(&program);

        let vao = {
            let mut names: [Option<gl::VertexArrayName>; 1] = mem::uninitialized();
            gl.gen_vertex_arrays(&mut names);
            let [name] = names;
            name.expect("Failed to acquire vertex array name.")
        };
        gl.bind_vertex_array(&vao);

        let (vb, eb) = {
            let mut names: [Option<gl::BufferName>; 2] = mem::uninitialized();
            gl.gen_buffers(&mut names);
            let [vb, eb] = names;
            (
                vb.expect("Failed to acquire buffer name."),
                eb.expect("Failed to acquire buffer name."),
            )
        };

        gl.bind_buffer(gl::ARRAY_BUFFER, &vb);
        gl.buffer_reserve(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&world.model.mesh.positions[..])
                + std::mem::size_of_val(&world.model.mesh.texcoords[..]),
            gl::STATIC_DRAW,
        );
        gl.buffer_sub_data(gl::ARRAY_BUFFER, 0, &world.model.mesh.positions);
        gl.buffer_sub_data(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&world.model.mesh.positions[..]),
            &world.model.mesh.texcoords,
        );

        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, &eb);
        gl.buffer_data(
            gl::ELEMENT_ARRAY_BUFFER,
            &world.model.mesh.indices,
            gl::STATIC_DRAW,
        );

        // AOS layout.

        let vs_ver_pos_loc = gl
            .get_attrib_location(&program, gl::static_cstr!("vs_ver_pos"))
            .expect("Could not find attribute location.");

        let vs_tex_pos_loc = gl
            .get_attrib_location(&program, gl::static_cstr!("vs_tex_pos"))
            .expect("Could not find attribute location.");

        gl.vertex_attrib_pointer(
            &vs_ver_pos_loc,
            3,
            gl::FLOAT,
            gl::FALSE,
            std::mem::size_of::<[f32; 3]>(),
            0,
        );

        gl.vertex_attrib_pointer(
            &vs_tex_pos_loc,
            2,
            gl::FLOAT,
            gl::FALSE,
            std::mem::size_of::<[f32; 2]>(),
            std::mem::size_of_val(&world.model.mesh.positions[..]),
        );

        gl.enable_vertex_attrib_array(&vs_ver_pos_loc);
        gl.enable_vertex_attrib_array(&vs_tex_pos_loc);

        gl.bind_vertex_array(&gl::Unbind);
        gl.bind_buffer(gl::ARRAY_BUFFER, &gl::Unbind);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, &gl::Unbind);

        let pos_from_wld_to_clp_loc: gl::UniformLocation<[[f32; 4]; 4]> = gl
            .get_uniform_location(&program, gl::static_cstr!("pos_from_wld_to_clp"))
            .expect("Could not find uniform location.");

        let diffuse_sampler_loc = gl
            .get_uniform_location(&program, gl::static_cstr!("diffuse_sampler"))
            .expect("Could not find attribute location.");

        // let normal_sampler_loc = gl
        //     .get_uniform_location(&program, gl::static_cstr!("normal_sampler"))
        //     .expect("Could not find attribute location.");

        let (diffuse_texture, normal_texture) = {
            let mut names: [Option<gl::TextureName>; 2] = std::mem::uninitialized();
            gl.gen_textures(&mut names);
            let [n0, n1] = names;
            (n0.unwrap(), n1.unwrap())
        };

        {
            let img = image::open("data/keyboard-diffuse.png").unwrap().flipv().to_rgba();
            gl.bind_texture(gl::TEXTURE_2D, &diffuse_texture);
            gl.tex_image_2d(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8,
                img.width() as i32,
                img.height() as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_ptr() as *const std::os::raw::c_void,
            );
            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
        }

        {
            let img = image::open("data/keyboard-normals.png").unwrap().flipv().to_rgba();
            gl.bind_texture(gl::TEXTURE_2D, &normal_texture);
            gl.tex_image_2d(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8,
                img.width() as i32,
                img.height() as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_ptr() as *const std::os::raw::c_void,
            );
            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
        }

        Renderer {
            program,
            pos_from_wld_to_clp_loc,
            diffuse_sampler_loc,
            // normal_sampler_loc,
            vao,
            diffuse_texture,
            normal_texture,
        }
    }
}
