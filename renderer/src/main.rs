#![allow(non_snake_case)]

mod ao_renderer;
mod basic_renderer;
mod camera;
mod convert;
mod filters;
mod frustrum;
mod gl_ext;
mod keyboard_model;
mod overlay_renderer;
mod post_renderer;
mod random_unit_sphere_surface;
mod random_unit_sphere_volume;
mod resources;
mod shader_defines;
mod shadow_renderer;
mod vsm_filter;

use crate::gl_ext::*;
use cgmath::*;
use convert::*;
use gl_typed as gl;
use gl_typed::convert::*;
use glutin::GlContext;
use notify::Watcher;
use openvr as vr;
use openvr::enums::Enum;
use renderer::log;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

const DESIRED_UPS: f32 = 90.0;
const DESIRED_FPS: f32 = 90.0;

const SHADOW_W: i32 = 1024;
const SHADOW_H: i32 = 1024;

const EYES: [vr::Eye; 2] = [vr::Eye::Left, vr::Eye::Right];

pub struct World {
    time: f32,
    clear_color: [f32; 3],
    camera: camera::Camera,
    sun_pos: Vector3<f32>,
    sun_rot: Rad<f32>,
    smooth_camera: bool,
    pos_from_cam_to_hmd: cgmath::Matrix4<f32>,
    keyboard_model: keyboard_model::KeyboardModel,
}

pub struct ViewIndependentResources {
    // Main shadow resources.
    pub shadow_framebuffer_name: gl::FramebufferName,
    pub shadow_texture: Texture<gl::symbols::Texture2D, gl::symbols::Rg32f>,
    pub shadow_depth_renderbuffer_name: gl::RenderbufferName,
    // Filter shadow resources.
    pub shadow_2_framebuffer_name: gl::FramebufferName,
    pub shadow_2_texture: Texture<gl::symbols::Texture2D, gl::symbols::Rg32f>,
}

impl ViewIndependentResources {
    pub fn new(gl: &gl::Gl, width: i32, height: i32) -> Self {
        unsafe {
            // Renderbuffers.

            let [shadow_depth_renderbuffer_name]: [gl::RenderbufferName; 1] = {
                let mut names: [Option<gl::RenderbufferName>; 1] = mem::uninitialized();
                gl.gen_renderbuffers(&mut names);
                names.try_transmute_each().unwrap()
            };

            gl.bind_renderbuffer(gl::RENDERBUFFER, shadow_depth_renderbuffer_name);
            gl.renderbuffer_storage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            gl.unbind_renderbuffer(gl::RENDERBUFFER);

            // Textures.

            let max_anisotropy = gl.get_max_texture_max_anisotropy();

            let texture_update = TextureUpdate::new()
                .data(width, height, None)
                .min_filter(gl::NEAREST.into())
                .mag_filter(gl::NEAREST.into())
                .wrap_s(gl::CLAMP_TO_EDGE.into())
                .wrap_t(gl::CLAMP_TO_EDGE.into());

            let shadow_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RG32F).unwrap();
            shadow_texture.update(gl, texture_update.max_anisotropy(max_anisotropy));

            let shadow_2_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RG32F).unwrap();
            shadow_2_texture.update(gl, texture_update.max_level(0));

            // Framebuffers.

            let [shadow_framebuffer_name, shadow_2_framebuffer_name]: [gl::FramebufferName; 2] = {
                let mut names: [Option<gl::FramebufferName>; 2] = mem::uninitialized();
                gl.gen_framebuffers(&mut names);
                names.try_transmute_each().unwrap()
            };

            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(shadow_framebuffer_name));
            {
                gl.framebuffer_texture_2d(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    shadow_texture.name(),
                    0,
                );

                gl.framebuffer_renderbuffer(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    gl::RENDERBUFFER,
                    shadow_depth_renderbuffer_name,
                );
                assert_eq!(
                    gl.check_framebuffer_status(gl::FRAMEBUFFER),
                    gl::FRAMEBUFFER_COMPLETE.into()
                );
            }

            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(shadow_2_framebuffer_name));
            {
                gl.framebuffer_texture_2d(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    shadow_2_texture.name(),
                    0,
                );

                gl.framebuffer_renderbuffer(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    gl::RENDERBUFFER,
                    shadow_depth_renderbuffer_name,
                );

                assert_eq!(
                    gl.check_framebuffer_status(gl::FRAMEBUFFER),
                    gl::FRAMEBUFFER_COMPLETE.into()
                );
            }

            ViewIndependentResources {
                shadow_framebuffer_name,
                shadow_texture,
                shadow_depth_renderbuffer_name,
                shadow_2_framebuffer_name,
                shadow_2_texture,
            }
        }
    }
}

