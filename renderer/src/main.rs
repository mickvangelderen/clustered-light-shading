#![feature(euclidean_division)]
#![allow(non_snake_case)]

// Has to go first.
#[macro_use]
mod macros;

pub(crate) use gl_typed as gl;
pub(crate) use incremental as ic;
pub(crate) use log::*;
pub(crate) use regex::{Regex, RegexBuilder};

// mod ao_filter;
// mod ao_renderer;
// mod basic_renderer;
mod bounding_box;
pub mod camera;
mod cgmath_ext;
pub mod clamp;
mod cls;
// mod cluster_renderer;
mod configuration;
mod convert;
mod filters;
pub mod frustrum;
mod gl_ext;
mod glutin_ext;
mod keyboard;
mod keyboard_model;
mod light;
// mod line_renderer;
mod mono_stereo;
// mod overlay_renderer;
// mod post_renderer;
mod random_unit_sphere_dense;
mod random_unit_sphere_surface;
mod rendering;
mod resources;
mod shadow_renderer;
mod timings;
mod viewport;
// mod vsm_filter;
mod window_mode;

use crate::bounding_box::*;
use crate::cgmath_ext::*;
use crate::frustrum::*;
use crate::gl_ext::*;
use crate::mono_stereo::*;
use crate::rendering::*;
use crate::resources::Resources;
use crate::timings::*;
use crate::viewport::*;
use crate::window_mode::*;
use arrayvec::ArrayVec;
use cgmath::*;
use convert::*;
use derive::EnumNext;
use glutin::GlContext;
use glutin_ext::*;
use keyboard::*;
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

impl_enum_and_enum_map! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
    enum CameraKey => struct CameraMap {
        Main => main,
        Debug => debug,
    }
}

impl CameraKey {
    pub const fn iter() -> CameraKeyIter {
        CameraKeyIter(Some(CameraKey::Main))
    }
}

pub struct CameraKeyIter(Option<CameraKey>);

impl Iterator for CameraKeyIter {
    type Item = CameraKey;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.0;
        self.0 = self.0.and_then(CameraKey::next);
        item
    }
}

impl std::iter::FusedIterator for CameraKeyIter {}

impl std::iter::ExactSizeIterator for CameraKeyIter {
    fn len(&self) -> usize {
        2
    }
}

use vr::Eye;

impl_enum_map! {
    Eye => struct EyeMap {
        Left => left,
        Right => right,
    }
}

pub const EYE_KEYS: [Eye; 2] = [Eye::Left, Eye::Right];

pub struct World {
    pub resource_dir: PathBuf,
    pub tick: u64,
    pub global: ic::Global,
    pub clear_color: [f32; 3],
    pub window_mode: WindowMode,
    pub render_technique: ic::Leaf<RenderTechnique>,
    pub render_technique_regex: Regex,
    pub attenuation_mode: ic::Leaf<AttenuationMode>,
    pub attenuation_mode_regex: Regex,
    pub gl_log_regex: Regex,
    pub sources: Vec<ShaderSource>,
    pub target_camera_key: CameraKey,
    pub transition_camera: camera::TransitionCamera,
    pub cameras: CameraMap<camera::SmoothCamera>,
    pub sun_pos: Vector3<f32>,
    pub sun_rot: Rad<f32>,
    pub keyboard_model: keyboard_model::KeyboardModel,
}

impl World {
    fn target_camera(&self) -> &camera::SmoothCamera {
        &self.cameras[self.target_camera_key]
    }

    fn target_camera_mut(&mut self) -> &mut camera::SmoothCamera {
        &mut self.cameras[self.target_camera_key]
    }
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

    pub global_resources: rendering::GlobalResources,
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

            gl.named_framebuffer_draw_buffers(shadow_framebuffer_name, &[gl::COLOR_ATTACHMENT0.into()]);

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

            gl.named_framebuffer_draw_buffers(shadow_2_framebuffer_name, &[gl::COLOR_ATTACHMENT0.into()]);

            assert_eq!(
                gl.check_named_framebuffer_status(shadow_2_framebuffer_name, gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE.into()
            );

            // Nested resources.

            let cls_resources = rendering::CLSResources::new(gl);
            cls_resources.bind(gl);

            let global_resources = rendering::GlobalResources::new(&gl);
            global_resources.bind(gl);

            ViewIndependentResources {
                shadow_framebuffer_name,
                shadow_texture,
                shadow_depth_renderbuffer_name,
                shadow_2_framebuffer_name,
                shadow_2_texture,
                global_resources,
                cls_resources,
            }
        }
    }
}

