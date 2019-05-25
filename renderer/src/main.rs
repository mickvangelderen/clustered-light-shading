#![allow(non_snake_case)]

// Has to go first.
#[macro_use]
mod macros;

mod ao_filter;
mod ao_renderer;
mod basic_renderer;
pub mod camera;
mod cgmath_ext;
pub mod clamp;
mod configuration;
mod convert;
mod filters;
pub mod frustrum;
mod gl_ext;
mod keyboard_model;
mod light;
mod line_renderer;
mod overlay_renderer;
mod post_renderer;
mod random_unit_sphere_dense;
mod random_unit_sphere_surface;
mod rendering;
mod resources;
mod shadow_renderer;
mod vsm_filter;

use crate::clamp::Clamp;
use crate::gl_ext::*;
use arrayvec::ArrayVec;
use cgmath::*;
use convert::*;
use gl_typed as gl;
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
    smooth_camera: camera::Camera,
    sun_pos: Vector3<f32>,
    sun_rot: Rad<f32>,
    use_smooth_camera: bool,
    keyboard_model: keyboard_model::KeyboardModel,
}

impl World {
    fn get_camera(&self) -> &camera::Camera {
        if self.use_smooth_camera {
            &self.smooth_camera
        } else {
            &self.camera
        }
    }
}

pub struct ViewIndependentResources {
    // Main shadow resources.
    pub shadow_framebuffer_name: gl::NonDefaultFramebufferName,
    pub shadow_texture: Texture<gl::symbols::Texture2D, gl::symbols::Rg32f>,
    pub shadow_depth_renderbuffer_name: gl::RenderbufferName,
    // Filter shadow resources.
    pub shadow_2_framebuffer_name: gl::NonDefaultFramebufferName,
    pub shadow_2_texture: Texture<gl::symbols::Texture2D, gl::symbols::Rg32f>,
    // Storage buffers.
    pub cls_buffer_name: gl::BufferName,
}

impl ViewIndependentResources {
    pub fn new(gl: &gl::Gl, width: i32, height: i32) -> Self {
        unsafe {
            // Renderbuffers.

            let shadow_depth_renderbuffer_name = gl.create_renderbuffer();
            gl.named_renderbuffer_storage(shadow_depth_renderbuffer_name, gl::DEPTH_COMPONENT32F, width, height);

            // Textures.

            let max_anisotropy = gl.get_max_texture_max_anisotropy();

            let texture_update = TextureUpdate::new()
                .data(width, height, None)
                .min_filter(gl::NEAREST.into())
                .mag_filter(gl::NEAREST.into())
                .wrap_s(gl::CLAMP_TO_EDGE.into())
                .wrap_t(gl::CLAMP_TO_EDGE.into());

            let shadow_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RG32F);
            shadow_texture.update(gl, texture_update.max_anisotropy(max_anisotropy));

            let shadow_2_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RG32F);
            shadow_2_texture.update(gl, texture_update.max_level(0));

            // Framebuffers.

            let shadow_framebuffer_name = gl.create_framebuffer();
            gl.bind_framebuffer(gl::FRAMEBUFFER, shadow_framebuffer_name);

            gl.named_framebuffer_texture(shadow_framebuffer_name, gl::COLOR_ATTACHMENT0, shadow_texture.name(), 0);

            gl.named_framebuffer_renderbuffer(
                shadow_framebuffer_name,
                gl::DEPTH_ATTACHMENT,
                gl::RENDERBUFFER,
                shadow_depth_renderbuffer_name,
            );

            assert_eq!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE.into()
            );

            let shadow_2_framebuffer_name = gl.create_framebuffer();

            gl.named_framebuffer_texture(
                shadow_2_framebuffer_name,
                gl::COLOR_ATTACHMENT0,
                shadow_2_texture.name(),
                0,
            );

            gl.named_framebuffer_renderbuffer(
                shadow_2_framebuffer_name,
                gl::DEPTH_ATTACHMENT,
                gl::RENDERBUFFER,
                shadow_depth_renderbuffer_name,
            );

            assert_eq!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE.into()
            );

            // Storage buffers,

            let cls_buffer_name = gl.create_buffer();

            ViewIndependentResources {
                shadow_framebuffer_name,
                shadow_texture,
                shadow_depth_renderbuffer_name,
                shadow_2_framebuffer_name,
                shadow_2_texture,
                cls_buffer_name,
            }
        }
    }
}

pub struct ViewDependentResources {
    // Main frame resources.
    pub framebuffer_name: gl::NonDefaultFramebufferName,
    pub color_texture: Texture<gl::symbols::Texture2D, gl::symbols::Rgba8>,
    pub depth_texture: Texture<gl::symbols::Texture2D, gl::symbols::Depth24Stencil8>,
    pub nor_in_cam_texture: Texture<gl::symbols::Texture2D, gl::symbols::R11fG11fB10f>,
    // AO resources.
    pub ao_framebuffer_name: gl::NonDefaultFramebufferName,
    pub ao_x_framebuffer_name: gl::NonDefaultFramebufferName,
    pub ao_texture: Texture<gl::symbols::Texture2D, gl::symbols::R8>,
    pub ao_x_texture: Texture<gl::symbols::Texture2D, gl::symbols::R8>,
    pub ao_depth_renderbuffer_name: gl::RenderbufferName,
    // Post resources.
    pub post_framebuffer_name: gl::NonDefaultFramebufferName,
    pub post_color_texture: Texture<gl::symbols::Texture2D, gl::symbols::Rgba8>,
    pub post_depth_texture: Texture<gl::symbols::Texture2D, gl::symbols::Depth24Stencil8>,
    // Uniform buffers.
    pub lighting_buffer_name: gl::BufferName,
}