pub struct ViewDependentResources {
    // Main frame resources.
    pub framebuffer_name: gl::FramebufferName,
    pub color_texture: Texture<gl::symbols::Texture2D, gl::symbols::Rgba8>,
    pub depth_texture: Texture<gl::symbols::Texture2D, gl::symbols::Depth24Stencil8>,
    pub nor_in_cam_texture: Texture<gl::symbols::Texture2D, gl::symbols::R11fG11fB10f>,
    // AO resources.
    pub ao_framebuffer_name: gl::FramebufferName,
    pub ao_texture: Texture<gl::symbols::Texture2D, gl::symbols::Rg8ui>,
    pub ao_depth_renderbuffer_name: gl::RenderbufferName,
    // Post resources.
    pub post_framebuffer_name: gl::FramebufferName,
    pub post_color_texture: Texture<gl::symbols::Texture2D, gl::symbols::Rgba8>,
    pub post_depth_texture: Texture<gl::symbols::Texture2D, gl::symbols::Depth24Stencil8>,
}

impl ViewDependentResources {
    pub fn new(gl: &gl::Gl, width: i32, height: i32) -> Self {
        unsafe {
            // Renderbuffers.

            let [ao_depth_renderbuffer_name]: [gl::RenderbufferName; 1] = {
                let mut names: [Option<gl::RenderbufferName>; 1] = mem::uninitialized();
                gl.gen_renderbuffers(&mut names);
                names.try_transmute_each().unwrap()
            };

            gl.bind_renderbuffer(gl::RENDERBUFFER, ao_depth_renderbuffer_name);
            gl.renderbuffer_storage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            gl.unbind_renderbuffer(gl::RENDERBUFFER);

            // Textures.

            let texture_update = TextureUpdate::new()
                .data(width, height, None)
                .min_filter(gl::NEAREST.into())
                .mag_filter(gl::NEAREST.into())
                .max_level(0)
                .wrap_s(gl::CLAMP_TO_EDGE.into())
                .wrap_t(gl::CLAMP_TO_EDGE.into());

            let color_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RGBA8).unwrap();
            color_texture.update(gl, texture_update);

            let nor_in_cam_texture = Texture::new(gl, gl::TEXTURE_2D, gl::R11F_G11F_B10F).unwrap();
            nor_in_cam_texture.update(gl, texture_update);

            let depth_texture = Texture::new(gl, gl::TEXTURE_2D, gl::DEPTH24_STENCIL8).unwrap();
            depth_texture.update(gl, texture_update);

            let ao_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RG8UI).unwrap();
            ao_texture.update(gl, texture_update);

