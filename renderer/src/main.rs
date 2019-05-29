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
mod glutin_ext;
mod keyboard;
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
use glutin_ext::*;
use keyboard::*;
use notify::Watcher;
use openvr as vr;
use openvr::enums::Enum;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time;

const DESIRED_UPS: f64 = 90.0;

const SHADOW_W: i32 = 1024;
const SHADOW_H: i32 = 1024;

const EYES: [vr::Eye; 2] = [vr::Eye::Left, vr::Eye::Right];

pub struct World {
    tick: u64,
    clear_color: [f32; 3],
    camera: camera::SmoothCamera,
    sun_pos: Vector3<f32>,
    sun_rot: Rad<f32>,
    keyboard_model: keyboard_model::KeyboardModel,
}

pub struct ViewIndependentResources {
    // Main shadow resources.
    pub shadow_framebuffer_name: gl::NonDefaultFramebufferName,
    pub shadow_texture: Texture<gl::TEXTURE_2D, gl::RG32F>,
    pub shadow_depth_renderbuffer_name: gl::RenderbufferName,
    // Filter shadow resources.
    pub shadow_2_framebuffer_name: gl::NonDefaultFramebufferName,
    pub shadow_2_texture: Texture<gl::TEXTURE_2D, gl::RG32F>,
    // Storage buffers.
    pub cls_resources: rendering::CLSResources,
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
                gl.check_named_framebuffer_status(shadow_framebuffer_name, gl::FRAMEBUFFER),
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
                gl.check_named_framebuffer_status(shadow_2_framebuffer_name, gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE.into()
            );

            ViewIndependentResources {
                shadow_framebuffer_name,
                shadow_texture,
                shadow_depth_renderbuffer_name,
                shadow_2_framebuffer_name,
                shadow_2_texture,
                cls_resources: rendering::CLSResources::new(gl),
            }
        }
    }
}

pub struct ViewDependentResources {
    pub width: i32,
    pub height: i32,
    // Main frame resources.
    pub framebuffer_name: gl::NonDefaultFramebufferName,
    pub color_texture: Texture<gl::TEXTURE_2D, gl::RGBA8>,
    pub depth_texture: Texture<gl::TEXTURE_2D, gl::DEPTH24_STENCIL8>,
    pub nor_in_cam_texture: Texture<gl::TEXTURE_2D, gl::R11F_G11F_B10F>,
    // AO resources.
    pub ao_framebuffer_name: gl::NonDefaultFramebufferName,
    pub ao_x_framebuffer_name: gl::NonDefaultFramebufferName,
    pub ao_texture: Texture<gl::TEXTURE_2D, gl::R8>,
    pub ao_x_texture: Texture<gl::TEXTURE_2D, gl::R8>,
    pub ao_depth_renderbuffer_name: gl::RenderbufferName,
    // Post resources.
    pub post_framebuffer_name: gl::NonDefaultFramebufferName,
    pub post_color_texture: Texture<gl::TEXTURE_2D, gl::RGBA8>,
    pub post_depth_texture: Texture<gl::TEXTURE_2D, gl::DEPTH24_STENCIL8>,
    // Uniform buffers.
    pub lighting_buffer_name: gl::BufferName,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Span<T> {
    pub start: T,
    pub end: T,
}

impl<T> Span<T>
where
    T: Copy + std::ops::Sub,
{
    fn delta(&self) -> <T as std::ops::Sub>::Output {
        self.end - self.start
    }
}

#[derive(Debug)]
pub struct Timings {
    pub accumulate_file_updates: Span<time::Instant>,
    pub execute_file_updates: Span<time::Instant>,
    pub wait_for_pose: Span<time::Instant>,
    pub accumulate_window_updates: Span<time::Instant>,
    pub accumulate_vr_updates: Span<time::Instant>,
    pub simulate: Span<time::Instant>,
    pub prepare_render_data: Span<time::Instant>,
    pub render: Span<time::Instant>,
    pub swap_buffers: Span<time::Instant>,
}

impl Timings {
    fn print_deltas(&self) {
        println!(
            "accumulate_file_updates   {:4>}μs",
            self.accumulate_file_updates.delta().as_micros()
        );
        println!(
            "execute_file_updates      {:4>}μs",
            self.execute_file_updates.delta().as_micros()
        );
        println!(
            "wait_for_pose             {:4>}μs",
            self.wait_for_pose.delta().as_micros()
        );
        println!(
            "accumulate_window_updates {:4>}μs",
            self.accumulate_window_updates.delta().as_micros()
        );
        println!(
            "accumulate_vr_updates     {:4>}μs",
            self.accumulate_vr_updates.delta().as_micros()
        );
        println!("simulate                  {:4>}μs", self.simulate.delta().as_micros());
        println!(
            "prepare_render_data       {:4>}μs",
            self.prepare_render_data.delta().as_micros()
        );
        println!("render                    {:4>}μs", self.render.delta().as_micros());
        println!(
            "swap_buffers              {:4>}μs",
            self.swap_buffers.delta().as_micros()
        );
    }
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
                gl.check_named_framebuffer_status(framebuffer_name, gl::FRAMEBUFFER),
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
                gl.check_named_framebuffer_status(ao_framebuffer_name, gl::FRAMEBUFFER),
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
                gl.check_named_framebuffer_status(ao_x_framebuffer_name, gl::FRAMEBUFFER),
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
                gl.check_named_framebuffer_status(post_framebuffer_name, gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE.into()
            );

