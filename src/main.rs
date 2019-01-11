#![allow(non_snake_case)]

use openvr as vr;
use openvr::enums::Enum;

use glutin::GlContext;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_void;
use std::ptr;

unsafe fn get_string(name: u32) -> &'static str {
    CStr::from_ptr(gl::GetString(name) as *const i8)
        .to_str()
        .unwrap()
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let gl_window = glutin::GlWindow::new(
        glutin::WindowBuilder::new()
            .with_title("Hello world!")
            .with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0)),
        glutin::ContextBuilder::new(),
        &events_loop,
    )
    .unwrap();

    let mut win_dpi = gl_window.get_hidpi_factor();
    let mut win_size = gl_window.get_inner_size().unwrap();

    unsafe { gl_window.make_current().unwrap() };

    let _ = gl::load_with(|s| gl_window.context().get_proc_address(s) as *const _);

    unsafe {
        println!("OpenGL version {}", get_string(gl::VERSION));
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

    unsafe fn create_fb_and_tex(dims: vr::Dimensions) -> (u32, u32) {
        let fb = {
            let mut names: [u32; 1] = mem::uninitialized();
            gl::GenFramebuffers(names.len() as i32, names.as_mut_ptr());
            assert!(names[0] > 0, "Failed to acquire framebuffer.");
            names[0]
        };
        let tex = {
            let mut names: [u32; 1] = mem::uninitialized();
            gl::GenTextures(names.len() as i32, names.as_mut_ptr());
            assert!(names[0] > 0, "Failed to acquire texture.");
            names[0]
        };

        gl::BindTexture(gl::TEXTURE_2D, tex);
        {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                dims.width as i32,
                dims.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
        }
        gl::BindTexture(gl::TEXTURE_2D, 0);

        gl::BindFramebuffer(gl::FRAMEBUFFER, fb);
        {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                tex,
                0,
            );

            assert!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE);
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        (fb, tex)
    }

    unsafe {
        {
            let r = create_fb_and_tex(render_dims);
            vr_fb_left = r.0;
            vr_fb_left_tex = r.1;
        }

        {
            let r = create_fb_and_tex(render_dims);
            vr_fb_right = r.0;
            vr_fb_right_tex = r.1;
        }

        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
        gl::CompileShader(vs);

        let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
        gl::CompileShader(fs);

        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        gl::UseProgram(program);

        let mut vb = mem::uninitialized();
        gl::GenBuffers(1, &mut vb);
        gl::BindBuffer(gl::ARRAY_BUFFER, vb);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(&VERTEX_DATA) as isize,
            VERTEX_DATA.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        let mut vao = mem::uninitialized();
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let pos_attrib = gl::GetAttribLocation(program, b"position\0".as_ptr() as *const _);
        let color_attrib = gl::GetAttribLocation(program, b"color\0".as_ptr() as *const _);
        gl::VertexAttribPointer(
            pos_attrib as u32,
            2,
            gl::FLOAT,
            0,
            5 * mem::size_of::<f32>() as i32,
            ptr::null(),
        );
        gl::VertexAttribPointer(
            color_attrib as u32,
            3,
            gl::FLOAT,
            0,
            5 * mem::size_of::<f32>() as i32,
            (2 * mem::size_of::<f32>()) as *const () as *const _,
        );
        gl::EnableVertexAttribArray(pos_attrib as u32);
        gl::EnableVertexAttribArray(color_attrib as u32);
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
            gl::Viewport(
                0,
                0,
                physical_size.width as i32,
                physical_size.height as i32,
            );
            gl::ClearColor(0.6, 0.7, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        // === VR ===
        unsafe fn render_vr(vr_fb: u32, render_dims: vr::Dimensions) {
            gl::Viewport(0, 0, render_dims.width as i32, render_dims.height as i32);
            gl::BindFramebuffer(gl::FRAMEBUFFER, vr_fb);
            gl::ClearColor(0.6, 0.7, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            // vr::Texture_t leftEyeTexture = {(void*)(uintptr_t)leftEyeDesc.m_nResolveTextureId, vr::TextureType_OpenGL, vr::ColorSpace_Gamma };
            // vr::VRCompositor()->Submit(vr::Eye_Left, &leftEyeTexture );
            // vr::Texture_t rightEyeTexture = {(void*)(uintptr_t)rightEyeDesc.m_nResolveTextureId, vr::TextureType_OpenGL, vr::ColorSpace_Gamma };
            // vr::VRCompositor()->Submit(vr::Eye_Right, &rightEyeTexture );
        }

        unsafe {
            render_vr(vr_fb_left, render_dims);
            render_vr(vr_fb_right, render_dims);

            let mut l = vr::sys::Texture_t {
                handle: &vr_fb_left_tex as *const u32 as *mut c_void, // Screw it
                eType: vr::sys::ETextureType_TextureType_OpenGL,
                eColorSpace: vr::sys::EColorSpace_ColorSpace_Gamma, // TODO(mickvangelderen): IDK
            };

            let mut r = vr::sys::Texture_t {
                handle: &vr_fb_right_tex as *const u32 as *mut c_void, // Screw it
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
