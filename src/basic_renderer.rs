use super::World;
use cgmath::*;
use gl_typed as gl;
use std::mem;

const VS_SRC: &'static [u8] = b"
#version 400 core

uniform mat4 pos_from_wld_to_clp;

in vec3 vs_ver_pos;
out vec3 vs_color;

void main() {
    mat4 pos_from_obj_to_wld = mat4(
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0
    );

    gl_Position = pos_from_wld_to_clp*pos_from_obj_to_wld*vec4(vs_ver_pos, 1.0);
    vs_color = vs_ver_pos;
}\0";

const FS_SRC: &'static [u8] = b"
#version 400

in vec3 vs_color;

void main() {
    gl_FragColor = vec4(vs_color, 1.0);
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
    pub unsafe fn render<'a, N>(
        &self,
        gl: &gl::Gl,
        params: &Parameters<'a, N>,
        world: &World,
    ) where
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

        let pos_from_wld_to_clp = params.pos_from_cam_to_clp
            * world.camera.pos_from_wld_to_cam();

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
        gl.buffer_data(
            gl::ARRAY_BUFFER,
            &world.model.mesh.positions,
            gl::STATIC_DRAW,
        );

        let vs_ver_pos_loc = gl
            .get_attrib_location(&program, gl::static_cstr!("vs_ver_pos"))
            .expect("Could not find attribute location.");
        const STRIDE: usize = 3 * mem::size_of::<f32>();
        gl.vertex_attrib_pointer(&vs_ver_pos_loc, 3, gl::FLOAT, gl::FALSE, STRIDE, 0);
        gl.enable_vertex_attrib_array(&vs_ver_pos_loc);

        let pos_from_wld_to_clp_loc: gl::UniformLocation<[[f32; 4]; 4]> = gl
            .get_uniform_location(&program, gl::static_cstr!("pos_from_wld_to_clp"))
            .expect("Could not find uniform location.");

        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, &eb);
        gl.buffer_data(
            gl::ELEMENT_ARRAY_BUFFER,
            &world.model.mesh.indices,
            gl::STATIC_DRAW,
        );

        gl.bind_vertex_array(&gl::Unbind);
        gl.bind_buffer(gl::ARRAY_BUFFER, &gl::Unbind);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, &gl::Unbind);

        Renderer {
            program,
            pos_from_wld_to_clp_loc,
            vao,
        }
    }
}