            let post_color_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RGBA8).unwrap();
            post_color_texture.update(gl, texture_update);

            let post_depth_texture =
                Texture::new(gl, gl::TEXTURE_2D, gl::DEPTH24_STENCIL8).unwrap();
            post_depth_texture.update(gl, texture_update);

            // Framebuffers.

            let [framebuffer_name, ao_framebuffer_name, post_framebuffer_name]: [gl::FramebufferName; 3] = {
                let mut names: [Option<gl::FramebufferName>; 3] = mem::uninitialized();
                gl.gen_framebuffers(&mut names);
                names.try_transmute_each().unwrap()
            };

            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(framebuffer_name));
            {
                gl.framebuffer_texture_2d(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    color_texture.name(),
                    0,
                );

                gl.framebuffer_texture_2d(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    gl::TEXTURE_2D,
                    depth_texture.name(),
                    0,
                );

                gl.framebuffer_texture_2d(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT1,
                    gl::TEXTURE_2D,
                    nor_in_cam_texture.name(),
                    0,
                );

                assert_eq!(
                    gl.check_framebuffer_status(gl::FRAMEBUFFER),
                    gl::FRAMEBUFFER_COMPLETE.into()
                );
            }

            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(ao_framebuffer_name));
            {
                gl.framebuffer_texture_2d(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    ao_texture.name(),
                    0,
                );

                gl.framebuffer_renderbuffer(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    gl::RENDERBUFFER,
                    ao_depth_renderbuffer_name,
                );

                assert_eq!(
                    gl.check_framebuffer_status(gl::FRAMEBUFFER),
                    gl::FRAMEBUFFER_COMPLETE.into()
                );
            }

            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(post_framebuffer_name));
            {
                gl.framebuffer_texture_2d(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    post_color_texture.name(),
                    0,
                );

                gl.framebuffer_texture_2d(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    gl::TEXTURE_2D,
                    post_depth_texture.name(),
                    0,
                );

                assert_eq!(
                    gl.check_framebuffer_status(gl::FRAMEBUFFER),
                    gl::FRAMEBUFFER_COMPLETE.into()
                );
            }

            gl.bind_framebuffer(gl::FRAMEBUFFER, None);

            ViewDependentResources {
                framebuffer_name,
                color_texture,
                nor_in_cam_texture,
                depth_texture,
                ao_framebuffer_name,
                ao_texture,
                ao_depth_renderbuffer_name,
                post_framebuffer_name,
                post_color_texture,
                post_depth_texture,
            }
        }
    }

    pub fn resize(&self, gl: &gl::Gl, width: i32, height: i32) {
        unsafe {
            gl.bind_renderbuffer(gl::RENDERBUFFER, self.ao_depth_renderbuffer_name);
            gl.renderbuffer_storage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            gl.unbind_renderbuffer(gl::RENDERBUFFER);

            let texture_update = TextureUpdate::new().data(width, height, None);
            self.color_texture.update(gl, texture_update);
            self.depth_texture.update(gl, texture_update);
            self.nor_in_cam_texture.update(gl, texture_update);
            self.ao_texture.update(gl, texture_update);
        }
    }

    pub fn drop(self, gl: &gl::Gl) {
        unsafe {
            {
                // FIXME: MUT
                let mut names = [Some(self.framebuffer_name), Some(self.ao_framebuffer_name)];
                gl.delete_framebuffers(&mut names[..]);
            }
            {
                // FIXME: MUT
                let mut names = [Some(self.ao_depth_renderbuffer_name)];
                gl.delete_renderbuffers(&mut names[..]);
            }
            {
                self.color_texture.drop(gl);
                self.depth_texture.drop(gl);
                self.nor_in_cam_texture.drop(gl);
                self.ao_texture.drop(gl);
            }
        }
    }
}

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let resource_dir: PathBuf = [current_dir.as_ref(), Path::new("resources")]
        .into_iter()
        .collect();
    let log_dir: PathBuf = [current_dir.as_ref(), Path::new("logs")]
        .into_iter()
        .collect();
    let shadow_renderer_vs_path = resource_dir.join("shadow_renderer.vert");
    let shadow_renderer_fs_path = resource_dir.join("shadow_renderer.frag");
    let vsm_filter_vs_path = resource_dir.join("vsm_filter.vert");
    let vsm_filter_fs_path = resource_dir.join("vsm_filter.frag");
    let basic_renderer_vs_path = resource_dir.join("basic_renderer.vert");
    let basic_renderer_fs_path = resource_dir.join("basic_renderer.frag");
    let ao_renderer_vs_path = resource_dir.join("ao_renderer.vert");
    let ao_renderer_fs_path = resource_dir.join("ao_renderer.frag");
    let post_renderer_vs_path = resource_dir.join("post_renderer.vert");
    let post_renderer_fs_path = resource_dir.join("post_renderer.frag");
    let overlay_renderer_vs_path = resource_dir.join("overlay_renderer.vert");
    let overlay_renderer_fs_path = resource_dir.join("overlay_renderer.frag");

    let start_instant = std::time::Instant::now();

    let (tx_log, rx_log) = mpsc::channel::<log::Entry>();

    let timing_thread = thread::Builder::new()
        .name("log".to_string())
        .spawn(move || {
            use std::fs;
            use std::io;
            use std::io::Write;

            let mut file = io::BufWriter::new(fs::File::create(log_dir.join("log.bin")).unwrap());

            for entry in rx_log.iter() {
                file.write_all(&entry.into_ne_bytes()).unwrap();
            }

            file.flush().unwrap();
        })
        .unwrap();

    let (tx_fs, rx_fs) = mpsc::channel();

    let mut watcher = notify::raw_watcher(tx_fs).unwrap();

    watcher
        .watch("resources", notify::RecursiveMode::Recursive)
        .unwrap();

    let mut world = World {
        time: 0.0,
        clear_color: [0.0, 0.0, 0.0],
        camera: camera::Camera {
            smooth_position: Vector3::new(0.0, 1.0, 1.5),
            position: Vector3::new(0.0, 1.0, 1.5),
            smooth_yaw: Rad(0.0),
            yaw: Rad(0.0),
            smooth_pitch: Rad(0.0),
            pitch: Rad(0.0),
            smooth_fovy: Deg(90.0).into(),
            fovy: Deg(90.0).into(),
            positional_velocity: 2.0,
            angular_velocity: 0.4,
            zoom_velocity: 1.0,
        },
        sun_pos: Vector3::new(0.0, 0.0, 0.0),
        sun_rot: Deg(85.2).into(),
        smooth_camera: true,
        pos_from_cam_to_hmd: Matrix4::from_translation(Vector3::zero()),
        keyboard_model: keyboard_model::KeyboardModel::new(),
    };

    let mut events_loop = glutin::EventsLoop::new();

    let gl_window = glutin::GlWindow::new(
        glutin::WindowBuilder::new()
            .with_title("Hello world!")
            .with_dimensions(
                // Jump through some hoops to ensure a physical size, which is
                // what I want in case I'm recording at a specific resolution.
                glutin::dpi::PhysicalSize::new(1280.0, 720.0)
                    .to_logical(events_loop.get_primary_monitor().get_hidpi_factor()),
            )
            .with_maximized(false),
        glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)))
            .with_gl_profile(glutin::GlProfile::Core)
            // We do not wan't vsync since it will cause our loop to sync to the
            // desktop display frequency which is probably lower than the HMD's
            // 90Hz.
            .with_gl_debug_flag(cfg!(debug_assertions))
            .with_multisampling(4)
            .with_vsync(false)
            .with_double_buffer(Some(true)),
        &events_loop,
    )
    .unwrap();

    let mut win_dpi = gl_window.get_hidpi_factor();
    let mut win_size = gl_window.get_inner_size().unwrap();

    unsafe { gl_window.make_current().unwrap() };

    let gl = unsafe { gl::Gl::load_with(|s| gl_window.context().get_proc_address(s) as *const _) };

    unsafe {
        println!("OpenGL version {}.", gl.get_string(gl::VERSION));
        let flags = gl.get_context_flags();
        if flags.contains(gl::ContextFlags::CONTEXT_FLAG_DEBUG_BIT) {
            println!("OpenGL debugging enabled.");
            gl.enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        }
    }

    let glutin::dpi::PhysicalSize { width, height } = win_size.to_physical(win_dpi);

    let view_ind_res = ViewIndependentResources::new(&gl, SHADOW_W, SHADOW_H);
    let view_dep_res = ViewDependentResources::new(&gl, width as i32, height as i32);

    let random_unit_sphere_surface_texture = Texture::new(&gl, gl::TEXTURE_2D, gl::RGB8).unwrap();
    random_unit_sphere_surface_texture.update(
        &gl,
        TextureUpdate::new()
            .data(
                random_unit_sphere_surface::WIDTH as i32,
                random_unit_sphere_surface::HEIGHT as i32,
                Some(random_unit_sphere_surface::get().flatten()),
            )
            .max_level(0)
            .min_filter(gl::NEAREST.into())
            .mag_filter(gl::NEAREST.into())
            .wrap_s(gl::REPEAT.into())
            .wrap_t(gl::REPEAT.into()),
    );

    let mut shadow_renderer = {
        let mut shadow_renderer = shadow_renderer::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&shadow_renderer_vs_path).unwrap();
        let fs_bytes = std::fs::read(&shadow_renderer_fs_path).unwrap();
        shadow_renderer.update(
            &gl,
            shadow_renderer::Update {
                vertex_shader: Some(&vs_bytes),
                fragment_shader: Some(&fs_bytes),
            },
        );
        shadow_renderer
    };

    let mut vsm_filter = {
        let mut vsm_filter = vsm_filter::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&vsm_filter_vs_path).unwrap();
        let fs_bytes = std::fs::read(&vsm_filter_fs_path).unwrap();
        vsm_filter.update(
            &gl,
            vsm_filter::Update {
                vertex_shader: Some(&vs_bytes),
                fragment_shader: Some(&fs_bytes),
            },
        );
        vsm_filter
    };

    let mut basic_renderer = unsafe {
        let mut basic_renderer = basic_renderer::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&basic_renderer_vs_path).unwrap();
        let fs_bytes = std::fs::read(&basic_renderer_fs_path).unwrap();
        basic_renderer.update(
            &gl,
            basic_renderer::Update {
                vertex_shader: Some(&vs_bytes),
                fragment_shader: Some(&fs_bytes),
            },
        );
        basic_renderer
    };

    let mut ao_renderer = unsafe {
        let mut ao_renderer = ao_renderer::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&ao_renderer_vs_path).unwrap();
        let fs_bytes = std::fs::read(&ao_renderer_fs_path).unwrap();
        ao_renderer.update(
            &gl,
            ao_renderer::Update {
                vertex_shader: Some(&vs_bytes),
                fragment_shader: Some(&fs_bytes),
            },
        );
        ao_renderer
    };

    let mut post_renderer = unsafe {
        let mut post_renderer = post_renderer::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&post_renderer_vs_path).unwrap();
        let fs_bytes = std::fs::read(&post_renderer_fs_path).unwrap();
        post_renderer.update(
            &gl,
            post_renderer::Update {
                vertex_shader: Some(&vs_bytes),
                fragment_shader: Some(&fs_bytes),
            },
        );
        post_renderer
    };

    let mut overlay_renderer = {
        let mut overlay_renderer = overlay_renderer::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&overlay_renderer_vs_path).unwrap();
        let fs_bytes = std::fs::read(&overlay_renderer_fs_path).unwrap();
        overlay_renderer.update(
            &gl,
            overlay_renderer::Update {
                vertex_shader: Some(&vs_bytes),
                fragment_shader: Some(&fs_bytes),
            },
        );
        overlay_renderer
    };

    let resources = resources::Resources::new(&gl, &resource_dir);

    // === VR ===
    let vr_resources = match vr::Context::new(vr::ApplicationType::Scene) {
        Ok(context) => {
            let dims = context.system().get_recommended_render_target_size();
            println!("Recommended render target size: {:?}", dims);
            let eye_left = ViewDependentResources::new(&gl, dims.width as i32, dims.height as i32);
            let eye_right = ViewDependentResources::new(&gl, dims.width as i32, dims.height as i32);

            Some(VrResources {
                context,
                dims,
                eye_left,
                eye_right,
            })
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

    let mut focus = false;
    let mut input_forward = glutin::ElementState::Released;
    let mut input_backward = glutin::ElementState::Released;
    let mut input_left = glutin::ElementState::Released;
    let mut input_right = glutin::ElementState::Released;
    let mut input_up = glutin::ElementState::Released;
    let mut input_down = glutin::ElementState::Released;
    let mut input_sun_up = glutin::ElementState::Released;
    let mut input_sun_down = glutin::ElementState::Released;

    let mut fps_average = filters::MovingAverageF32::new(DESIRED_FPS);
    let mut last_frame_start = std::time::Instant::now();

    let mut running = true;
    while running {
        // File watch events.
        let mut shadow_renderer_update = shadow_renderer::Update::default();
        let mut vsm_filter_update = vsm_filter::Update::default();
        let mut basic_renderer_update = basic_renderer::Update::default();
        let mut ao_renderer_update = ao_renderer::Update::default();
        let mut post_renderer_update = post_renderer::Update::default();
        let mut overlay_renderer_update = overlay_renderer::Update::default();

        for event in rx_fs.try_iter() {
            if let Some(ref path) = event.path {
                match path {
                    path if path == &shadow_renderer_vs_path => {
                        shadow_renderer_update.vertex_shader = Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &shadow_renderer_fs_path => {
                        shadow_renderer_update.fragment_shader =
                            Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &vsm_filter_vs_path => {
                        vsm_filter_update.vertex_shader = Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &vsm_filter_fs_path => {
                        vsm_filter_update.fragment_shader = Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &basic_renderer_vs_path => {
                        basic_renderer_update.vertex_shader = Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &basic_renderer_fs_path => {
                        basic_renderer_update.fragment_shader = Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &ao_renderer_vs_path => {
                        ao_renderer_update.vertex_shader = Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &ao_renderer_fs_path => {
                        ao_renderer_update.fragment_shader = Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &post_renderer_vs_path => {
                        post_renderer_update.vertex_shader = Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &post_renderer_fs_path => {
                        post_renderer_update.fragment_shader = Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &overlay_renderer_vs_path => {
                        overlay_renderer_update.vertex_shader = Some(std::fs::read(&path).unwrap());
                    }
                    path if path == &overlay_renderer_fs_path => {
                        overlay_renderer_update.fragment_shader =
                            Some(std::fs::read(&path).unwrap());
                    }
                    _ => {}
                }
            }
        }

        if shadow_renderer_update.should_update() {
            shadow_renderer.update(&gl, shadow_renderer_update);
        }

        if vsm_filter_update.should_update() {
            vsm_filter.update(&gl, vsm_filter_update);
        }

        if basic_renderer_update.should_update() {
            unsafe {
                basic_renderer.update(&gl, basic_renderer_update);
            }
        }

        if ao_renderer_update.should_update() {
            unsafe {
                ao_renderer.update(&gl, ao_renderer_update);
            }
        }

        if post_renderer_update.should_update() {
            unsafe {
                post_renderer.update(&gl, post_renderer_update);
            }
        }

        if overlay_renderer_update.should_update() {
            overlay_renderer.update(&gl, overlay_renderer_update);
        }

        let simulation_start_nanos = start_instant.elapsed().as_nanos() as u64;

        let mut mouse_dx = 0.0;
        let mut mouse_dy = 0.0;
        let mut mouse_dscroll = 0.0;
        let mut should_resize = false;

        events_loop.poll_events(|event| {
            use glutin::Event;
            match event {
                Event::WindowEvent { event, .. } => {
                    use glutin::WindowEvent;
                    match event {
                        WindowEvent::CloseRequested => running = false,
                        WindowEvent::HiDpiFactorChanged(val) => {
                            win_dpi = val;
                            should_resize = true;
                        }
                        WindowEvent::Focused(val) => focus = val,
                        WindowEvent::Resized(val) => {
                            win_size = val;
                            should_resize = true;
                        }
                        _ => (),
                    }
                }
                Event::DeviceEvent { event, .. } => {
                    use glutin::DeviceEvent;
                    match event {
                        DeviceEvent::Key(keyboard_input) => {
                            if let Some(vk) = keyboard_input.virtual_keycode {
                                // This has to update regardless of focus.
                                world.keyboard_model.process_event(vk, keyboard_input.state);

                                // use glutin::ElementState;
                                use glutin::VirtualKeyCode;
                                match vk {
                                    VirtualKeyCode::W => input_forward = keyboard_input.state,
                                    VirtualKeyCode::S => input_backward = keyboard_input.state,
                                    VirtualKeyCode::A => input_left = keyboard_input.state,
                                    VirtualKeyCode::D => input_right = keyboard_input.state,
                                    VirtualKeyCode::Q => input_up = keyboard_input.state,
                                    VirtualKeyCode::Z => input_down = keyboard_input.state,
                                    VirtualKeyCode::C => {
                                        if keyboard_input.state == ElementState::Pressed && focus {
                                            world.smooth_camera = !world.smooth_camera;
                                        }
                                    }
                                    VirtualKeyCode::Escape => {
                                        if keyboard_input.state == ElementState::Pressed && focus {
                                            running = false;
                                        }
                                    }
                                    VirtualKeyCode::Up => input_sun_up = keyboard_input.state,
                                    VirtualKeyCode::Down => input_sun_down = keyboard_input.state,
                                    _ => (),
                                }
                            }
                        }
                        DeviceEvent::Motion { axis, value } => {
                            if focus {
                                match axis {
                                    0 => mouse_dx += value,
                                    1 => mouse_dy += value,
                                    3 => mouse_dscroll += value,
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

        if should_resize {
            let glutin::dpi::PhysicalSize { width, height } = win_size.to_physical(win_dpi);
            println!("win_size: {:?}", win_size);
            println!(
                "win_size: {:?}",
                glutin::dpi::PhysicalSize { width, height }
            );
            view_dep_res.resize(&gl, width as i32, height as i32);
        }

        use glutin::ElementState;

        let delta_time = 1.0 / DESIRED_UPS as f32;

        world.camera.update(&camera::CameraUpdate {
            delta_time,
            delta_position: if focus {
                Vector3 {
                    x: match input_left {
                        ElementState::Pressed => -1.0,
                        ElementState::Released => 0.0,
                    } + match input_right {
                        ElementState::Pressed => 1.0,
                        ElementState::Released => 0.0,
                    },
                    y: match input_up {
                        ElementState::Pressed => 1.0,
                        ElementState::Released => 0.0,
                    } + match input_down {
                        ElementState::Pressed => -1.0,
                        ElementState::Released => 0.0,
                    },
                    z: match input_forward {
                        ElementState::Pressed => -1.0,
                        ElementState::Released => 0.0,
                    } + match input_backward {
                        ElementState::Pressed => 1.0,
                        ElementState::Released => 0.0,
                    },
                }
            } else {
                Vector3::zero()
            },
            delta_yaw: Rad(mouse_dx as f32),
            delta_pitch: Rad(mouse_dy as f32),
            delta_scroll: mouse_dscroll as f32,
        });

        if focus {
            world.sun_rot += Rad(0.5)
                * (match input_sun_up {
                    ElementState::Pressed => -1.0,
                    ElementState::Released => 0.0,
                } + match input_sun_down {
                    ElementState::Pressed => 1.0,
                    ElementState::Released => 0.0,
                })
                * delta_time;
        }

        world.keyboard_model.simulate(delta_time);

        world.time += delta_time;

        let simulation_end_pose_start_nanos = start_instant.elapsed().as_nanos() as u64;

        // === VR ===
        if let Some(ref vr_resources) = vr_resources {
            while let Some(event) = vr_resources.system().poll_next_event() {
                println!("{:?}", &event);
            }

            let mut poses: [vr::sys::TrackedDevicePose_t;
                vr::sys::k_unMaxTrackedDeviceCount as usize] = unsafe { mem::zeroed() };

            vr_resources
                .compositor()
                .wait_get_poses(&mut poses[..], None)
                .unwrap();

            const HMD_POSE_INDEX: usize = vr::sys::k_unTrackedDeviceIndex_Hmd as usize;
            let hmd_pose = poses[HMD_POSE_INDEX];
            if hmd_pose.bPoseIsValid {
                for i in 0..3 {
                    // Fun.
                    world.clear_color[i] = hmd_pose.vAngularVelocity.v[i].abs() * 0.1;
                }
                world.pos_from_cam_to_hmd =
                    Matrix4::<f32>::from_hmd(hmd_pose.mDeviceToAbsoluteTracking.m)
                        .invert()
                        .unwrap();
            } else {
                // TODO: Structure code better to facilitate reset in case of vr crash/disconnect.
                world.pos_from_cam_to_hmd = Matrix4::from_translation(Vector3::zero());
            }
        }
        // --- VR ---

        let pose_end_render_start_nanos = start_instant.elapsed().as_nanos() as u64;

        // draw everything here

        let sun_frustrum = frustrum::Frustrum {
            x0: -25.0,
            x1: 25.0,
            y0: -25.0,
            y1: 25.0,
            z0: 30.0,
            z1: -30.0,
        };

        let pos_from_lgt_to_clp = sun_frustrum.orthographic();

        let sun_ori = Quaternion::from_angle_y(Deg(10.0)) * Quaternion::from_angle_x(world.sun_rot);

        let sun_pos_in_sun = sun_ori * world.sun_pos;

        let pos_from_wld_to_lgt =
            Matrix4::from_translation(sun_pos_in_sun) * Matrix4::from(sun_ori);

        unsafe {
            let physical_size = win_size.to_physical(win_dpi);

            // View independent.
            shadow_renderer.render(
                &gl,
                &shadow_renderer::Parameters {
                    framebuffer: Some(view_ind_res.shadow_framebuffer_name),
                    width: SHADOW_W,
                    height: SHADOW_H,
                    pos_from_wld_to_clp: pos_from_lgt_to_clp * pos_from_wld_to_lgt,
                    frustrum: &sun_frustrum,
                },
                &resources,
            );

            // View independent.
            vsm_filter.render(
                &gl,
                &vsm_filter::Parameters {
                    width: SHADOW_W,
                    height: SHADOW_H,
                    framebuffer_x: view_ind_res.shadow_2_framebuffer_name,
                    framebuffer_xy: view_ind_res.shadow_framebuffer_name,
                    color: view_ind_res.shadow_texture.name(),
                    color_x: view_ind_res.shadow_2_texture.name(),
                },
            );

            let frustrum = {
                let z0 = -0.1;
                let dy = -z0 * Rad::tan(Rad(Rad::from(world.camera.fovy).0 as f64) / 2.0);
                let dx = dy * physical_size.width as f64 / physical_size.height as f64;
                frustrum::Frustrum::<f64> {
                    x0: -dx,
                    x1: dx,
                    y0: -dy,
                    y1: dy,
                    z0,
                    z1: -100.0,
                }
            };

            // Do math in f64 precision.
            let pos_from_hmd_to_clp: Matrix4<f32> =
                frustrum.perspective_infinite_far().cast().unwrap();
            let frustrum: frustrum::Frustrum<f32> = frustrum.cast().unwrap();

            basic_renderer.render(
                &gl,
                &basic_renderer::Parameters {
                    framebuffer: Some(view_dep_res.framebuffer_name),
                    width: physical_size.width as i32,
                    height: physical_size.height as i32,
                    pos_from_cam_to_clp: pos_from_hmd_to_clp,
                    pos_from_wld_to_lgt: pos_from_lgt_to_clp * pos_from_wld_to_lgt,
                    shadow_texture_name: view_ind_res.shadow_texture.name(),
                    frustrum: &frustrum,
                },
                &world,
                &resources,
            );

            ao_renderer.render(
                &gl,
                &ao_renderer::Parameters {
                    framebuffer: Some(view_dep_res.ao_framebuffer_name),
                    width: physical_size.width as i32,
                    height: physical_size.height as i32,
                    color_texture_name: view_dep_res.color_texture.name(),
                    depth_texture_name: view_dep_res.depth_texture.name(),
                    nor_in_cam_texture_name: view_dep_res.nor_in_cam_texture.name(),
                    random_unit_sphere_surface_texture_name: random_unit_sphere_surface_texture
                        .name(),
                    frustrum: &frustrum,
                },
                &world,
            );

            post_renderer.render(
                &gl,
                &post_renderer::Parameters {
                    framebuffer: None,
                    width: physical_size.width as i32,
                    height: physical_size.height as i32,
                    color_texture_name: view_dep_res.color_texture.name(),
                    depth_texture_name: view_dep_res.depth_texture.name(),
                    nor_in_cam_texture_name: view_dep_res.nor_in_cam_texture.name(),
                    ao_texture_name: view_dep_res.ao_texture.name(),
                    frustrum: &frustrum,
                },
                &world,
            );

            overlay_renderer.render(
                &gl,
                &overlay_renderer::Parameters {
                    framebuffer: None,
                    x0: 0,
                    x1: (physical_size.height / 3.0) as i32,
                    y0: 0,
                    y1: (physical_size.height / 3.0) as i32,
                    color_texture_name: view_ind_res.shadow_texture.name(),
                },
            );
        }

        // === VR ===
        if let Some(ref vr_resources) = vr_resources {
            for &eye in EYES.into_iter() {
                // VIVE:
                // Left: [-1.3896277, 1.2525954, -1.4736392, 1.4612536]
                // Left: Matrix4 [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [-0.0307, 0.0, 0.015, 1.0]]
                // Right: [-1.2475655, 1.3957016, -1.473202, 1.4637187]
                // Right: Matrix4 [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0307, 0.0, 0.015, 1.0]]

                let frustrum = {
                    // These are the tangents.
                    let [l, r, b, t] = vr_resources.context.system().get_projection_raw(eye);
                    let z0 = -0.1;
                    let z1 = -100.0;
                    frustrum::Frustrum::<f64> {
                        x0: -z0 * l as f64,
                        x1: -z0 * r as f64,
                        y0: -z0 * b as f64,
                        y1: -z0 * t as f64,
                        z0,
                        z1,
                    }
                };

                // NOTE: At least on the HTC Vive, the z coordinate is mapped to (0, 1) instead of (-1, 1).
                let pos_from_eye_to_clp: Matrix4<f32> = frustrum.perspective_z0p1().cast().unwrap();
                let frustrum: frustrum::Frustrum<f32> = frustrum.cast().unwrap();
                let pos_from_eye_to_hmd: Matrix4<f32> = vr_resources
                    .context
                    .system()
                    .get_eye_to_head_transform(eye)
                    .hmd_into();

                let pos_from_hmd_to_clp =
                    pos_from_eye_to_clp * pos_from_eye_to_hmd.invert().unwrap();

                unsafe {
                    let view_dep_res = &vr_resources[eye];

                    basic_renderer.render(
                        &gl,
                        &basic_renderer::Parameters {
                            framebuffer: Some(view_dep_res.framebuffer_name),
                            width: vr_resources.dims.width as i32,
                            height: vr_resources.dims.height as i32,
                            pos_from_cam_to_clp: pos_from_hmd_to_clp * world.pos_from_cam_to_hmd,
                            pos_from_wld_to_lgt: pos_from_lgt_to_clp * pos_from_wld_to_lgt,
                            shadow_texture_name: view_ind_res.shadow_texture.name(),
                            frustrum: &frustrum,
                        },
                        &world,
                        &resources,
                    );

                    ao_renderer.render(
                        &gl,
                        &ao_renderer::Parameters {
                            framebuffer: Some(view_dep_res.ao_framebuffer_name),
                            width: vr_resources.dims.width as i32,
                            height: vr_resources.dims.height as i32,
                            color_texture_name: view_dep_res.color_texture.name(),
                            depth_texture_name: view_dep_res.depth_texture.name(),
                            nor_in_cam_texture_name: view_dep_res.nor_in_cam_texture.name(),
                            random_unit_sphere_surface_texture_name:
                                random_unit_sphere_surface_texture.name(),
                            frustrum: &frustrum,
                        },
                        &world,
                    );

                    post_renderer.render(
                        &gl,
                        &post_renderer::Parameters {
                            framebuffer: Some(view_dep_res.post_framebuffer_name),
                            width: vr_resources.dims.width as i32,
                            height: vr_resources.dims.height as i32,
                            color_texture_name: view_dep_res.color_texture.name(),
                            depth_texture_name: view_dep_res.depth_texture.name(),
                            nor_in_cam_texture_name: view_dep_res.nor_in_cam_texture.name(),
                            ao_texture_name: view_dep_res.ao_texture.name(),
                            frustrum: &frustrum,
                        },
                        &world,
                    );
                }
            }

            for &eye in EYES.into_iter() {
                // NOTE(mickvangelderen): Binding the color attachments causes SIGSEGV!!!
                let mut texture_t = gen_texture_t(vr_resources[eye].post_color_texture.name());
                vr_resources
                    .compositor()
                    .submit(eye, &mut texture_t, None, vr::SubmitFlag::Default)
                    .unwrap_or_else(|error| {
                        panic!(
                            "failed to submit texture: {:?}",
                            vr::CompositorError::from_unchecked(error).unwrap()
                        );
                    });
            }
        }
        // --- VR ---

        let render_end_nanos = start_instant.elapsed().as_nanos() as u64;

        gl_window.swap_buffers().unwrap();

        // std::thread::sleep(std::time::Duration::from_millis(17));

        tx_log
            .send(log::Entry {
                simulation_start_nanos,
                simulation_end_pose_start_nanos,
                pose_end_render_start_nanos,
                render_end_nanos,
            })
            .unwrap();

        {
            let duration = {
                let now = std::time::Instant::now();
                let duration = now.duration_since(last_frame_start);
                last_frame_start = now;
                duration
            };
            const NANOS_PER_SEC: f32 = 1_000_000_000.0;
            let fps = NANOS_PER_SEC
                / (duration.as_secs() as f32 * NANOS_PER_SEC + duration.subsec_nanos() as f32);
            fps_average.submit(fps);
            gl_window
                .window()
                .set_title(&format!("VR Lab - {:02.1} FPS", fps_average.compute()));
        }
    }

    drop(tx_log);

    timing_thread.join().unwrap();
}

struct VrResources {
    context: vr::Context,
    dims: vr::Dimensions,
    eye_left: ViewDependentResources,
    eye_right: ViewDependentResources,
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

impl std::ops::Index<vr::Eye> for VrResources {
    type Output = ViewDependentResources;

    #[inline]
    fn index(&self, eye: vr::Eye) -> &Self::Output {
        match eye {
            vr::Eye::Left => &self.eye_left,
            vr::Eye::Right => &self.eye_right,
        }
    }
}

impl std::ops::IndexMut<vr::Eye> for VrResources {
    #[inline]
    fn index_mut(&mut self, eye: vr::Eye) -> &mut Self::Output {
        match eye {
            vr::Eye::Left => &mut self.eye_left,
            vr::Eye::Right => &mut self.eye_right,
        }
    }
}

fn gen_texture_t(name: gl::TextureName) -> vr::sys::Texture_t {
    // NOTE(mickvangelderen): The handle is not actually a pointer in
    // OpenGL's case, it's just the texture name.
    vr::sys::Texture_t {
        handle: name.into_u32() as usize as *const c_void as *mut c_void,
        eType: vr::sys::ETextureType_TextureType_OpenGL,
        eColorSpace: vr::sys::EColorSpace_ColorSpace_Gamma, // TODO(mickvangelderen): IDK
    }
}