impl ViewDependentResources {
    pub fn new(gl: &gl::Gl, width: i32, height: i32) -> Self {
        unsafe {
            // Renderbuffers.
            let ao_depth_renderbuffer_name = gl.create_renderbuffer();
            gl.named_renderbuffer_storage(ao_depth_renderbuffer_name, gl::DEPTH24_STENCIL8, width, height);

            // Textures.
            let texture_update = TextureUpdate::new()
                .data(width, height, None)
                .min_filter(gl::NEAREST.into())
                .mag_filter(gl::NEAREST.into())
                .max_level(0)
                .wrap_s(gl::CLAMP_TO_EDGE.into())
                .wrap_t(gl::CLAMP_TO_EDGE.into());

            let color_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RGBA8);
            color_texture.update(gl, texture_update);

            let nor_in_cam_texture = Texture::new(gl, gl::TEXTURE_2D, gl::R11F_G11F_B10F);
            nor_in_cam_texture.update(gl, texture_update);

            let depth_texture = Texture::new(gl, gl::TEXTURE_2D, gl::DEPTH24_STENCIL8);
            depth_texture.update(gl, texture_update);

            let ao_texture = Texture::new(gl, gl::TEXTURE_2D, gl::R8);
            ao_texture.update(gl, texture_update);

            let ao_x_texture = Texture::new(gl, gl::TEXTURE_2D, gl::R8);
            ao_x_texture.update(gl, texture_update);

            let post_color_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RGBA8);
            post_color_texture.update(gl, texture_update);

            let post_depth_texture = Texture::new(gl, gl::TEXTURE_2D, gl::DEPTH24_STENCIL8);
            post_depth_texture.update(gl, texture_update);

            // Framebuffers.

            let framebuffer_name = gl.create_framebuffer();
            gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT0, color_texture.name(), 0);

            gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT1, nor_in_cam_texture.name(), 0);

            gl.named_framebuffer_texture(framebuffer_name, gl::DEPTH_STENCIL_ATTACHMENT, depth_texture.name(), 0);
            assert_eq!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE.into()
            );

            let ao_framebuffer_name = gl.create_framebuffer().into();

            gl.named_framebuffer_texture(ao_framebuffer_name, gl::COLOR_ATTACHMENT0, ao_texture.name(), 0);

            gl.named_framebuffer_renderbuffer(
                ao_framebuffer_name,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                ao_depth_renderbuffer_name,
            );

            assert_eq!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE.into()
            );

            let ao_x_framebuffer_name = gl.create_framebuffer().into();

            gl.named_framebuffer_texture(ao_x_framebuffer_name, gl::COLOR_ATTACHMENT0, ao_x_texture.name(), 0);

            gl.named_framebuffer_renderbuffer(
                ao_x_framebuffer_name,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                ao_depth_renderbuffer_name,
            );

            assert_eq!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE.into()
            );

            let post_framebuffer_name = gl.create_framebuffer();

            gl.named_framebuffer_texture(
                post_framebuffer_name,
                gl::COLOR_ATTACHMENT0,
                post_color_texture.name(),
                0,
            );

            gl.named_framebuffer_texture(
                post_framebuffer_name,
                gl::DEPTH_STENCIL_ATTACHMENT,
                post_depth_texture.name(),
                0,
            );

            assert_eq!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE.into()
            );

            // Uniform block buffers,

            let lighting_buffer_name = gl.create_buffer();

            ViewDependentResources {
                framebuffer_name,
                color_texture,
                nor_in_cam_texture,
                depth_texture,
                ao_framebuffer_name,
                ao_x_framebuffer_name,
                ao_texture,
                ao_x_texture,
                ao_depth_renderbuffer_name,
                post_framebuffer_name,
                post_color_texture,
                post_depth_texture,
                lighting_buffer_name,
            }
        }
    }

    pub fn resize(&self, gl: &gl::Gl, width: i32, height: i32) {
        unsafe {
            gl.named_renderbuffer_storage(self.ao_depth_renderbuffer_name, gl::DEPTH24_STENCIL8, width, height);

            let texture_update = TextureUpdate::new().data(width, height, None);
            self.color_texture.update(gl, texture_update);
            self.depth_texture.update(gl, texture_update);
            self.nor_in_cam_texture.update(gl, texture_update);
            self.ao_texture.update(gl, texture_update);
            self.ao_x_texture.update(gl, texture_update);
            self.post_color_texture.update(gl, texture_update);
            self.post_depth_texture.update(gl, texture_update);
        }
    }

    pub fn drop(self, gl: &gl::Gl) {
        unsafe {
            gl.delete_framebuffer(self.framebuffer_name);
            gl.delete_framebuffer(self.ao_framebuffer_name);
            gl.delete_renderbuffer(self.ao_depth_renderbuffer_name);
            self.color_texture.drop(gl);
            self.depth_texture.drop(gl);
            self.nor_in_cam_texture.drop(gl);
            self.ao_texture.drop(gl);
            self.ao_x_texture.drop(gl);
            self.post_color_texture.drop(gl);
            self.post_depth_texture.drop(gl);
        }
    }
}