            // Uniform block buffers,

            let lighting_buffer_name = gl.create_buffer();

            ViewDependentResources {
                width,
                height,
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

    pub fn resize(&mut self, gl: &gl::Gl, width: i32, height: i32) {
        unsafe {
            self.width = width;
            self.height = height;
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

pub fn read_configuration(configuration_path: &std::path::Path) -> configuration::Root {
    match std::fs::read_to_string(&configuration_path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(configuration) => configuration,
            Err(err) => {
                eprintln!("Failed to parse configuration file {:?}: {}.", configuration_path, err);
                Default::default()
            }
        },
        Err(err) => {
            eprintln!("Failed to read configuration file {:?}: {}.", configuration_path, err);
            Default::default()
        }
    }
}

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let resource_dir: PathBuf = [current_dir.as_ref(), Path::new("resources")].into_iter().collect();
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

    let (tx_fs, rx_fs) = mpsc::channel();

    let mut watcher = notify::raw_watcher(tx_fs).unwrap();

    watcher.watch("resources", notify::RecursiveMode::Recursive).unwrap();

    let mut configuration: configuration::Root = read_configuration(&configuration_path);

    let mut world = World {
        tick: 0,
        clear_color: [0.0, 0.0, 0.0],
        camera: camera::SmoothCamera::new(
            configuration.main_camera.maximum_smoothness,
            camera::Camera {
                properties: camera::CameraProperties {
                    positional_velocity: configuration.main_camera.positional_velocity,
                    angular_velocity: configuration.main_camera.angular_velocity,
                    zoom_velocity: configuration.main_camera.zoom_velocity,
                },
                state: camera::CameraState {
                    position: Vector3::new(0.0, 1.0, 1.5),
                    yaw: Rad(0.0),
                    pitch: Rad(0.0),
                    fovy: Deg(90.0).into(),
                },
            },
        ),
        sun_pos: Vector3::new(0.0, 0.0, 0.0),
        sun_rot: Deg(85.2).into(),
        keyboard_model: keyboard_model::KeyboardModel::new(),
    };

    let mut keyboard_state = KeyboardState::default();

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
        gl.depth_func(gl::GREATER);
    }

    let glutin::dpi::PhysicalSize { width, height } = win_size.to_physical(win_dpi);

    let view_ind_res = ViewIndependentResources::new(&gl, SHADOW_W, SHADOW_H);

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

    let vr_context = vr::Context::new(vr::ApplicationType::Scene)
        .map_err(|error| {
            eprintln!(
                "Failed to acquire context: {:?}",
                vr::InitError::from_unchecked(error).unwrap()
            );
        })
        .ok();

    let mut view_dep_res: ArrayVec<[ViewDependentResources; 2]> = match &vr_context {
        Some(context) => {
            let dims = context.system().get_recommended_render_target_size();

            ArrayVec::from(EYES)
                .into_iter()
                .map(|_| ViewDependentResources::new(&gl, dims.width as i32, dims.height as i32))
                .collect()
        }
        None => ArrayVec::from([ViewDependentResources::new(&gl, width as i32, height as i32)])
            .into_iter()
            .collect(),
    };

    let mut focus = false;
    let mut fps_average = filters::MovingAverageF32::new(0.0);
    let mut last_frame_start = time::Instant::now();

    let mut running = true;
    while running {
        macro_rules! timing_transition {
            ($timings: ident, $old: ident, $new: ident) => {
                $timings.$old.end = time::Instant::now();
                $timings.$new.start = $timings.$old.end;
            };
        }

        let mut timings: Timings = unsafe { std::mem::zeroed() };

        timings.accumulate_file_updates.start = time::Instant::now();
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

            timing_transition!(timings, accumulate_file_updates, execute_file_updates);

            if configuration_update {
                // Read from file.
                configuration = read_configuration(&configuration_path);

                // Apply updates.
                world.camera.maximum_smoothness = configuration.main_camera.maximum_smoothness;
                world.camera.properties = camera::CameraProperties {
                    positional_velocity: configuration.main_camera.positional_velocity,
                    angular_velocity: configuration.main_camera.angular_velocity,
                    zoom_velocity: configuration.main_camera.zoom_velocity,
                };
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

        timing_transition!(timings, execute_file_updates, wait_for_pose);

        // NOTE: OpenVR will block upon querying the pose for as long as
        // possible but no longer than it takes to submit the new frame. This is
        // done to render the most up-to-date application state as possible.

        let pos_from_hmd_to_bdy: Matrix4<f64> = match &vr_context {
            Some(vr_context) => {
                let mut poses: [vr::sys::TrackedDevicePose_t; vr::sys::k_unMaxTrackedDeviceCount as usize] =
                    unsafe { mem::zeroed() };

                vr_context.compositor().wait_get_poses(&mut poses[..], None).unwrap();

                let hmd_pose = poses[vr::sys::k_unTrackedDeviceIndex_Hmd as usize];
                if hmd_pose.bPoseIsValid {
                    Matrix4::from_hmd(hmd_pose.mDeviceToAbsoluteTracking.m).cast().unwrap()
                } else {
                    panic!("Pose is not valid!");
                }
            }
            None => Matrix4::identity(),
        };

        timing_transition!(timings, wait_for_pose, accumulate_window_updates);

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
                            keyboard_state.update(keyboard_input);

                            if let Some(vk) = keyboard_input.virtual_keycode {
                                // This has to update regardless of focus.
                                world.keyboard_model.process_event(vk, keyboard_input.state);

                                // use glutin::ElementState;
                                use glutin::VirtualKeyCode;
                                match vk {
                                    VirtualKeyCode::C => {
                                        if keyboard_input.state.is_pressed() && focus {
                                            world.camera.toggle_smoothness();
                                        }
                                    }
                                    VirtualKeyCode::Escape => {
                                        if keyboard_input.state.is_pressed() && focus {
                                            running = false;
                                        }
                                    }
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

        timing_transition!(timings, accumulate_window_updates, accumulate_vr_updates);

        if let Some(vr_context) = &vr_context {
            while let Some(_event) = vr_context.system().poll_next_event() {
                // TODO: Handle vr events.
            }
        }

        timing_transition!(timings, accumulate_vr_updates, simulate);

        if should_resize {
            let glutin::dpi::PhysicalSize { width, height } = win_size.to_physical(win_dpi);
            if vr_context.is_none() {
                view_dep_res[0].resize(&gl, width as i32, height as i32);
            }
        }

        let delta_time = 1.0 / DESIRED_UPS as f32;

        world.camera.update(&camera::CameraUpdate {
            delta_time,
            delta_position: if focus {
                Vector3::new(
                    keyboard_state.d.to_f32() - keyboard_state.a.to_f32(),
                    keyboard_state.q.to_f32() - keyboard_state.z.to_f32(),
                    keyboard_state.s.to_f32() - keyboard_state.w.to_f32(),
                ) * (1.0 + keyboard_state.lshift.to_f32() * 3.0)
            } else {
                Vector3::zero()
            },
            delta_yaw: Rad(-mouse_dx as f32),
            delta_pitch: Rad(-mouse_dy as f32),
            delta_fovy: Rad(mouse_dscroll as f32),
        });

        if vr_context.is_some() {
            // Pitch makes me dizzy.
            world.camera.target_state.pitch = Rad(0.0);
        }

        if focus {
            world.sun_rot += Rad(0.5) * (keyboard_state.up.to_f32() - keyboard_state.down.to_f32()) * delta_time;
        }

        world.keyboard_model.simulate(delta_time);

        world.tick += 1;

        timing_transition!(timings, simulate, prepare_render_data);

        // Space abbreviations:
        //  - world (wld)
        //  - camera body (bdy)
        //  - head (hmd)
        //  - clustered light shading (cls)
        //  - camera (cam)
        //  - clip (clp)
        //
        // Space relations:
        //  - wld --[camera position and orientation]--> bdy
        //  - bdy --[VR pose]--> hmd
        //  - hmd --[VR head to eye]--> cam
        //  - hmd --[clustered light shading dimensions]--> cls
        //  - cam --[projection]--> clp

        let pos_from_hmd_to_wld: Matrix4<f64> = {
            let pos_from_bdy_to_wld = world.camera.pos_to_parent().cast().unwrap();
            pos_from_bdy_to_wld * pos_from_hmd_to_bdy
        };
        let pos_from_wld_to_hmd = pos_from_hmd_to_wld.invert().unwrap();

        let sun_frustrum = frustrum::Frustrum::<f64> {
            x0: -25.0,
            x1: 25.0,
            y0: -25.0,
            y1: 25.0,
            z0: 30.0,
            z1: -30.0,
        };

        let (sun_frustrum_vertices, sun_frustrum_indices) = sun_frustrum.cast::<f32>().unwrap().line_mesh();

        struct View {
            pos_from_cam_to_hmd: Matrix4<f64>,

            pos_from_cam_to_clp: Matrix4<f64>,
            pos_from_clp_to_cam: Matrix4<f64>,

            pos_from_clp_to_hmd: Matrix4<f64>,
        }

        let views: ArrayVec<[View; 2]> = match &vr_context {
            Some(vr_context) => {
                ArrayVec::from(EYES)
                    .into_iter()
                    .map(|eye| {
                        let frustrum = {
                            // These are the tangents.
                            let [l, r, b, t] = vr_context.system().get_projection_raw(eye);
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
                        let pos_from_clp_to_cam = pos_from_cam_to_clp.invert().unwrap();

                        let pos_from_cam_to_hmd: Matrix4<f64> =
                            Matrix4::from_hmd(vr_context.system().get_eye_to_head_transform(eye))
                                .cast()
                                .unwrap();

                        View {
                            pos_from_cam_to_hmd,

                            pos_from_cam_to_clp,
                            pos_from_clp_to_cam,

                            pos_from_clp_to_hmd: pos_from_cam_to_hmd * pos_from_cam_to_clp.invert().unwrap(),
                        }
                    })
                    .collect()
            }
            None => {
                let physical_size = win_size.to_physical(win_dpi);

                let frustrum = {
                    let z0 = -0.1;
                    let dy = -z0 * Rad::tan(Rad(Rad::from(world.camera.current_state.fovy).0 as f64) / 2.0);
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
                let pos_from_clp_to_cam = pos_from_cam_to_clp.invert().unwrap();

                ArrayVec::from([View {
                    pos_from_cam_to_hmd: Matrix4::identity(),

                    pos_from_cam_to_clp,
                    pos_from_clp_to_cam,

                    pos_from_clp_to_hmd: pos_from_clp_to_cam,
                }])
                .into_iter()
                .collect()
            }
        };

        let global_data = {
            let light_pos_from_cam_to_clp = sun_frustrum.orthographic(DEPTH_RANGE).cast().unwrap();

            let light_rot_from_wld_to_cam =
                Quaternion::from_angle_x(world.sun_rot) * Quaternion::from_angle_y(Deg(40.0));

            let light_pos_from_wld_to_cam =
                Matrix4::from(light_rot_from_wld_to_cam) * Matrix4::from_translation(-world.sun_pos);
            let light_pos_from_cam_to_wld =
                Matrix4::from_translation(world.sun_pos) * Matrix4::from(light_rot_from_wld_to_cam.invert());

            let light_pos_from_wld_to_clp: Matrix4<f32> = (light_pos_from_cam_to_clp
                * light_pos_from_wld_to_cam.cast().unwrap())
            .cast()
            .unwrap();

            rendering::GlobalData {
                light_pos_from_wld_to_cam,
                light_pos_from_cam_to_wld,

                light_pos_from_cam_to_clp,
                light_pos_from_clp_to_cam: light_pos_from_cam_to_clp.invert().unwrap(),

                light_pos_from_wld_to_clp,
                light_pos_from_clp_to_wld: light_pos_from_wld_to_clp.invert().unwrap(),

                time: (world.tick as f64 / DESIRED_UPS) as f32,
                _pad0: [0.0; 3],
            }
        };

        // Clustered light shading.
        let (cls_header, clustering) = {
            let cluster_bounding_box = {
                let corners_in_clp = frustrum::Frustrum::corners_in_clp(DEPTH_RANGE);
                let mut corners_in_cam = views
                    .iter()
                    .flat_map(|view| {
                        corners_in_clp
                            .into_iter()
                            .map(move |&p| view.pos_from_clp_to_hmd.transform_point(p))
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
            let pos_from_hmd_to_cls = Matrix4::from_nonuniform_scale(cbb_sx, cbb_sy, cbb_sz)
                * Matrix4::from_translation(Vector3::new(
                    -cluster_bounding_box.x0,
                    -cluster_bounding_box.y0,
                    -cluster_bounding_box.z0,
                ));

            let pos_from_wld_to_cls: Matrix4<f64> = pos_from_hmd_to_cls.cast().unwrap() * pos_from_wld_to_hmd;
            let pos_from_cls_to_wld: Matrix4<f32> = pos_from_wld_to_cls.invert().unwrap().cast().unwrap();
            let pos_from_wld_to_cls: Matrix4<f32> = pos_from_wld_to_cls.cast().unwrap();

            let mut clustering: Vec<[u32; 16]> = (0..cbb_n).into_iter().map(|_| Default::default()).collect();

            println!(
                "cluster x * y * z = {} * {} * {} = {} ({} MB)",
                cbb_cx,
                cbb_cy,
                cbb_cz,
                cbb_n,
                std::mem::size_of_val(&clustering[..]) as f32 / 1_000_000.0
            );

            for (i, l) in resources.point_lights.iter().enumerate() {
                let pos_in_cls = pos_from_wld_to_cls.transform_point(l.pos_in_pnt);

                let r = l.attenuation.clip_far;
                let r_sq = r * r;

                // NOTE: We must clamp as f32 because the value might actually overflow.
                let x0 = Clamp::clamp_range(f32::floor(pos_in_cls.x - r * cbb_sx), (0.0, cbb_cx as f32)) as usize;
                let x1 = f32::floor(pos_in_cls.x) as usize;
                let x2 = Clamp::clamp_range(f32::floor(pos_in_cls.x + r * cbb_sx) + 1.0, (0.0, cbb_cx as f32)) as usize;
                let y0 = Clamp::clamp_range(f32::floor(pos_in_cls.y - r * cbb_sy), (0.0, cbb_cy as f32)) as usize;
                let y1 = f32::floor(pos_in_cls.y) as usize;
                let y2 = Clamp::clamp_range(f32::floor(pos_in_cls.y + r * cbb_sy) + 1.0, (0.0, cbb_cy as f32)) as usize;
                let z0 = Clamp::clamp_range(f32::floor(pos_in_cls.z - r * cbb_sz), (0.0, cbb_cz as f32)) as usize;
                let z1 = f32::floor(pos_in_cls.z) as usize;
                let z2 = Clamp::clamp_range(f32::floor(pos_in_cls.z + r * cbb_sz) + 1.0, (0.0, cbb_cz as f32)) as usize;

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

            let cls_header = rendering::CLSBufferHeader {
                cluster_dims: Vector4::new(cbb_cx as u32, cbb_cy as u32, cbb_cz as u32, 16),
                pos_from_wld_to_cls,
                pos_from_cls_to_wld,
            };

            (cls_header, clustering)
        };

        timing_transition!(timings, prepare_render_data, render);

        global_resources.write(&gl, &global_data);

        view_ind_res.cls_resources.write(&gl, &cls_header, &clustering);
        view_ind_res.cls_resources.bind(&gl);

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
                    pos_from_cam_to_hmd,

                    pos_from_cam_to_clp,
                    pos_from_clp_to_cam,
                    ..
                } = view;

                let pos_from_cam_to_wld = pos_from_hmd_to_wld * pos_from_cam_to_hmd;
                let pos_from_wld_to_cam = pos_from_cam_to_wld.invert().unwrap();

                let pos_from_wld_to_clp = pos_from_cam_to_clp * pos_from_wld_to_cam;
                let pos_from_clp_to_wld = pos_from_cam_to_wld * pos_from_clp_to_cam;

                let light_pos_from_cam_to_wld: Matrix4<f64> = global_data.light_pos_from_cam_to_wld.cast().unwrap();
                let light_dir_in_cam =
                    pos_from_wld_to_cam.transform_vector(light_pos_from_cam_to_wld.transform_vector(Vector3::unit_z()));

                rendering::ViewData {
                    pos_from_wld_to_cam: pos_from_wld_to_cam.cast().unwrap(),
                    pos_from_cam_to_wld: pos_from_cam_to_wld.cast().unwrap(),

                    pos_from_cam_to_clp: pos_from_cam_to_clp.cast().unwrap(),
                    pos_from_clp_to_cam: pos_from_clp_to_cam.cast().unwrap(),

                    pos_from_wld_to_clp: pos_from_wld_to_clp.cast().unwrap(),
                    pos_from_clp_to_wld: pos_from_clp_to_wld.cast().unwrap(),

                    light_dir_in_cam: light_dir_in_cam.cast().unwrap(),
                    _pad0: 0.0,
                }
            })
            .collect();

        view_resources.write_all(&gl, &*view_datas);

        let physical_size = win_size.to_physical(win_dpi);

        for (view_index, view_dep_res) in view_dep_res.iter().enumerate() {
            let view_data = view_datas[view_index];
            view_resources.bind_index(&gl, view_index);

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
                gl.named_buffer_data(
                    view_dep_res.lighting_buffer_name,
                    lighting_buffer.value_as_bytes(),
                    gl::STREAM_DRAW,
                );
                gl.bind_buffer_base(
                    gl::UNIFORM_BUFFER,
                    rendering::LIGHTING_BUFFER_BINDING,
                    view_dep_res.lighting_buffer_name,
                );
            }

            basic_renderer.render(
                &gl,
                &basic_renderer::Parameters {
                    framebuffer: view_dep_res.framebuffer_name.into(),
                    width: view_dep_res.width,
                    height: view_dep_res.height,
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
                    width: view_dep_res.width,
                    height: view_dep_res.height,
                    vertices: &sun_frustrum_vertices[..],
                    indices: &sun_frustrum_indices[..],
                    pos_from_obj_to_wld: &global_data.light_pos_from_cam_to_wld,
                },
            );

            ao_renderer.render(
                &gl,
                &ao_renderer::Parameters {
                    framebuffer: view_dep_res.ao_framebuffer_name.into(),
                    width: view_dep_res.width,
                    height: view_dep_res.height,
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
                    width: view_dep_res.width,
                    height: view_dep_res.height,
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
                    // FIXME: Hack, use two versions of view dependent parameters instead.
                    framebuffer: if vr_context.is_some() {
                        view_dep_res.post_framebuffer_name.into()
                    } else {
                        gl::FramebufferName::Default
                    },
                    width: view_dep_res.width,
                    height: view_dep_res.height,
                    color_texture_name: view_dep_res.color_texture.name(),
                    depth_texture_name: view_dep_res.depth_texture.name(),
                    nor_in_cam_texture_name: view_dep_res.nor_in_cam_texture.name(),
                    ao_texture_name: view_dep_res.ao_texture.name(),
                },
                &world,
                &resources,
            );
        }

        if let Some(vr_context) = &vr_context {
            let viewports = {
                let w = physical_size.width as i32;
                let h = physical_size.height as i32;
                [(0, w / 2, 0, h), (w / 2, w, 0, h)]
            };

            for (view_index, &eye) in EYES.iter().enumerate() {
                let view_dep_res = &view_dep_res[view_index];

                // Render both eyes to the default framebuffer.
                let viewport = viewports[view_index];
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

                // NOTE(mickvangelderen): Binding the color attachments causes SIGSEGV!!!
                let mut texture_t = gen_texture_t(view_dep_res.post_color_texture.name());
                vr_context
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

        timing_transition!(timings, render, swap_buffers);

        gl_window.swap_buffers().unwrap();

        timings.swap_buffers.end = time::Instant::now();

        timings.print_deltas();

        // std::thread::sleep(time::Duration::from_millis(17));

        {
            let duration = {
                let now = time::Instant::now();
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
