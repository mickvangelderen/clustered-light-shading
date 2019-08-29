use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
    pub dimensions_loc: gl::OptionUniformLocation,
    pub text_sampler_loc: gl::OptionUniformLocation,
    pub text_dimensions_loc: gl::OptionUniformLocation,
}

impl Context {
    pub fn render_text(&mut self) {
        let Context {
            ref gl,
            monospace: ref font,
            ref mut text_renderer,
            overlay_textbox: ref textbox,
            ..
        } = *self;

        unsafe {
            gl.named_buffer_data(font.vb, textbox.vertices.vec_as_bytes(), gl::STREAM_DRAW);
            gl.named_buffer_data(font.eb, textbox.indices.vec_as_bytes(), gl::STREAM_DRAW);
        }

        text_renderer.update(&mut rendering_context!(self));

        if let ProgramName::Linked(name) = text_renderer.program.name {
            unsafe {
                gl.disable(gl::DEPTH_TEST);
                gl.depth_mask(gl::FALSE);
                gl.enable(gl::BLEND);
                gl.blend_func(gl::SRC_ALPHA, gl::ONE);

                gl.use_program(name);
                gl.bind_vertex_array(font.vao);

                if let Some(loc) = text_renderer.dimensions_loc.into() {
                    gl.uniform_2f(loc, [self.win_size.width as f32, self.win_size.height as f32]);
                }

                if let Some(loc) = text_renderer.text_sampler_loc.into() {
                    gl.uniform_1i(loc, 0);
                }

                if let Some(loc) = text_renderer.text_dimensions_loc.into() {
                    gl.uniform_2f(loc, [font.meta.scale_x as f32, font.meta.scale_y as f32]);
                }

                // TODO: Handle more than 1 page.
                gl.bind_texture_unit(0, font.pages[0].texture_name);

                gl.draw_elements(gl::TRIANGLES, textbox.indices.len() as u32, gl::UNSIGNED_INT, 0);

                gl.unbind_vertex_array();
                gl.unuse_program();

                gl.enable(gl::DEPTH_TEST);
                gl.depth_mask(gl::TRUE);
                gl.disable(gl::BLEND);
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
                    self.dimensions_loc = get_uniform_location!(gl, name, "dimensions");
                    self.text_sampler_loc = get_uniform_location!(gl, name, "text_sampler");
                    self.text_dimensions_loc = get_uniform_location!(gl, name, "text_dimensions");
                }
            }
        }
    }

    pub fn new(context: &mut RenderingContext) -> Self {
        Renderer {
            program: vs_fs_program(context, "text_renderer.vert", "text_renderer.frag", String::new()),
            dimensions_loc: gl::OptionUniformLocation::NONE,
            text_sampler_loc: gl::OptionUniformLocation::NONE,
            text_dimensions_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
