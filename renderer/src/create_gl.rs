use gl_typed as gl;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GlConfiguration {
    pub framebuffer_srgb: bool,
}

pub fn create_gl(gl_window: &glutin::GlWindow, cfg: &GlConfiguration) -> gl::Gl {
    unsafe {
        let gl = gl::Gl::load_with(|s| glutin::GlContext::get_proc_address(gl_window.context(), s) as *const _);

        println!("OpenGL version {}.", gl.get_string(gl::VERSION));
        let flags = gl.get_context_flags();
        if flags.contains(gl::ContextFlag::DEBUG) {
            println!("OpenGL debugging enabled.");
            gl.enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        }

        // NOTE: This alignment is hardcoded in rendering.rs.
        assert_eq!(32, gl.get_uniform_buffer_offset_alignment());

        // Reverse-Z.
        gl.clip_control(gl::LOWER_LEFT, gl::ZERO_TO_ONE);
        gl.depth_func(gl::GREATER);

        if cfg.framebuffer_srgb {
            gl.enable(gl::FRAMEBUFFER_SRGB);
        } else {
            gl.disable(gl::FRAMEBUFFER_SRGB);
        }

        gl.enable(gl::MULTISAMPLE);

        gl
    }
}