const DEPTH_RANGE: (f64, f64) = (1.0, 0.0);

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let resource_dir: PathBuf = [current_dir.as_ref(), Path::new("resources")].into_iter().collect();
    let log_dir: PathBuf = [current_dir.as_ref(), Path::new("logs")].into_iter().collect();
    let configuration_path = resource_dir.join(configuration::FILE_PATH);
    let shadow_renderer_vs_path = resource_dir.join("shadow_renderer.vert");
    let shadow_renderer_fs_path = resource_dir.join("shadow_renderer.frag");
    let vsm_filter_vs_path = resource_dir.join("vsm_filter.vert");
    let vsm_filter_fs_path = resource_dir.join("vsm_filter.frag");
    let ao_filter_vs_path = resource_dir.join("ao_filter.vert");
    let ao_filter_fs_path = resource_dir.join("ao_filter.frag");
    let basic_renderer_vs_path = resource_dir.join("basic_renderer.vert");
    let basic_renderer_fs_path = resource_dir.join("basic_renderer.frag");
    let ao_renderer_vs_path = resource_dir.join("ao_renderer.vert");
    let ao_renderer_fs_path = resource_dir.join("ao_renderer.frag");
    let post_renderer_vs_path = resource_dir.join("post_renderer.vert");
    let post_renderer_fs_path = resource_dir.join("post_renderer.frag");
    let overlay_renderer_vs_path = resource_dir.join("overlay_renderer.vert");
    let overlay_renderer_fs_path = resource_dir.join("overlay_renderer.frag");
    let line_renderer_vs_path = resource_dir.join("line_renderer.vert");
    let line_renderer_fs_path = resource_dir.join("line_renderer.frag");

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

    watcher.watch("resources", notify::RecursiveMode::Recursive).unwrap();

    let mut world = World {
        time: 0.0,
        clear_color: [0.0, 0.0, 0.0],
        camera: camera::Camera {
            position: Vector3::new(0.0, 1.0, 1.5),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            fovy: Deg(90.0).into(),
            positional_velocity: 2.0,
            angular_velocity: 0.4,
            zoom_velocity: 1.0,
        },
        smooth_camera: camera::Camera {
            position: Vector3::new(0.0, 1.0, 1.5),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            fovy: Deg(90.0).into(),
            positional_velocity: 2.0,
            angular_velocity: 0.4,
            zoom_velocity: 1.0,
        },
        sun_pos: Vector3::new(0.0, 0.0, 0.0),
        sun_rot: Deg(85.2).into(),
        use_smooth_camera: true,
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

        // NOTE: This alignment is hardcoded in rendering.rs.
        assert_eq!(256, gl.get_uniform_buffer_offset_alignment());
        assert_eq!(0, std::mem::size_of::<rendering::GlobalData>() % 256);
        assert_eq!(0, std::mem::size_of::<rendering::ViewData>() % 256);
        assert_eq!(0, std::mem::size_of::<rendering::MaterialData>() % 256);

        // Reverse-Z.
        gl.clip_control(gl::LOWER_LEFT, gl::ZERO_TO_ONE);
        gl.depth_func(gl::GT);
    }

    let glutin::dpi::PhysicalSize { width, height } = win_size.to_physical(win_dpi);

    let view_ind_res = ViewIndependentResources::new(&gl, SHADOW_W, SHADOW_H);
    let view_dep_res = ViewDependentResources::new(&gl, width as i32, height as i32);

    let random_unit_sphere_surface_texture = Texture::new(&gl, gl::TEXTURE_2D, gl::RGB8);
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
            &rendering::VSFSProgramUpdate {
                vertex_shader: Some(vs_bytes),
                fragment_shader: Some(fs_bytes),
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
            &rendering::VSFSProgramUpdate {
                vertex_shader: Some(vs_bytes),
                fragment_shader: Some(fs_bytes),
            },
        );
        vsm_filter
    };

    let mut ao_filter = {
        let mut ao_filter = ao_filter::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&ao_filter_vs_path).unwrap();
        let fs_bytes = std::fs::read(&ao_filter_fs_path).unwrap();
        ao_filter.update(
            &gl,
            &rendering::VSFSProgramUpdate {
                vertex_shader: Some(vs_bytes),
                fragment_shader: Some(fs_bytes),
            },
        );
        ao_filter
    };

    let mut basic_renderer = {
        let mut basic_renderer = basic_renderer::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&basic_renderer_vs_path).unwrap();
        let fs_bytes = std::fs::read(&basic_renderer_fs_path).unwrap();
        basic_renderer.update(
            &gl,
            &rendering::VSFSProgramUpdate {
                vertex_shader: Some(vs_bytes),
                fragment_shader: Some(fs_bytes),
            },
        );
        basic_renderer
    };

    let mut ao_renderer = {
        let mut ao_renderer = ao_renderer::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&ao_renderer_vs_path).unwrap();
        let fs_bytes = std::fs::read(&ao_renderer_fs_path).unwrap();
        ao_renderer.update(
            &gl,
            &rendering::VSFSProgramUpdate {
                vertex_shader: Some(vs_bytes),
                fragment_shader: Some(fs_bytes),
            },
        );
        ao_renderer
    };

    let mut post_renderer = {
        let mut post_renderer = post_renderer::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&post_renderer_vs_path).unwrap();
        let fs_bytes = std::fs::read(&post_renderer_fs_path).unwrap();
        post_renderer.update(
            &gl,
            &rendering::VSFSProgramUpdate {
                vertex_shader: Some(vs_bytes),
                fragment_shader: Some(fs_bytes),
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
            &rendering::VSFSProgramUpdate {
                vertex_shader: Some(vs_bytes),
                fragment_shader: Some(fs_bytes),
            },
        );
        overlay_renderer
    };

    let mut line_renderer = {
        let mut line_renderer = line_renderer::Renderer::new(&gl);
        let vs_bytes = std::fs::read(&line_renderer_vs_path).unwrap();
        let fs_bytes = std::fs::read(&line_renderer_fs_path).unwrap();
        line_renderer.update(
            &gl,
            &rendering::VSFSProgramUpdate {
                vertex_shader: Some(vs_bytes),
                fragment_shader: Some(fs_bytes),
            },
        );
        line_renderer
    };

    let mut configuration: configuration::Root = Default::default();

    let resources = resources::Resources::new(&gl, &resource_dir);

    let global_resources = rendering::GlobalResources::new(&gl);
    global_resources.bind(&gl);

    let material_resources = rendering::MaterialResources::new(&gl);
    let material_datas: Vec<rendering::MaterialData> = resources
        .materials
        .iter()
        .map(|mat| rendering::MaterialData {
            shininess: mat.shininess,
        })
        .collect();
    println!("material_datas len {}", material_datas.len());
    material_resources.write_all(&gl, &material_datas);
    drop(material_datas);

    let view_resources = rendering::ViewResources::new(&gl);

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
        {
            let mut configuration_update = false;
            let mut shadow_renderer_update = rendering::VSFSProgramUpdate::default();
            let mut vsm_filter_update = rendering::VSFSProgramUpdate::default();
            let mut ao_filter_update = rendering::VSFSProgramUpdate::default();
            let mut basic_renderer_update = rendering::VSFSProgramUpdate::default();
            let mut ao_renderer_update = rendering::VSFSProgramUpdate::default();
            let mut post_renderer_update = rendering::VSFSProgramUpdate::default();
            let mut overlay_renderer_update = rendering::VSFSProgramUpdate::default();
            let mut line_renderer_update = rendering::VSFSProgramUpdate::default();

            for event in rx_fs.try_iter() {
                if let Some(ref path) = event.path {
                    // println!("Detected file change in {:?}", path.display());
                    match path {
                        path if path == &configuration_path => {
                            configuration_update = true;
                        }
                        path if path == &shadow_renderer_vs_path => {
                            shadow_renderer_update.vertex_shader = Some(std::fs::read(&path).unwrap());
                        }
                        path if path == &shadow_renderer_fs_path => {
                            shadow_renderer_update.fragment_shader = Some(std::fs::read(&path).unwrap());
                        }
                        path if path == &vsm_filter_vs_path => {
                            vsm_filter_update.vertex_shader = Some(std::fs::read(&path).unwrap());
                        }
                        path if path == &vsm_filter_fs_path => {
                            vsm_filter_update.fragment_shader = Some(std::fs::read(&path).unwrap());
                        }
                        path if path == &ao_filter_vs_path => {
                            ao_filter_update.vertex_shader = Some(std::fs::read(&path).unwrap());
                        }
                        path if path == &ao_filter_fs_path => {
                            ao_filter_update.fragment_shader = Some(std::fs::read(&path).unwrap());
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
                            overlay_renderer_update.fragment_shader = Some(std::fs::read(&path).unwrap());
                        }
                        path if path == &line_renderer_vs_path => {
                            line_renderer_update.vertex_shader = Some(std::fs::read(&path).unwrap());
                        }
                        path if path == &line_renderer_fs_path => {
                            line_renderer_update.fragment_shader = Some(std::fs::read(&path).unwrap());
                        }
                        _ => {}
                    }
                }
            }

            if configuration_update {
                match std::fs::read_to_string(&configuration_path) {
                    Ok(contents) => match toml::from_str(&contents) {
                        Ok(new_configuration) => {
                            println!("Updated configuration {:#?}", new_configuration);
                            configuration = new_configuration;
                        }
                        Err(err) => eprintln!("Failed to parse configuration file {:?}: {}.", configuration_path, err),
                    },
                    Err(err) => eprintln!("Failed to read configuration file {:?}: {}.", configuration_path, err),
                }
            }

            shadow_renderer.update(&gl, &shadow_renderer_update);
            vsm_filter.update(&gl, &vsm_filter_update);
            ao_filter.update(&gl, &ao_filter_update);
            basic_renderer.update(&gl, &basic_renderer_update);
            ao_renderer.update(&gl, &ao_renderer_update);
            post_renderer.update(&gl, &post_renderer_update);
            overlay_renderer.update(&gl, &overlay_renderer_update);
            line_renderer.update(&gl, &line_renderer_update);
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
                                            world.use_smooth_camera = !world.use_smooth_camera;
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
            println!("win_size: {:?}", glutin::dpi::PhysicalSize { width, height });
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
            delta_yaw: Rad(-mouse_dx as f32),
            delta_pitch: Rad(-mouse_dy as f32),
            delta_scroll: mouse_dscroll as f32,
        });
        world.smooth_camera.interpolate(&world.camera, 0.80);

        if vr_resources.is_some() {
            // Pitch makes me dizzy.
            world.camera.pitch = Rad(0.0);
            world.smooth_camera.pitch = Rad(0.0);
        }

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

        // Render the scene.

        let pos_from_hmd_to_bdy: Option<Matrix4<f64>> = vr_resources.as_ref().map(|vr_resources| {
            // TODO: Is this the right place to put this?
            while let Some(_event) = vr_resources.system().poll_next_event() {
                // println!("{:?}", &event);
            }

            let mut poses: [vr::sys::TrackedDevicePose_t; vr::sys::k_unMaxTrackedDeviceCount as usize] =
                unsafe { mem::zeroed() };

            vr_resources.compositor().wait_get_poses(&mut poses[..], None).unwrap();

            const HMD_POSE_INDEX: usize = vr::sys::k_unTrackedDeviceIndex_Hmd as usize;
            let hmd_pose = poses[HMD_POSE_INDEX];
            if hmd_pose.bPoseIsValid {
                Matrix4::from_hmd(hmd_pose.mDeviceToAbsoluteTracking.m).cast().unwrap()
            } else {
                panic!("Pose is not valid!");
            }
        });

        let pose_end_render_start_nanos = start_instant.elapsed().as_nanos() as u64;

        let pos_from_hmd_to_wld = {
            // TODO: Rename camera functions. Something like from_pnt_to_chd?
            let pos_from_bdy_to_wld = world.camera.pos_from_cam_to_wld().cast().unwrap();

            match pos_from_hmd_to_bdy {
                Some(pos_from_hmd_to_bdy) => pos_from_bdy_to_wld * pos_from_hmd_to_bdy,
                None => {
                    pos_from_bdy_to_wld /* pos_from_hmd_to_bdy = I */
                }
            }
        };
        let pos_from_wld_to_hmd = pos_from_hmd_to_wld.invert().unwrap();

        // Shadowing.
        let sun_frustrum = frustrum::Frustrum {
            x0: -25.0,
            x1: 25.0,
            y0: -25.0,
            y1: 25.0,
            z0: 30.0,
            z1: -30.0,
        };

        let (sun_frustrum_vertices, sun_frustrum_indices) = sun_frustrum.cast().unwrap().line_mesh();

        struct View {
            pos_from_wld_to_cam: Matrix4<f64>,
            pos_from_cam_to_wld: Matrix4<f64>,

            pos_from_cam_to_clp: Matrix4<f64>,
            pos_from_clp_to_cam: Matrix4<f64>,
        }

        let views: ArrayVec<[View; 2]> = if let Some(ref vr_resources) = vr_resources {
            EYES.into_iter()
                .map(|&eye| {
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
                    let pos_from_cam_to_clp = frustrum.perspective(DEPTH_RANGE);

                    let pos_from_cam_to_hmd: Matrix4<f64> =
                        Matrix4::from_hmd(vr_resources.context.system().get_eye_to_head_transform(eye))
                            .cast()
                            .unwrap();

                    let pos_from_cam_to_wld = pos_from_hmd_to_wld * pos_from_cam_to_hmd;

                    View {
                        pos_from_wld_to_cam: pos_from_cam_to_wld.invert().unwrap(),
                        pos_from_cam_to_wld,

                        pos_from_cam_to_clp,
                        pos_from_clp_to_cam: pos_from_cam_to_clp.invert().unwrap(),
                    }
                })
                .collect()
        } else {
            let physical_size = win_size.to_physical(win_dpi);
            let frustrum = {
                let z0 = -0.1;
                let dy = -z0 * Rad::tan(Rad(Rad::from(world.get_camera().fovy).0 as f64) / 2.0);
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
            let pos_from_cam_to_clp = frustrum.perspective(DEPTH_RANGE);
            let mut views = ArrayVec::new();
            views.push(View {
                pos_from_wld_to_cam: /* pos_from_hmd_to_cam = I */ pos_from_wld_to_hmd,
                pos_from_cam_to_wld: /* pos_from_cam_to_hmd = I */ pos_from_hmd_to_wld,

                pos_from_cam_to_clp,
                pos_from_clp_to_cam: pos_from_cam_to_clp.invert().unwrap(),
            });
            views
        };

        let global_data = {
            let pos_from_cam_to_clp = sun_frustrum.orthographic(DEPTH_RANGE).cast().unwrap();

            let rot_from_wld_to_cam = Quaternion::from_angle_x(world.sun_rot) * Quaternion::from_angle_y(Deg(40.0));

            let pos_from_wld_to_cam = Matrix4::from(rot_from_wld_to_cam) * Matrix4::from_translation(-world.sun_pos);
            let pos_from_cam_to_wld =
                Matrix4::from_translation(world.sun_pos) * Matrix4::from(rot_from_wld_to_cam.invert());

            let pos_from_wld_to_clp: Matrix4<f32> = (pos_from_cam_to_clp * pos_from_wld_to_cam.cast().unwrap())
                .cast()
                .unwrap();

            // Clustered light shading.

            let cluster_bounding_box = {
                let corners_in_clp = frustrum::Frustrum::corners_in_clp(DEPTH_RANGE);
                let mut corners_in_cam = views
                    .iter()
                    .flat_map(|view| {
                        corners_in_clp
                            .into_iter()
                            .map(move |&p| view.pos_from_clp_to_cam.transform_point(p))
                    })
                    .map(|p| -> Point3<f32> { p.cast().unwrap() });
                let first = frustrum::BoundingBox::from_point(corners_in_cam.next().unwrap());
                corners_in_cam.fold(first, |b, p| b.enclose(p))
            };

            let cluster_side = configuration.clustered_light_shading.cluster_side;
            let cbb_dx = cluster_bounding_box.x1 - cluster_bounding_box.x0;
            let cbb_dy = cluster_bounding_box.y1 - cluster_bounding_box.y0;
            let cbb_dz = cluster_bounding_box.z1 - cluster_bounding_box.z0;
            let cbb_cx = f32::ceil(cbb_dx / cluster_side);
            let cbb_cy = f32::ceil(cbb_dy / cluster_side);
            let cbb_cz = f32::ceil(cbb_dz / cluster_side);
            let cbb_sx = cbb_cx / cbb_dx;
            let cbb_sy = cbb_cy / cbb_dy;
            let cbb_sz = cbb_cz / cbb_dz;
            let cbb_sx_inv = cbb_dx / cbb_cx;
            let cbb_sy_inv = cbb_dy / cbb_cy;
            let cbb_sz_inv = cbb_dz / cbb_cz;
            let cbb_cx = cbb_cx as usize;
            let cbb_cy = cbb_cy as usize;
            let cbb_cz = cbb_cz as usize;
            let cbb_n = cbb_cx * cbb_cy * cbb_cz;
            let pos_from_cam_to_cls = Matrix4::from_nonuniform_scale(cbb_sx, cbb_sy, cbb_sz)
                * Matrix4::from_translation(Vector3::new(
                    -cluster_bounding_box.x0,
                    -cluster_bounding_box.y0,
                    -cluster_bounding_box.z0,
                ));
            let pos_from_wld_to_cls = pos_from_cam_to_cls * pos_from_wld_to_cam;

            let mut clustering: Vec<[u32; 16]> = (0..cbb_n).into_iter().map(|_| Default::default()).collect();

            // println!(
            //     "cluster x * y * z = {} * {} * {} = {} ({} MB)",
            //     cbb_cx,
            //     cbb_cy,
            //     cbb_cz,
            //     cbb_n,
            //     std::mem::size_of_val(&clustering[..]) as f32 / 1_000_000.0
            // );

            for (i, l) in resources.point_lights.iter().enumerate() {
                let pos_in_cls = pos_from_wld_to_cls.transform_point(l.pos_in_pnt);

                // NOTE: We must clamp as f32 because the value might actually overflow.
                let x0 = Clamp::clamp(f32::floor(pos_in_cls.x - l.radius * cbb_sx), (0.0, cbb_cx as f32)) as usize;
                let x1 = f32::floor(pos_in_cls.x) as usize;
                let x2 =
                    Clamp::clamp(f32::floor(pos_in_cls.x + l.radius * cbb_sx) + 1.0, (0.0, cbb_cx as f32)) as usize;
                let y0 = Clamp::clamp(f32::floor(pos_in_cls.y - l.radius * cbb_sy), (0.0, cbb_cy as f32)) as usize;
                let y1 = f32::floor(pos_in_cls.y) as usize;
                let y2 =
                    Clamp::clamp(f32::floor(pos_in_cls.y + l.radius * cbb_sy) + 1.0, (0.0, cbb_cy as f32)) as usize;
                let z0 = Clamp::clamp(f32::floor(pos_in_cls.z - l.radius * cbb_sz), (0.0, cbb_cz as f32)) as usize;
                let z1 = f32::floor(pos_in_cls.z) as usize;
                let z2 =
                    Clamp::clamp(f32::floor(pos_in_cls.z + l.radius * cbb_sz) + 1.0, (0.0, cbb_cz as f32)) as usize;

                // println!(
                //     "lights[{}] pos_in_cls: {:?}, radius: {}, x: ({}, {}, {}), y: ({}, {}, {}), z: ({}, {}, {})",
                //     i, pos_in_cls, l.radius, x0, x1, x2, y0, y1, y2, z0, z1, z2
                // );

                let r_sq = l.radius * l.radius;

                for z in z0..z2 {
                    let dz = if z < z1 {
                        pos_in_cls.z - (z + 1) as f32
                    } else if z > z1 {
                        pos_in_cls.z - z as f32
                    } else {
                        0.0
                    } * cbb_sz_inv;
                    for y in y0..y2 {
                        let dy = if y < y1 {
                            pos_in_cls.y - (y + 1) as f32
                        } else if y > y1 {
                            pos_in_cls.y - y as f32
                        } else {
                            0.0
                        } * cbb_sy_inv;
                        for x in x0..x2 {
                            let dx = if x < x1 {
                                pos_in_cls.x - (x + 1) as f32
                            } else if x > x1 {
                                pos_in_cls.x - x as f32
                            } else {
                                0.0
                            } * cbb_sx_inv;
                            if dz * dz + dy * dy + dx * dx < r_sq {
                                // It's a hit!
                                let thing = &mut clustering[((z * cbb_cy) + y) * cbb_cx + x];
                                thing[0] += 1;
                                let offset = thing[0] as usize;
                                if offset < thing.len() {
                                    thing[offset] = i as u32;
                                } else {
                                    eprintln!("Overflowing clustered light assignment!");
                                }
                            }
                        }
                    }
                }
            }

            let cls_header = light::CLSBufferHeader {
                cluster_dims: Vector4::new(cbb_cx as u32, cbb_cy as u32, cbb_cz as u32, 16),
            };

            unsafe {
                let header_size = std::mem::size_of::<light::CLSBufferHeader>();
                let body_size = std::mem::size_of_val(&clustering[..]);
                let total_size = header_size + body_size;
                gl.bind_buffer(gl::SHADER_STORAGE_BUFFER, view_ind_res.cls_buffer_name);
                gl.buffer_reserve(gl::SHADER_STORAGE_BUFFER, total_size, gl::STREAM_DRAW);
                gl.buffer_sub_data(gl::SHADER_STORAGE_BUFFER, 0, &[cls_header]);
                gl.buffer_sub_data(gl::SHADER_STORAGE_BUFFER, header_size, &clustering[..]);
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    rendering::CLS_BUFFER_BINDING,
                    view_ind_res.cls_buffer_name,
                );
                gl.unbind_buffer(gl::SHADER_STORAGE_BUFFER);
            }

            rendering::GlobalData {
                light_pos_from_wld_to_cam: pos_from_wld_to_cam,
                light_pos_from_cam_to_wld: pos_from_cam_to_wld,

                light_pos_from_cam_to_clp: pos_from_cam_to_clp,
                light_pos_from_clp_to_cam: pos_from_cam_to_clp.invert().unwrap(),

                light_pos_from_wld_to_clp: pos_from_wld_to_clp,
                light_pos_from_clp_to_wld: pos_from_wld_to_clp.invert().unwrap(),

                pos_from_wld_to_cls,
                pos_from_cls_to_wld: pos_from_wld_to_cls.invert().unwrap(),

                time: world.time,
                _pad0: [0.0; 3],
            }
        };

        global_resources.write(&gl, &global_data);

        // View independent.
        shadow_renderer.render(
            &gl,
            &shadow_renderer::Parameters {
                framebuffer: view_ind_res.shadow_framebuffer_name.into(),
                width: SHADOW_W,
                height: SHADOW_H,
            },
            &resources,
        );

        // View independent.
        vsm_filter.render(
            &gl,
            &vsm_filter::Parameters {
                width: SHADOW_W,
                height: SHADOW_H,
                framebuffer_x: view_ind_res.shadow_2_framebuffer_name.into(),
                framebuffer_xy: view_ind_res.shadow_framebuffer_name.into(),
                color: view_ind_res.shadow_texture.name(),
                color_x: view_ind_res.shadow_2_texture.name(),
            },
            &resources,
        );

        let view_datas: ArrayVec<[_; 2]> = views
            .into_iter()
            .map(|view| {
                let View {
                    pos_from_wld_to_cam,
                    pos_from_cam_to_wld,

                    pos_from_clp_to_cam,
                    pos_from_cam_to_clp,
                } = view;

                let pos_from_wld_to_clp = pos_from_cam_to_clp * pos_from_wld_to_cam;
                let pos_from_clp_to_wld = pos_from_cam_to_wld * pos_from_clp_to_cam;

                let light_pos_from_cam_to_wld: Matrix4<f64> = global_data.light_pos_from_cam_to_wld.cast().unwrap();
                let light_dir_in_cam =
                    pos_from_wld_to_cam.transform_vector(light_pos_from_cam_to_wld.transform_vector(Vector3::unit_z()));

                rendering::ViewData {
                    pos_from_wld_to_cam: pos_from_wld_to_cam.cast().unwrap(),
                    pos_from_cam_to_wld: pos_from_wld_to_cam.cast().unwrap(),

                    pos_from_cam_to_clp: pos_from_cam_to_clp.cast().unwrap(),
                    pos_from_clp_to_cam: pos_from_cam_to_clp.cast().unwrap(),

                    pos_from_wld_to_clp: pos_from_wld_to_clp.cast().unwrap(),
                    pos_from_clp_to_wld: pos_from_clp_to_wld.cast().unwrap(),

                    light_dir_in_cam: light_dir_in_cam.cast().unwrap(),
                    _pad0: 0.0,
                }
            })
            .collect();

        view_resources.write_all(&gl, &*view_datas);

        let physical_size = win_size.to_physical(win_dpi);

        if let Some(ref vr_resources) = vr_resources {
            // === VR ===
            let viewports = {
                let w = physical_size.width as i32;
                let h = physical_size.height as i32;
                [(0, w / 2, 0, h), (w / 2, w, 0, h)]
            };

            for (i, &eye) in EYES.into_iter().enumerate() {
                let viewport = viewports[i];
                let view_data = view_datas[i];
                view_resources.bind_index(&gl, i);

                let view_dep_res = &vr_resources[eye];

                unsafe {
                    let mut point_lights: [light::PointLightBufferEntry; rendering::POINT_LIGHT_CAPACITY as usize] =
                        std::mem::uninitialized();
                    for i in 0..rendering::POINT_LIGHT_CAPACITY as usize {
                        point_lights[i] = light::PointLightBufferEntry::from_point_light(
                            resources.point_lights[i],
                            view_data.pos_from_wld_to_cam,
                        );
                    }
                    let lighting_buffer = light::LightingBuffer { point_lights };
                    gl.bind_buffer(gl::UNIFORM_BUFFER, view_dep_res.lighting_buffer_name);
                    gl.buffer_data(gl::UNIFORM_BUFFER, lighting_buffer.as_ref(), gl::STREAM_DRAW);
                    gl.bind_buffer_base(
                        gl::UNIFORM_BUFFER,
                        rendering::LIGHTING_BUFFER_BINDING,
                        view_dep_res.lighting_buffer_name,
                    );
                    gl.unbind_buffer(gl::UNIFORM_BUFFER);
                }

                basic_renderer.render(
                    &gl,
                    &basic_renderer::Parameters {
                        framebuffer: view_dep_res.framebuffer_name.into(),
                        width: vr_resources.dims.width as i32,
                        height: vr_resources.dims.height as i32,
                        material_resources,
                        shadow_texture_name: view_ind_res.shadow_texture.name(),
                        shadow_texture_dimensions: [SHADOW_W as f32, SHADOW_H as f32],
                    },
                    &world,
                    &resources,
                );

                ao_renderer.render(
                    &gl,
                    &ao_renderer::Parameters {
                        framebuffer: view_dep_res.ao_framebuffer_name.into(),
                        width: vr_resources.dims.width as i32,
                        height: vr_resources.dims.height as i32,
                        color_texture_name: view_dep_res.color_texture.name(),
                        depth_texture_name: view_dep_res.depth_texture.name(),
                        nor_in_cam_texture_name: view_dep_res.nor_in_cam_texture.name(),
                        random_unit_sphere_surface_texture_name: random_unit_sphere_surface_texture.name(),
                    },
                    &world,
                    &resources,
                );

                ao_filter.render(
                    &gl,
                    &ao_filter::Parameters {
                        width: vr_resources.dims.width as i32,
                        height: vr_resources.dims.height as i32,
                        framebuffer_x: view_dep_res.ao_x_framebuffer_name.into(),
                        framebuffer_xy: view_dep_res.ao_framebuffer_name.into(),
                        color: view_dep_res.ao_texture.name(),
                        color_x: view_dep_res.ao_x_texture.name(),
                        depth: view_dep_res.depth_texture.name(),
                    },
                    &resources,
                );

                post_renderer.render(
                    &gl,
                    &post_renderer::Parameters {
                        framebuffer: view_dep_res.post_framebuffer_name.into(),
                        width: vr_resources.dims.width as i32,
                        height: vr_resources.dims.height as i32,
                        color_texture_name: view_dep_res.color_texture.name(),
                        depth_texture_name: view_dep_res.depth_texture.name(),
                        nor_in_cam_texture_name: view_dep_res.nor_in_cam_texture.name(),
                        ao_texture_name: view_dep_res.ao_texture.name(),
                    },
                    &world,
                    &resources,
                );

                overlay_renderer.render(
                    &gl,
                    &overlay_renderer::Parameters {
                        framebuffer: gl::FramebufferName::Default,
                        x0: viewport.0,
                        x1: viewport.1,
                        y0: viewport.2,
                        y1: viewport.3,
                        color_texture_name: view_dep_res.post_color_texture.name(),
                        default_colors: [0.0, 0.0, 0.0, 0.0],
                        color_matrix: [
                            [1.0, 0.0, 0.0, 0.0],
                            [0.0, 1.0, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [0.0, 0.0, 0.0, 1.0],
                        ],
                    },
                    &resources,
                );
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
        // --- VR ---
        } else {
            let view_data = view_datas[0];
            view_resources.bind_index(&gl, 0);

            unsafe {
                let mut point_lights: [light::PointLightBufferEntry; rendering::POINT_LIGHT_CAPACITY as usize] =
                    std::mem::uninitialized();
                for i in 0..rendering::POINT_LIGHT_CAPACITY as usize {
                    point_lights[i] = light::PointLightBufferEntry::from_point_light(
                        resources.point_lights[i],
                        view_data.pos_from_wld_to_cam,
                    );
                }
                let lighting_buffer = light::LightingBuffer { point_lights };
                gl.bind_buffer(gl::UNIFORM_BUFFER, view_dep_res.lighting_buffer_name);
                gl.buffer_data(gl::UNIFORM_BUFFER, lighting_buffer.as_ref(), gl::STREAM_DRAW);
                gl.bind_buffer_base(
                    gl::UNIFORM_BUFFER,
                    rendering::LIGHTING_BUFFER_BINDING,
                    view_dep_res.lighting_buffer_name,
                );
                gl.unbind_buffer(gl::UNIFORM_BUFFER);
            }

            basic_renderer.render(
                &gl,
                &basic_renderer::Parameters {
                    framebuffer: view_dep_res.framebuffer_name.into(),
                    width: physical_size.width as i32,
                    height: physical_size.height as i32,
                    material_resources,
                    shadow_texture_name: view_ind_res.shadow_texture.name(),
                    shadow_texture_dimensions: [SHADOW_W as f32, SHADOW_H as f32],
                },
                &world,
                &resources,
            );

            line_renderer.render(
                &gl,
                &line_renderer::Parameters {
                    framebuffer: view_dep_res.framebuffer_name.into(),
                    width: physical_size.width as i32,
                    height: physical_size.height as i32,
                    vertices: &sun_frustrum_vertices[..],
                    indices: &sun_frustrum_indices[..],
                    pos_from_obj_to_wld: &global_data.light_pos_from_cam_to_wld,
                },
            );

            ao_renderer.render(
                &gl,
                &ao_renderer::Parameters {
                    framebuffer: view_dep_res.ao_framebuffer_name.into(),
                    width: physical_size.width as i32,
                    height: physical_size.height as i32,
                    color_texture_name: view_dep_res.color_texture.name(),
                    depth_texture_name: view_dep_res.depth_texture.name(),
                    nor_in_cam_texture_name: view_dep_res.nor_in_cam_texture.name(),
                    random_unit_sphere_surface_texture_name: random_unit_sphere_surface_texture.name(),
                },
                &world,
                &resources,
            );

            ao_filter.render(
                &gl,
                &ao_filter::Parameters {
                    width: physical_size.width as i32,
                    height: physical_size.height as i32,
                    framebuffer_x: view_dep_res.ao_x_framebuffer_name.into(),
                    framebuffer_xy: view_dep_res.ao_framebuffer_name.into(),
                    color: view_dep_res.ao_texture.name(),
                    color_x: view_dep_res.ao_x_texture.name(),
                    depth: view_dep_res.depth_texture.name(),
                },
                &resources,
            );

            post_renderer.render(
                &gl,
                &post_renderer::Parameters {
                    framebuffer: gl::FramebufferName::Default,
                    width: physical_size.width as i32,
                    height: physical_size.height as i32,
                    color_texture_name: view_dep_res.color_texture.name(),
                    depth_texture_name: view_dep_res.depth_texture.name(),
                    nor_in_cam_texture_name: view_dep_res.nor_in_cam_texture.name(),
                    ao_texture_name: view_dep_res.ao_texture.name(),
                },
                &world,
                &resources,
            );

            // overlay_renderer.render(
            //     &gl,
            //     &overlay_renderer::Parameters {
            //         framebuffer: None,
            //         x0: 0,
            //         x1: (physical_size.height / 3.0) as i32,
            //         y0: 0,
            //         y1: (physical_size.height / 3.0) as i32,
            //         color_texture_name: view_ind_res.shadow_texture.name(),
            //         default_colors: [0.0, 0.0, 0.0, 1.0],
            //         color_matrix: [
            //             [1.0, 0.0, 0.0, 0.0],
            //             [1.0, 0.0, 0.0, 0.0],
            //             [1.0, 0.0, 0.0, 0.0],
            //             [0.0, 0.0, 0.0, 0.0],
            //         ],
            //     },
            // );
        }

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
            let fps = NANOS_PER_SEC / (duration.as_secs() as f32 * NANOS_PER_SEC + duration.subsec_nanos() as f32);
            fps_average.submit(fps);
            gl_window
                .window()
                .set_title(&format!("VR Lab - {:02.1} FPS", fps_average.compute()));
        }
    }

    drop(tx_log);

    timing_thread.join().unwrap();
}

struct RenderPass {
    pub framebuffer_name: gl::FramebufferName,
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
