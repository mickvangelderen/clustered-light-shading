#![allow(non_snake_case)]

use openvr as vr;
use openvr::enums::Enum;

use gl_typed as gl;
use glutin::GlContext;
use std::mem;
use std::os::raw::c_void;
use std::ptr;

/// Print out all loaded properties of some models and associated materials
pub fn print_model_info(models: &[tobj::Model], materials: &[tobj::Material]) {
    println!("# of models: {}", models.len());
    println!("# of materials: {}", materials.len());
    for m in models.iter() {
        dbg!(&m.name);
        dbg!(m.mesh.material_id);
        assert_eq!(dbg!(m.mesh.positions.len()) % 3, 0);
        assert_eq!(dbg!(m.mesh.indices.len()) % 3, 0);
    }
}

fn main() {
    let obj = tobj::load_obj(&std::path::Path::new("data/bunny.obj"));
    let (models, materials) = obj.unwrap();
    let model = models.iter().next().unwrap();

    print_model_info(&models, &materials);

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
    let vr_resources = match vr::Context::new(vr::ApplicationType::Scene) {
        Ok(context) => {
            unsafe {
            let dims = context.system().get_recommended_render_target_size();
            println!("Recommender render target size: {:?}", dims);
            let eye_left = EyeResources::new(&gl, dims);
            let eye_right = EyeResources::new(&gl, dims);

            Some(
                VrResources {
                    context,
                    dims,
                    eye_left,
                    eye_right,
                },
            )
        }
        }
        Err(error) => {
            eprintln!(
                "Failed to acquire context: {:?}",
                vr::InitError::from_unchecked(error).unwrap()
            );
            None
        }
    };

    // --- VR ---

    unsafe { gl_window.make_current().unwrap() };

    let (program, vao) = unsafe {
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
        gl.buffer_data(gl::ARRAY_BUFFER, &model.mesh.positions, gl::STATIC_DRAW);

        let pos_attrib = gl
            .get_attrib_location(&program, gl::static_cstr!("position"))
            .expect("Could not find attribute location.");
        const STRIDE: usize = 3 * mem::size_of::<f32>();
        gl.vertex_attrib_pointer(&pos_attrib, 3, gl::FLOAT, gl::FALSE, STRIDE, 0);
        gl.enable_vertex_attrib_array(&pos_attrib);

        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, &eb);
        gl.buffer_data(gl::ELEMENT_ARRAY_BUFFER, &model.mesh.indices, gl::STATIC_DRAW);

        gl.bind_vertex_array(&gl::Unbind);
        gl.bind_buffer(gl::ARRAY_BUFFER, &gl::Unbind);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, &gl::Unbind);

        (program, vao)
    };

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
        if let Some(ref vr_resources) = vr_resources {
            while let Some(event) = vr_resources.system().poll_next_event() {
                println!("{:?}", &event);
            }

            let mut poses: [vr::sys::TrackedDevicePose_t;
                vr::sys::k_unMaxTrackedDeviceCount as usize] = unsafe { mem::zeroed() };

            vr_resources.compositor().wait_get_poses(&mut poses[..], None).unwrap();
        }
        // --- VR ---

        // draw everything here
        unsafe {
            gl.enable(gl::DEPTH_TEST);
            // gl.polygon_mode(gl::FRONT_AND_BACK, gl::LINE);

            gl.use_program(&program);
            gl.bind_vertex_array(&vao);

            let physical_size = win_size.to_physical(win_dpi);
            gl.viewport(
                0,
                0,
                physical_size.width as i32,
                physical_size.height as i32,
            );
            gl.clear_color(0.6, 0.7, 0.8, 1.0);
            gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);
            gl.draw_elements(gl::TRIANGLES, model.mesh.indices.len(), gl::UNSIGNED_INT, 0);
        }

        // === VR ===
        unsafe fn render_vr(
            gl: &gl::Gl,
            eye: &EyeResources,
            model: &tobj::Model,
            render_dims: vr::Dimensions,
        ) {
            gl.viewport(0, 0, render_dims.width as i32, render_dims.height as i32);
            gl.bind_framebuffer(gl::FRAMEBUFFER, &eye.framebuffer);
            gl.clear_color(0.6, 0.7, 0.8, 1.0);
            gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT);
            gl.draw_elements(
                gl::TRIANGLES,
                model.mesh.indices.len() / 3,
                gl::UNSIGNED_INT,
                0,
            );
            gl.bind_framebuffer(gl::FRAMEBUFFER, &gl::DefaultFramebufferName);
        }

        if let Some(ref vr_resources) = vr_resources {
            unsafe {
                render_vr(&gl, &vr_resources.eye_left, &model, vr_resources.dims);
                render_vr(&gl, &vr_resources.eye_right, &model, vr_resources.dims);

                // NOTE(mickvangelderen): Binding the color attachments causes SIGSEGV!!!
                {
                    let mut texture_t = vr_resources.eye_left.gen_texture_t();
                    vr_resources.compositor()
                        .submit(vr::Eye::Left, &mut texture_t, None, vr::SubmitFlag::Default)
                        .unwrap_or_else(|error| {
                            panic!(
                                "failed to submit texture: {:?}",
                                vr::CompositorError::from_unchecked(error).unwrap()
                            );
                        });
                }
                {
                    let mut texture_t = vr_resources.eye_right.gen_texture_t();
                    vr_resources.compositor()
                        .submit(
                            vr::Eye::Right,
                            &mut texture_t,
                            None,
                            vr::SubmitFlag::Default,
                        )
                        .unwrap_or_else(|error| {
                            panic!(
                                "failed to submit texture: {:?}",
                                vr::CompositorError::from_unchecked(error).unwrap()
                            );
                        });
                }
            }
        }
        // --- VR ---

        gl_window.swap_buffers().unwrap();

        // std::thread::sleep(std::time::Duration::from_millis(17));
    }
}

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

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
attribute vec3 position;
varying vec3 v_color;
void main() {
    gl_Position = vec4(position, 1.0);
    v_color = position;
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

struct VrResources {
    context: vr::Context,
    dims: vr::Dimensions,
    eye_left: EyeResources,
    eye_right: EyeResources,
}

impl VrResources {
    #[inline]
    fn system(&self) -> &vr::System {
        &self.context.system()
    }

    #[inline]
    fn compositor(&self) -> &vr::Compositor {
        &self.context.compositor()
    }
}

struct EyeResources {
    framebuffer: gl::FramebufferName,
    texture: gl::TextureName,
}

impl EyeResources {
    unsafe fn new(gl: &gl::Gl, dims: vr::Dimensions) -> Self {
        let framebuffer = {
            let mut names: [Option<gl::FramebufferName>; 1] = mem::uninitialized();
            gl.gen_framebuffers(&mut names);
            let [name] = names;
            name.expect("Failed to acquire framebuffer name.")
        };
        let texture = {
            let mut names: [Option<gl::TextureName>; 1] = mem::uninitialized();
            gl.gen_textures(&mut names);
            let [name] = names;
            name.expect("Failed to acquire texture name.")
        };

        gl.bind_texture(gl::TEXTURE_2D, &texture);
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

        gl.bind_framebuffer(gl::FRAMEBUFFER, &framebuffer);
        {
            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                &texture,
                0,
            );

            assert!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE.into()
            );
        }
        gl.bind_framebuffer(gl::FRAMEBUFFER, &gl::DefaultFramebufferName);
        EyeResources {
            framebuffer,
            texture,
        }
    }

    fn gen_texture_t(&self) -> vr::sys::Texture_t {
        // NOTE(mickvangelderen): The handle is not actually a pointer in
        // OpenGL's case, it's just the texture name.
        vr::sys::Texture_t {
            handle: self.texture.as_u32() as usize as *const c_void as *mut c_void,
            eType: vr::sys::ETextureType_TextureType_OpenGL,
            eColorSpace: vr::sys::EColorSpace_ColorSpace_Gamma, // TODO(mickvangelderen): IDK
        }
    }
}