pub struct ViewDependentResources {
    pub width: i32,
    pub height: i32,
    // Main frame resources.
    pub main_framebuffer_name: gl::NonDefaultFramebufferName,
    pub main_color_texture: Texture<gl::TEXTURE_2D, gl::RGBA16F>,
    pub main_depth_texture: Texture<gl::TEXTURE_2D, gl::DEPTH24_STENCIL8>,
    pub main_nor_in_cam_texture: Texture<gl::TEXTURE_2D, gl::R11F_G11F_B10F>,
    // AO resources.
    pub ao_framebuffer_name: gl::NonDefaultFramebufferName,
    pub ao_texture: Texture<gl::TEXTURE_2D, gl::R8>,
    pub ao_x_framebuffer_name: gl::NonDefaultFramebufferName,
    pub ao_x_texture: Texture<gl::TEXTURE_2D, gl::R8>,
    pub ao_depth_renderbuffer_name: gl::RenderbufferName,
    // Post resources.
    pub post_framebuffer_name: gl::NonDefaultFramebufferName,
    pub post_color_texture: Texture<gl::TEXTURE_2D, gl::RGBA16F>,
    pub post_depth_texture: Texture<gl::TEXTURE_2D, gl::DEPTH24_STENCIL8>,
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

            let main_color_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RGBA16F);
            main_color_texture.update(gl, texture_update);

            let main_nor_in_cam_texture = Texture::new(gl, gl::TEXTURE_2D, gl::R11F_G11F_B10F);
            main_nor_in_cam_texture.update(gl, texture_update);

            let main_depth_texture = Texture::new(gl, gl::TEXTURE_2D, gl::DEPTH24_STENCIL8);
            main_depth_texture.update(gl, texture_update);

            let ao_texture = Texture::new(gl, gl::TEXTURE_2D, gl::R8);
            ao_texture.update(gl, texture_update);

            let ao_x_texture = Texture::new(gl, gl::TEXTURE_2D, gl::R8);
            ao_x_texture.update(gl, texture_update);

            let post_color_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RGBA16F);
            post_color_texture.update(gl, texture_update);

            let post_depth_texture = Texture::new(gl, gl::TEXTURE_2D, gl::DEPTH24_STENCIL8);
            post_depth_texture.update(gl, texture_update);

            // Framebuffers.

            let main_framebuffer_name = {
                let framebuffer_name = gl.create_framebuffer();
                gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT0, main_color_texture.name(), 0);
                gl.named_framebuffer_texture(
                    framebuffer_name,
                    gl::COLOR_ATTACHMENT1,
                    main_nor_in_cam_texture.name(),
                    0,
                );
                gl.named_framebuffer_texture(
                    framebuffer_name,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    main_depth_texture.name(),
                    0,
                );

                gl.named_framebuffer_draw_buffers(
                    framebuffer_name,
                    &[gl::COLOR_ATTACHMENT0.into(), gl::COLOR_ATTACHMENT1.into()],
                );
                assert_eq!(
                    gl.check_named_framebuffer_status(framebuffer_name, gl::FRAMEBUFFER),
                    gl::FRAMEBUFFER_COMPLETE.into()
                );
                framebuffer_name
            };

            let ao_framebuffer_name = {
                let framebuffer_name = gl.create_framebuffer().into();

                gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT0, ao_texture.name(), 0);

                gl.named_framebuffer_renderbuffer(
                    framebuffer_name,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    gl::RENDERBUFFER,
                    ao_depth_renderbuffer_name,
                );

                gl.named_framebuffer_draw_buffers(framebuffer_name, &[gl::COLOR_ATTACHMENT0.into()]);

                assert_eq!(
                    gl.check_named_framebuffer_status(framebuffer_name, gl::FRAMEBUFFER),
                    gl::FRAMEBUFFER_COMPLETE.into()
                );

                framebuffer_name
            };

            let ao_x_framebuffer_name = gl.create_framebuffer().into();

            gl.named_framebuffer_texture(ao_x_framebuffer_name, gl::COLOR_ATTACHMENT0, ao_x_texture.name(), 0);

            gl.named_framebuffer_renderbuffer(
                ao_x_framebuffer_name,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                ao_depth_renderbuffer_name,
            );

            gl.named_framebuffer_draw_buffers(ao_x_framebuffer_name, &[gl::COLOR_ATTACHMENT0.into()]);

            assert_eq!(
                gl.check_named_framebuffer_status(ao_x_framebuffer_name, gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE.into()
            );

            let post_framebuffer_name = {
                let framebuffer_name = gl.create_framebuffer();

                gl.named_framebuffer_texture(framebuffer_name, gl::COLOR_ATTACHMENT0, post_color_texture.name(), 0);

                gl.named_framebuffer_texture(
                    framebuffer_name,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    post_depth_texture.name(),
                    0,
                );

                assert_eq!(
                    gl.check_named_framebuffer_status(framebuffer_name, gl::FRAMEBUFFER),
                    gl::FRAMEBUFFER_COMPLETE.into()
                );

                framebuffer_name
            };

            // Uniform block buffers,

            let lighting_buffer_name = gl.create_buffer();

            ViewDependentResources {
                width,
                height,
                main_framebuffer_name,
                main_color_texture,
                main_depth_texture,
                main_nor_in_cam_texture,
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
            self.main_color_texture.update(gl, texture_update);
            self.main_depth_texture.update(gl, texture_update);
            self.main_nor_in_cam_texture.update(gl, texture_update);
            self.ao_texture.update(gl, texture_update);
            self.ao_x_texture.update(gl, texture_update);
            self.post_color_texture.update(gl, texture_update);
            self.post_depth_texture.update(gl, texture_update);
        }
    }

    pub fn drop(self, gl: &gl::Gl) {
        unsafe {
            gl.delete_framebuffer(self.main_framebuffer_name);
            gl.delete_framebuffer(self.ao_framebuffer_name);
            gl.delete_renderbuffer(self.ao_depth_renderbuffer_name);
            self.main_color_texture.drop(gl);
            self.main_depth_texture.drop(gl);
            self.main_nor_in_cam_texture.drop(gl);
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
    env_logger::init();

    let current_dir = std::env::current_dir().unwrap();
    let resource_dir: PathBuf = [current_dir.as_ref(), Path::new("resources")].into_iter().collect();
    let configuration_path = resource_dir.join(configuration::FILE_PATH);

    let (tx_fs, rx_fs) = mpsc::channel();

    let mut watcher = notify::watcher(tx_fs, time::Duration::from_millis(100)).unwrap();

    notify::Watcher::watch(&mut watcher, &resource_dir, notify::RecursiveMode::Recursive).unwrap();

    let mut configuration: configuration::Root = read_configuration(&configuration_path);

    let mut world = {
        let default_camera_transform = camera::CameraTransform {
            position: Vector3::new(0.0, 1.0, 1.5),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            fovy: Deg(90.0).into(),
        };

        let mut cameras = CameraMap::new(|key| camera::Camera {
            properties: match key {
                CameraKey::Main => configuration.main_camera,
                CameraKey::Debug => configuration.debug_camera,
            }
            .into(),
            transform: default_camera_transform,
        });

        // Load state.
        {
            use std::fs;
            use std::io;
            use std::io::Read;
            match fs::File::open("state.bin") {
                Ok(file) => {
                    let mut file = io::BufReader::new(file);
                    for key in CameraKey::iter() {
                        file.read_exact(cameras[key].value_as_bytes_mut())
                            .unwrap_or_else(|_| eprintln!("Failed to read state file."));
                    }
                }
                Err(_) => {
                    // Whatever.
                }
            }
        }

        let mut global = ic::Global::new();

        World {
            resource_dir,
            tick: 0,
            clear_color: [0.0, 0.0, 0.0],
            window_mode: WindowMode::Main,
            render_technique: ic::Leaf::clean(&mut global, RenderTechnique::Clustered),
            render_technique_regex: RenderTechnique::regex(),
            attenuation_mode: ic::Leaf::clean(&mut global, AttenuationMode::Interpolated),
            attenuation_mode_regex: AttenuationMode::regex(),
            gl_log_regex: RegexBuilder::new(r"^\d+").multi_line(true).build().unwrap(),
            sources: vec![],
            target_camera_key: CameraKey::Main,
            transition_camera: camera::TransitionCamera {
                start_camera: cameras.main,
                current_camera: cameras.main,
                progress: 0.0,
            },
            cameras: cameras.map(|camera| camera::SmoothCamera {
                properties: camera.properties,
                current_transform: camera.transform,
                target_transform: camera.transform,
                smooth_enabled: true,
                current_smoothness: configuration.camera.maximum_smoothness,
                maximum_smoothness: configuration.camera.maximum_smoothness,
            }),
            sun_pos: Vector3::new(0.0, 0.0, 0.0),
            sun_rot: Deg(85.2).into(),
            keyboard_model: keyboard_model::KeyboardModel::new(),
            global,
        }
    };

    let mut keyboard_state = KeyboardState::default();

    let mut events_loop = glutin::EventsLoop::new();

    let gl_window = glutin::GlWindow::new(
        glutin::WindowBuilder::new()
            .with_title("VR Lab - Loading...")
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
            .with_vsync(configuration.window.vsync)
            // .with_multisampling(16)
            .with_pixel_format(configuration.window.rgb_bits, configuration.window.alpha_bits)
            .with_srgb(configuration.window.srgb)
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

        if configuration.global.framebuffer_srgb {
            gl.enable(gl::FRAMEBUFFER_SRGB);
        } else {
            gl.disable(gl::FRAMEBUFFER_SRGB);
        }
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

    let mut add_source = |path| {
        let index = world.sources.len();
        world.sources.push(ShaderSource {
            path: world.resource_dir.join(path),
            modified: ic::Modified::clean(&world.global),
        });
        index
    };

    let mut shadow_renderer = shadow_renderer::Renderer::new(
        &gl,
        vec![add_source("shadow_renderer.vert")],
        vec![add_source("shadow_renderer.frag")],
    );

    let resources = resources::Resources::new(&gl, &world.resource_dir, &configuration);

    let material_resources = rendering::MaterialResources::new(&gl);
    let material_datas: Vec<rendering::MaterialData> = resources
        .materials
        .iter()
        .map(|mat| rendering::MaterialData {
            shininess: mat.shininess,
        })
        .collect();

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

            ArrayVec::from(EYE_KEYS)
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

            for event in rx_fs.try_iter() {
                match event {
                    notify::DebouncedEvent::NoticeWrite(path) => {
                        info!("Noticed write to file {:?}", path.strip_prefix(&world.resource_dir).unwrap().display());
                        for source in world.sources.iter_mut() {
                            if source.path == path {
                                world.global.mark(&mut source.modified);
                            }
                        }

                        if &path == &configuration_path {
                            configuration_update = true;
                        }
                    }
                    _ => {
                        // Don't care.
                    }
                }
            }

            timing_transition!(timings, accumulate_file_updates, execute_file_updates);

            if configuration_update {
                // Read from file.
                configuration = read_configuration(&configuration_path);

                // Apply updates.
                for key in CameraKey::iter() {
                    world.cameras[key].properties = match key {
                        CameraKey::Main => configuration.main_camera,
                        CameraKey::Debug => configuration.debug_camera,
                    }
                    .into();
                    world.cameras[key].maximum_smoothness = configuration.camera.maximum_smoothness;
                }

                unsafe {
                    if configuration.global.framebuffer_srgb {
                        gl.enable(gl::FRAMEBUFFER_SRGB);
                    } else {
                        gl.disable(gl::FRAMEBUFFER_SRGB);
                    }
                }
            }
        }

        timing_transition!(timings, execute_file_updates, wait_for_pose);

        // NOTE: OpenVR will block upon querying the pose for as long as
        // possible but no longer than it takes to submit the new frame. This is
        // done to render the most up-to-date application state as possible.
        struct Mono0 {}

        pub struct Eyes0 {
            tangents: [f32; 4],
            // Input
            pos_from_cam_to_hmd: Matrix4<f64>,
            // Derived
            pos_from_cam_to_bdy: Matrix4<f64>,
        }

        struct Stereo0<'a> {
            vr_context: &'a vr::Context,
            // Input
            pos_from_hmd_to_bdy: Matrix4<f64>,
            eyes: EyeMap<Eyes0>,
        }

        let render_data = match &vr_context {
            Some(vr_context) => {
                let mut poses: [vr::sys::TrackedDevicePose_t; vr::sys::k_unMaxTrackedDeviceCount as usize] =
                    unsafe { mem::zeroed() };
                vr_context.compositor().wait_get_poses(&mut poses[..], None).unwrap();
                let hmd_pose = poses[vr::sys::k_unTrackedDeviceIndex_Hmd as usize];
                assert!(hmd_pose.bPoseIsValid, "Received invalid pose from VR.");
                let pos_from_hmd_to_bdy = Matrix4::from_hmd(hmd_pose.mDeviceToAbsoluteTracking.m).cast().unwrap();

                MonoStereoBox::Stereo(Stereo0 {
                    vr_context,
                    pos_from_hmd_to_bdy,
                    eyes: EyeMap::new(|eye_key| {
                        let eye = Eye::from(eye_key);
                        let pos_from_cam_to_hmd = Matrix4::from_hmd(vr_context.system().get_eye_to_head_transform(eye))
                            .cast()
                            .unwrap();
                        Eyes0 {
                            tangents: vr_context.system().get_projection_raw(eye),
                            pos_from_cam_to_hmd: pos_from_cam_to_hmd,
                            pos_from_cam_to_bdy: pos_from_hmd_to_bdy * pos_from_cam_to_hmd,
                        }
                    }),
                })
            }
            None => MonoStereoBox::Mono(Mono0 {}),
        };

        timing_transition!(timings, wait_for_pose, accumulate_window_updates);

        let mut mouse_dx = 0.0;
        let mut mouse_dy = 0.0;
        let mut mouse_dscroll = 0.0;
        let mut should_resize = false;
        let mut should_export_state = false;
        let mut new_target_camera_key = world.target_camera_key;
        let mut new_window_mode = world.window_mode;
        let mut new_attenuation_mode = world.attenuation_mode.value;

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

                                if keyboard_input.state.is_pressed() && focus {
                                    use glutin::VirtualKeyCode;
                                    match vk {
                                        VirtualKeyCode::Tab => {
                                            // Don't trigger when we ALT TAB.
                                            if keyboard_state.lalt.is_released() {
                                                new_target_camera_key.wrapping_next_assign();
                                            }
                                        }
                                        VirtualKeyCode::Key1 => {
                                            new_attenuation_mode.wrapping_next_assign();
                                        }
                                        VirtualKeyCode::Key2 => {
                                            new_window_mode.wrapping_next_assign();
                                        }
                                        VirtualKeyCode::C => {
                                            world.target_camera_mut().toggle_smoothness();
                                        }
                                        VirtualKeyCode::F5 => {
                                            should_export_state = true;
                                        }
                                        VirtualKeyCode::Escape => {
                                            running = false;
                                        }
                                        _ => (),
                                    }
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

        world.window_mode = new_window_mode;
        world.attenuation_mode.replace(&mut world.global, new_attenuation_mode);

        for key in CameraKey::iter() {
            let is_target = world.target_camera_key == key;
            let delta = camera::CameraDelta {
                time: delta_time,
                position: if is_target && focus {
                    Vector3::new(
                        keyboard_state.d.to_f32() - keyboard_state.a.to_f32(),
                        keyboard_state.q.to_f32() - keyboard_state.z.to_f32(),
                        keyboard_state.s.to_f32() - keyboard_state.w.to_f32(),
                    ) * (1.0 + keyboard_state.lshift.to_f32() * 3.0)
                } else {
                    Vector3::zero()
                },
                yaw: Rad(if is_target { -mouse_dx as f32 } else { 0.0 }),
                pitch: Rad(if is_target { -mouse_dy as f32 } else { 0.0 }),
                fovy: Rad(if is_target { mouse_dscroll as f32 } else { 0.0 }),
            };
            world.cameras[key].update(&delta);
        }

        if new_target_camera_key != world.target_camera_key {
            world.target_camera_key = new_target_camera_key;
            world.transition_camera.start_transition();
        }

        world.transition_camera.update(camera::TransitionCameraUpdate {
            delta_time,
            end_camera: &world.target_camera().current_to_camera(),
        });

        if vr_context.is_some() {
            // Pitch makes me dizzy.
            world.transition_camera.current_camera.transform.pitch = Rad(0.0);
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

        let sun_frustrum = Frustrum::<f64> {
            x0: -25.0,
            x1: 25.0,
            y0: -25.0,
            y1: 25.0,
            z0: 30.0,
            z1: -30.0,
        };

        let (sun_frustrum_vertices, sun_frustrum_indices) = sun_frustrum.cast::<f32>().unwrap().line_mesh();

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

                time: world.tick as f64 / DESIRED_UPS,
            }
        };

        fn mono_frustrum(camera: camera::Camera, viewport: Viewport<i32>) -> Frustrum<f64> {
            let z0 = camera.properties.z0 as f64;
            let z1 = camera.properties.z1 as f64;
            let dy = -z0 * Rad::tan(Rad(Rad::from(camera.transform.fovy).0 as f64) / 2.0);
            let dx = dy * viewport.dimensions.x as f64 / viewport.dimensions.y as f64;
            Frustrum::<f64> {
                x0: -dx,
                x1: dx,
                y0: -dy,
                y1: dy,
                z0,
                z1,
            }
        }

        fn stereo_frustrum(camera_properties: camera::CameraProperties, tangents: [f32; 4]) -> Frustrum<f64> {
            let [l, r, b, t] = tangents;
            let z0 = camera_properties.z0 as f64;
            let z1 = camera_properties.z1 as f64;
            Frustrum::<f64> {
                x0: -z0 * l as f64,
                x1: -z0 * r as f64,
                y0: -z0 * b as f64,
                y1: -z0 * t as f64,
                z0,
                z1,
            }
        }

        pub struct ViewDataExtN<VD> {
            pub viewport: Viewport<i32>,

            pub view_data: VD,

            // CLS
            pub pos_from_wld_to_hmd: Matrix4<f64>,
            pub pos_from_hmd_to_wld: Matrix4<f64>,
            pub pos_from_clp_to_hmd: Matrix4<f64>,
        }

        pub type ViewDataExt0 = ViewDataExtN<rendering::ViewData0>;
        pub type ViewDataExt = ViewDataExtN<rendering::ViewData>;

        impl ViewDataExt0 {
            pub fn mono(viewport: Viewport<i32>, camera: camera::Camera) -> Self {
                let frustrum = mono_frustrum(camera, viewport);

                let pos_from_cam_to_clp = frustrum.perspective(DEPTH_RANGE);
                let pos_from_clp_to_cam = pos_from_cam_to_clp.invert().unwrap();

                let pos_from_wld_to_cam = camera.transform.pos_from_parent().cast::<f64>().unwrap();
                let pos_from_cam_to_wld = camera.transform.pos_to_parent().cast::<f64>().unwrap();

                ViewDataExt0 {
                    viewport,

                    view_data: rendering::ViewData0 {
                        pos_from_wld_to_cam,
                        pos_from_cam_to_wld,

                        pos_from_cam_to_clp,
                        pos_from_clp_to_cam,
                    },

                    pos_from_wld_to_hmd: pos_from_wld_to_cam,
                    pos_from_hmd_to_wld: pos_from_cam_to_wld,
                    pos_from_clp_to_hmd: pos_from_clp_to_cam,
                }
            }

            pub fn stereo(
                viewport: Viewport<i32>,
                pos_from_hmd_to_bdy: Matrix4<f64>,
                eye: Eyes0,
                camera: camera::Camera,
            ) -> Self {
                let Eyes0 {
                    tangents,
                    pos_from_cam_to_hmd,
                    pos_from_cam_to_bdy,
                } = eye;
                let frustrum = stereo_frustrum(camera.properties, tangents);

                let pos_from_cam_to_clp = frustrum.perspective(DEPTH_RANGE);
                let pos_from_clp_to_cam = pos_from_cam_to_clp.invert().unwrap();

                let pos_from_bdy_to_wld = camera.transform.pos_to_parent().cast().unwrap();
                let pos_from_cam_to_wld = pos_from_bdy_to_wld * pos_from_cam_to_bdy;

                let pos_from_hmd_to_wld = pos_from_hmd_to_bdy * pos_from_bdy_to_wld;

                ViewDataExt0 {
                    viewport,

                    view_data: rendering::ViewData0 {
                        pos_from_wld_to_cam: pos_from_cam_to_wld.invert().unwrap(),
                        pos_from_cam_to_wld,

                        pos_from_cam_to_clp,
                        pos_from_clp_to_cam,
                    },

                    pos_from_wld_to_hmd: pos_from_hmd_to_wld.invert().unwrap(),
                    pos_from_hmd_to_wld,

                    pos_from_clp_to_hmd: pos_from_clp_to_cam * pos_from_cam_to_hmd,
                }
            }

            pub fn into_view_data_ext(self, global_data: &rendering::GlobalData) -> ViewDataExt {
                let ViewDataExt0 {
                    viewport,

                    view_data,

                    pos_from_wld_to_hmd,
                    pos_from_hmd_to_wld,

                    pos_from_clp_to_hmd,
                } = self;

                ViewDataExt {
                    viewport,

                    view_data: view_data.into_view_data(global_data),

                    pos_from_wld_to_hmd,
                    pos_from_hmd_to_wld,

                    pos_from_clp_to_hmd,
                }
            }
        }

        struct RenderData2 {
            cls_buffer: rendering::CLSBuffer,
            cls_view_data_ext: ViewDataExt,
            full_viewport: Viewport<i32>,
            mode_data: WindowModeBox<ViewDataExt, ViewDataExt, CameraMap<ViewDataExt>>,
        }

        let RenderData2 {
            cls_buffer,
            cls_view_data_ext,
            full_viewport,
            mode_data,
        } = {
            let glutin::dpi::PhysicalSize { width, height } = win_size.to_physical(win_dpi);
            let (width, height) = (width as i32, height as i32);

            let full_viewport = Viewport::from_dimensions(Vector2::new(width, height));

            match &render_data {
                MonoStereoBox::Mono(_) => {
                    let viewport_map = CameraMap {
                        main: Viewport::from_coordinates(
                            Point2::origin(),
                            if width > height {
                                Point2::new(width / 2, height)
                            } else {
                                Point2::new(width, height / 2)
                            },
                        ),
                        debug: Viewport::from_coordinates(
                            if width > height {
                                Point2::new(width / 2, 0)
                            } else {
                                Point2::new(0, height / 2)
                            },
                            Point2::new(width, height),
                        ),
                    };

                    let mode_data = match world.window_mode {
                        WindowMode::Main => WindowModeBox::Main(
                            ViewDataExt0::mono(full_viewport, world.transition_camera.current_camera)
                                .into_view_data_ext(&global_data),
                        ),
                        WindowMode::Debug => WindowModeBox::Debug(
                            ViewDataExt0::mono(full_viewport, world.transition_camera.current_camera)
                                .into_view_data_ext(&global_data),
                        ),
                        WindowMode::Split => {
                            WindowModeBox::Split(viewport_map.zip(world.cameras.as_ref(), |viewport, camera| {
                                ViewDataExt0::mono(viewport, camera.current_to_camera())
                                    .into_view_data_ext(&global_data)
                            }))
                        }
                    };

                    let cls_view_data_ext = match world.window_mode {
                        WindowMode::Main | WindowMode::Debug => {
                            ViewDataExt0::mono(full_viewport, world.cameras.main.current_to_camera())
                        }
                        WindowMode::Split => {
                            ViewDataExt0::mono(viewport_map.main, world.cameras.main.current_to_camera())
                        }
                    }
                    .into_view_data_ext(&global_data);

                    RenderData2 {
                        cls_buffer: {
                            cls::compute_light_assignment(
                                &[cls_view_data_ext.pos_from_clp_to_hmd],
                                cls_view_data_ext.pos_from_wld_to_hmd,
                                cls_view_data_ext.pos_from_hmd_to_wld,
                                &resources.point_lights[..],
                                &configuration.clustered_light_shading,
                            )
                        },
                        full_viewport,
                        cls_view_data_ext,
                        mode_data,
                    }
                }
                MonoStereoBox::Stereo(stereo) => {
                    unimplemented!()
                    // let view_cams = match world.window_mode {
                    //     WindowMode::Main => {
                    //         // Blit eyes to main
                    //         WindowModeBox::Main(())
                    //     },
                    //     WindowMode::Debug => WindowModeBox::Debug(ViewCam {
                    //         viewport: full_viewport,
                    //         camera: world.transition_camera.current_camera,
                    //     }),
                    //     WindowMode::Split => {
                    //         WindowModeBox::Split(split_viewports.zip(world.cameras.as_ref(), |viewport, camera| ViewCam {
                    //             viewport,
                    //             camera: camera.current_to_camera(),
                    //         }))
                    //     }
                    // };

                    // let eyes = stereo.eyes.as_ref().map(|eye| {
                    //     let frustrum = stereo_frustrum(world.cameras.main.properties, eye.tangents);

                    //     let pos_from_eye_to_clp = frustrum.perspective(DEPTH_RANGE);
                    //     let pos_from_clp_to_eye = pos_from_eye_to_clp.invert().unwrap();

                    //     let pos_from_clp_to_hmd = eye.pos_from_eye_to_hmd * pos_from_clp_to_eye;

                    //     pos_from_clp_to_hmd
                    // });

                    // let matrices = [eyes.left, eyes.right];

                    // RenderData2 {
                    //     cluster_bounding_box: cls::compute_bounding_box(matrices.iter()),
                    //     mono_stereo: MonoStereoBox::Stereo(Stereo2 {}),
                    // }
                }
            }
        };

        if should_export_state {
            use std::io::Write;
            let mut f = std::fs::File::create("cls.bin").unwrap();
            f.write_all(cls_buffer.header.value_as_bytes()).unwrap();
            f.write_all(cls_buffer.body.vec_as_bytes()).unwrap();
        }

        timing_transition!(timings, prepare_render_data, render);

        view_ind_res.global_resources.write(&gl, &global_data);
        view_ind_res.cls_resources.write(&gl, &cls_buffer);

        let shadow_viewport = Viewport::from_dimensions(Vector2::new(SHADOW_W, SHADOW_H));

        // View independent.
        shadow_renderer.render(
            &gl,
            &shadow_renderer::Parameters {
                viewport: shadow_viewport,
                framebuffer: view_ind_res.shadow_framebuffer_name.into(),
            },
            &mut world,
            &resources,
        );

        // View independent.
        // vsm_filter.render(
        //     &gl,
        //     &vsm_filter::Parameters {
        //         viewport: shadow_viewport,
        //         framebuffer_x: view_ind_res.shadow_2_framebuffer_name.into(),
        //         framebuffer_xy: view_ind_res.shadow_framebuffer_name.into(),
        //         color: view_ind_res.shadow_texture.name(),
        //         color_x: view_ind_res.shadow_2_texture.name(),
        //     },
        //     &resources,
        // );

        let compute_and_upload_light_positions = |view_dep_res: &ViewDependentResources, pos_from_wld_to_cam| unsafe {
            let mut point_lights: [light::PointLightBufferEntry; rendering::POINT_LIGHT_CAPACITY as usize] =
                std::mem::uninitialized();
            for i in 0..rendering::POINT_LIGHT_CAPACITY as usize {
                point_lights[i] =
                    light::PointLightBufferEntry::from_point_light(resources.point_lights[i], pos_from_wld_to_cam);
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
        };

        fn render_start(gl: &gl::Gl, framebuffer: gl::FramebufferName, world: &World) {
            unsafe {
                gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer);

                gl.clear_color(world.clear_color[0], world.clear_color[1], world.clear_color[2], 1.0);

                // Reverse-Z projection.
                gl.clear_depth(0.0);
                gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);
            }
        };

        let render_main = |viewport: Viewport<i32>,
                           view_ind_res: &ViewIndependentResources,
                           view_dep_res: &ViewDependentResources| {
            // basic_renderer.render(
            //     &gl,
            //     &basic_renderer::Parameters {
            //         viewport,
            //         framebuffer: view_dep_res.main_framebuffer_name.into(),
            //         material_resources,
            //         shadow_texture_name: view_ind_res.shadow_texture.name(),
            //         shadow_texture_dimensions: [SHADOW_W as f32, SHADOW_H as f32],
            //     },
            //     &world,
            //     &resources,
            // );

            // line_renderer.render(
            //     &gl,
            //     &line_renderer::Parameters {
            //         viewport,
            //         framebuffer: view_dep_res.main_framebuffer_name.into(),
            //         vertices: &sun_frustrum_vertices[..],
            //         indices: &sun_frustrum_indices[..],
            //         pos_from_obj_to_wld: &global_data.light_pos_from_cam_to_wld,
            //     },
            // );
        };

        let render_end = |viewport: Viewport<i32>, view_dep_res: &ViewDependentResources| {
            // ao_renderer.render(
            //     &gl,
            //     &ao_renderer::Parameters {
            //         viewport,
            //         framebuffer: view_dep_res.ao_framebuffer_name.into(),
            //         color_texture_name: view_dep_res.main_color_texture.name(),
            //         depth_texture_name: view_dep_res.main_depth_texture.name(),
            //         nor_in_cam_texture_name: view_dep_res.main_nor_in_cam_texture.name(),
            //         random_unit_sphere_surface_texture_name: random_unit_sphere_surface_texture.name(),
            //     },
            //     &world,
            //     &resources,
            // );

            // ao_filter.render(
            //     &gl,
            //     &ao_filter::Parameters {
            //         viewport,
            //         framebuffer_x: view_dep_res.ao_x_framebuffer_name.into(),
            //         framebuffer_xy: view_dep_res.ao_framebuffer_name.into(),
            //         color: view_dep_res.ao_texture.name(),
            //         color_x: view_dep_res.ao_x_texture.name(),
            //         depth: view_dep_res.main_depth_texture.name(),
            //     },
            //     &resources,
            // );

            // post_renderer.render(
            //     &gl,
            //     &post_renderer::Parameters {
            //         viewport,
            //         framebuffer: gl::FramebufferName::Default,
            //         color_texture_name: view_dep_res.main_color_texture.name(),
            //         depth_texture_name: view_dep_res.main_depth_texture.name(),
            //         nor_in_cam_texture_name: view_dep_res.main_nor_in_cam_texture.name(),
            //         ao_texture_name: view_dep_res.ao_texture.name(),
            //     },
            //     &world,
            //     &resources,
            // );
        };

        let render_debug =
            |viewport: Viewport<i32>, framebuffer: gl::FramebufferName, cls_view_data_ext: &ViewDataExt| {
                // cluster_renderer.render(
                //     &gl,
                //     &cluster_renderer::Parameters {
                //         viewport,
                //         framebuffer,
                //         cls_buffer: &cls_buffer,
                //         configuration: &configuration.clustered_light_shading,
                //     },
                //     &world,
                //     &resources,
                // );

                // let corners_in_clp = Frustrum::corners_in_clp(DEPTH_RANGE);
                // let vertices: Vec<[f32; 3]> = corners_in_clp
                //     .iter()
                //     .map(|point| point.cast().unwrap().into())
                //     .collect();

                // line_renderer.render(
                //     &gl,
                //     &line_renderer::Parameters {
                //         viewport,
                //         framebuffer,
                //         vertices: &vertices[..],
                //         indices: &sun_frustrum_indices[..],
                //         pos_from_obj_to_wld: &cls_view_data_ext.view_data.pos_from_clp_to_wld.cast().unwrap(),
                //     },
                // );
            };

        if let Some(vr_context) = &vr_context {
            // FIXME
            unimplemented!()
            // let viewports = {
            //     let w = physical_size.width as i32;
            //     let h = physical_size.height as i32;
            //     [(0, w / 2, 0, h), (w / 2, w, 0, h)]
            // };

            // for (view_index, &eye) in EYE_KEYS.iter().enumerate() {
            //     let view_dep_res = &view_dep_res[view_index];

            //     // Render both eyes to the default framebuffer.
            //     let viewport = viewports[view_index];
            //     overlay_renderer.render(
            //         &gl,
            //         &overlay_renderer::Parameters {
            //             framebuffer: gl::FramebufferName::Default,
            //             x0: viewport.0,
            //             x1: viewport.1,
            //             y0: viewport.2,
            //             y1: viewport.3,
            //             color_texture_name: view_dep_res.post_color_texture.name(),
            //             default_colors: [0.0, 0.0, 0.0, 0.0],
            //             color_matrix: [
            //                 [1.0, 0.0, 0.0, 0.0],
            //                 [0.0, 1.0, 0.0, 0.0],
            //                 [0.0, 0.0, 1.0, 0.0],
            //                 [0.0, 0.0, 0.0, 1.0],
            //             ],
            //         },
            //         &resources,
            //     );

            //     // NOTE(mickvangelderen): Binding the color attachments causes SIGSEGV!!!
            //     let mut texture_t = gen_texture_t(view_dep_res.post_color_texture.name());
            //     vr_context
            //         .compositor()
            //         .submit(eye, &mut texture_t, None, vr::SubmitFlag::Default)
            //         .unwrap_or_else(|error| {
            //             panic!(
            //                 "failed to submit texture: {:?}",
            //                 vr::CompositorError::from_unchecked(error).unwrap()
            //             );
            //         });
            // }
        }

        // MONOSCOPIC RENDERING USES SINGLE VIEW DEP
        let view_dep_res = &view_dep_res[0];

        render_start(&gl, view_dep_res.main_framebuffer_name.into(), &world);

        let (maybe_main, maybe_debug) = match mode_data {
            WindowModeBox::Main(view_data_ext) => {
                view_resources.write_all(&gl, &[view_data_ext.view_data]);
                (Some((0, view_data_ext)), None)
            }
            WindowModeBox::Debug(view_data_ext) => {
                view_resources.write_all(&gl, &[view_data_ext.view_data]);
                (None, Some((0, view_data_ext)))
            }
            WindowModeBox::Split(view_data_ext_map) => {
                view_resources.write_all_ref(
                    &gl,
                    &[&view_data_ext_map.main.view_data, &view_data_ext_map.debug.view_data],
                );
                (Some((0, view_data_ext_map.main)), Some((1, view_data_ext_map.debug)))
            }
        };

        if let Some((index, main_view_data)) = maybe_main {
            view_resources.bind_index(&gl, index);
            compute_and_upload_light_positions(&view_dep_res, main_view_data.view_data.pos_from_wld_to_cam);
            render_main(main_view_data.viewport, &view_ind_res, &view_dep_res);
        }

        if let Some((index, debug_view_data)) = maybe_debug {
            view_resources.bind_index(&gl, index);
            render_debug(
                debug_view_data.viewport,
                view_dep_res.main_framebuffer_name.into(),
                &cls_view_data_ext,
            );
        }

        render_end(full_viewport, view_dep_res);

        timing_transition!(timings, render, swap_buffers);

        gl_window.swap_buffers().unwrap();

        timings.swap_buffers.end = time::Instant::now();

        if keyboard_state.p.is_pressed() && keyboard_state.lalt.is_pressed() {
            timings.print_deltas();
        }

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
            gl_window.window().set_title(&format!(
                "VR Lab - {:?} - {:02.1} FPS",
                world.target_camera_key,
                fps_average.compute()
            ));
        }
    }

    // Save state.
    {
        use std::fs;
        use std::io;
        use std::io::Write;
        let mut file = io::BufWriter::new(fs::File::create("state.bin").unwrap());
        for key in CameraKey::iter() {
            let camera = world.cameras[key].current_to_camera();
            file.write_all(camera.value_as_bytes()).unwrap();
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
