#![allow(non_snake_case)]

use openvr as vr;
use openvr::enums::Enum;

use gl_typed as gl;
use glutin::GlContext;
use std::mem;
use std::os::raw::c_void;
use std::ptr;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let gl_window = glutin::GlWindow::new(
        glutin::WindowBuilder::new()
            .with_title("Hello world!")
            .with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0)),
        glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)))
            .with_gl_profile(glutin::GlProfile::Core),
        &events_loop,
    )
    .unwrap();

    let mut win_dpi = gl_window.get_hidpi_factor();
    let mut win_size = gl_window.get_inner_size().unwrap();

    unsafe { gl_window.make_current().unwrap() };

    let gl = unsafe { gl::Gl::load_with(|s| gl_window.context().get_proc_address(s) as *const _) };

    unsafe {
        println!("OpenGL version {}", gl.get_string(gl::VERSION));
    }

    // === VR ===
    let vr_context = vr::Context::new(vr::ApplicationType::Scene).unwrap_or_else(|error| {
        panic!(
            "Failed to acquire context: {:?}",
            vr::InitError::from_unchecked(error).unwrap()
        );
    });
    let vr_system = vr::System::new(&vr_context).unwrap();
    let vr_compositor = vr::Compositor::new(&vr_context).unwrap();

    let render_dims = vr_system.get_recommended_render_target_size();

    println!("Recommender render target size: {:?}", render_dims);

    // --- VR ---

    unsafe { gl_window.make_current().unwrap() };

    let vr_fb_left;
    let vr_fb_left_tex;

    let vr_fb_right;
    let vr_fb_right_tex;

    unsafe fn recompile_shader(gl: &gl::Gl,
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

    unsafe fn create_fb_and_tex(
        gl: &gl::Gl,
        dims: vr::Dimensions,
    ) -> (gl::FramebufferName, gl::TextureName) {
        let fb = {
            let mut names: [Option<gl::FramebufferName>; 1] = mem::uninitialized();
            gl.gen_framebuffers(&mut names);
            let [ name ] = names;
            name.expect("Failed to acquire framebuffer name.")
        };
        let tex = {
            let mut names: [Option<gl::TextureName>; 1] = mem::uninitialized();
            gl.gen_textures(&mut names);
            let [ name ] = names;
            name.expect("Failed to acquire texture name.")
        };

        gl.bind_texture(gl::TEXTURE_2D, &tex);
        {
            gl.tex_image_2d(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8,
                dims.width as i32,
                dims.height as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
        }
        gl.bind_texture(gl::TEXTURE_2D, &gl::Unbind);

        gl.bind_framebuffer(gl::FRAMEBUFFER, &fb);
        {
            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                &tex,
                0,
            );

            assert!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE.into()
            );
        }
        gl.bind_framebuffer(gl::FRAMEBUFFER, &gl::DefaultFramebufferName);
        (fb, tex)
    }

    unsafe {
        {
            let r = create_fb_and_tex(&gl, render_dims);
            vr_fb_left = r.0;
            vr_fb_left_tex = r.1;
        }

        {
            let r = create_fb_and_tex(&gl, render_dims);
            vr_fb_right = r.0;
            vr_fb_right_tex = r.1;
        }

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

        let vb = {
            let mut names: [Option<gl::BufferName>; 1] = mem::uninitialized();
            gl.gen_buffers(&mut names);
            let [ name ] = names;
            name.expect("Failed to acquire buffer name.")
        };
        gl.bind_buffer(gl::ARRAY_BUFFER, &vb);
        gl.buffer_data(gl::ARRAY_BUFFER, &VERTEX_DATA, gl::STATIC_DRAW);

        let vao = {
            let mut names: [Option<gl::VertexArrayName>; 1] = mem::uninitialized();
            gl.gen_vertex_arrays(&mut names);
            let [ name ] = names;
            name.expect("Failed to acquire vertex array name.")
        };
        gl.bind_vertex_array(&vao);

        let pos_attrib = gl
            .get_attrib_location(&program, gl::static_cstr!("position"))
            .expect("Could not find attribute location.");
        let color_attrib = gl
            .get_attrib_location(&program, gl::static_cstr!("color"))
            .expect("Could not find attribute location.");
        const STRIDE: usize = 5 * mem::size_of::<f32>();
        gl.vertex_attrib_pointer(&pos_attrib, 2, gl::FLOAT, gl::FALSE, STRIDE, 0);
        gl.vertex_attrib_pointer(
            &color_attrib,
            3,
            gl::FLOAT,
            gl::FALSE,
            STRIDE,
            2 * mem::size_of::<f32>(),
        );
        gl.enable_vertex_attrib_array(&pos_attrib);
        gl.enable_vertex_attrib_array(&color_attrib);
    }

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            use glutin::Event;
            match event {
                Event::WindowEvent { event, .. } => {
                    use glutin::WindowEvent;
                    match event {
                        WindowEvent::CloseRequested => running = false,
                        WindowEvent::HiDpiFactorChanged(x) => {
                            win_dpi = x;
                        }
                        WindowEvent::Resized(x) => {
                            win_size = x;
                        }
                        _ => (),
                    }
                }
                Event::DeviceEvent { event, .. } => {
                    use glutin::DeviceEvent;
                    match event {
                        DeviceEvent::Key(keyboard_input) => {
                            if let Some(vk) = keyboard_input.virtual_keycode {
                                use glutin::VirtualKeyCode;
                                match vk {
                                    VirtualKeyCode::Escape => running = false,
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        });

        // === VR ===
        while let Some(event) = vr_system.poll_next_event() {
            println!("{:?}", &event);
        }

        let mut poses: [vr::sys::TrackedDevicePose_t; vr::sys::k_unMaxTrackedDeviceCount as usize] =
            unsafe { mem::zeroed() };

        vr_compositor.wait_get_poses(&mut poses[..], None).unwrap();
        // --- VR ---

        // draw everything here
        unsafe {
            let physical_size = win_size.to_physical(win_dpi);
            gl.viewport(
                0,
                0,
                physical_size.width as i32,
                physical_size.height as i32,
            );
            gl.clear_color(0.6, 0.7, 0.8, 1.0);
            gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT);
            gl.draw_arrays(gl::TRIANGLES, 0, 3);
        }

        // === VR ===
        unsafe fn render_vr(gl: &gl::Gl, vr_fb: &gl::FramebufferName, render_dims: vr::Dimensions) {
            gl.viewport(0, 0, render_dims.width as i32, render_dims.height as i32);
            gl.bind_framebuffer(gl::FRAMEBUFFER, vr_fb);
            gl.clear_color(0.6, 0.7, 0.8, 1.0);
            gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT);
            gl.draw_arrays(gl::TRIANGLES, 0, 3);
            gl.bind_framebuffer(gl::FRAMEBUFFER, &gl::DefaultFramebufferName);
            // vr::Texture_t leftEyeTexture = {(void*)(uintptr_t)leftEyeDesc.m_nResolveTextureId, vr::TextureType_OpenGL, vr::ColorSpace_Gamma };
            // vr::VRCompositor()->Submit(vr::Eye_Left, &leftEyeTexture );
            // vr::Texture_t rightEyeTexture = {(void*)(uintptr_t)rightEyeDesc.m_nResolveTextureId, vr::TextureType_OpenGL, vr::ColorSpace_Gamma };
            // vr::VRCompositor()->Submit(vr::Eye_Right, &rightEyeTexture );
        }

        unsafe {
            render_vr(&gl, &vr_fb_left, render_dims);
            render_vr(&gl, &vr_fb_right, render_dims);

            let mut l = vr::sys::Texture_t {
                handle: vr_fb_left_tex.as_u32() as usize as *const c_void as *mut c_void, // Screw it
                eType: vr::sys::ETextureType_TextureType_OpenGL,
                eColorSpace: vr::sys::EColorSpace_ColorSpace_Gamma, // TODO(mickvangelderen): IDK
            };

            let mut r = vr::sys::Texture_t {
                handle: vr_fb_right_tex.as_u32() as usize as *const c_void as *mut c_void, // Screw it
                eType: vr::sys::ETextureType_TextureType_OpenGL,
                eColorSpace: vr::sys::EColorSpace_ColorSpace_Gamma, // TODO(mickvangelderen): IDK
            };

            // NOTE(mickvangelderen): Binding the color attachments causes SIGSEGV!!!
            // gl::BindTexture(gl::TEXTURE_2D, vr_fb_left_tex);
            {
                vr_compositor
                    .submit(vr::Eye::Left, &mut l, None, vr::SubmitFlag::Default)
                    .unwrap_or_else(|error| {
                        panic!(
                            "failed to submit texture: {:?}",
                            vr::CompositorError::from_unchecked(error).unwrap()
                        );
                    });
            }
            // gl::BindTexture(gl::TEXTURE_2D, vr_fb_right_tex);
            {
                vr_compositor
                    .submit(vr::Eye::Right, &mut r, None, vr::SubmitFlag::Default)
                    .unwrap_or_else(|error| {
                        panic!(
                            "failed to submit texture: {:?}",
                            vr::CompositorError::from_unchecked(error).unwrap()
                        );
                    });
            }
            // gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        // --- VR ---

        gl_window.swap_buffers().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(17));
    }
}

static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 0.5, 0.0, 1.0, 0.0, 0.5, -0.5, 0.0, 0.0, 1.0,
];

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
attribute vec2 position;
attribute vec3 color;
varying vec3 v_color;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_color = color;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
varying vec3 v_color;
void main() {
    gl_FragColor = vec4(v_color, 1.0);
}
\0";
