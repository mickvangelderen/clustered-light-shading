#![allow(non_snake_case)]

// Has to go first.
#[macro_use]
mod macros;

pub(crate) use gl_typed as gl;
pub(crate) use log::*;
pub(crate) use rand::prelude::*;
pub(crate) use regex::{Regex, RegexBuilder};
#[allow(unused_imports)]
pub(crate) use std::convert::{TryFrom, TryInto};
#[allow(unused_imports)]
pub(crate) use std::num::{NonZeroU32, NonZeroU64};
pub(crate) use std::time::Instant;

mod basic_renderer;
pub mod camera;
pub mod cgmath_ext;
pub mod clamp;
mod cls;
mod configuration;
mod convert;
mod depth_renderer;
mod filters;
pub mod frustrum;
pub mod gl_ext;
mod glutin_ext;
mod keyboard;
mod light;
mod line_renderer;
mod main_resources;
mod math;
mod overlay_renderer;
pub mod profiling;
mod rain;
mod rendering;
mod resources;
mod shader_compiler;
mod text_renderer;
mod text_rendering;
mod viewport;
mod window_mode;

use self::cgmath_ext::*;
use self::cls::*;
use self::frustrum::*;
use self::gl_ext::*;
use self::main_resources::*;
use self::math::{CeilToMultiple, DivCeil};
use self::profiling::*;
use self::rendering::*;
use self::resources::Resources;
use self::shader_compiler::{EntryPoint, ShaderCompiler};
use self::text_rendering::{FontContext, TextBox};
use self::viewport::*;
use self::window_mode::*;
use cgmath::*;
use convert::*;
use derive::EnumNext;
use glutin::GlContext;
use glutin_ext::*;
use keyboard::*;
use openvr as vr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time;

const DESIRED_UPS: f64 = 90.0;

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

pub struct CameraParameters {
    pub wld_to_cam: Matrix4<f64>,
    pub cam_to_clp: Matrix4<f64>,
}

pub struct MainParameters {
    pub wld_to_cam: Matrix4<f64>,
    pub cam_to_wld: Matrix4<f64>,

    pub cam_to_clp: Matrix4<f64>,
    pub clp_to_cam: Matrix4<f64>,

    pub cam_pos_in_wld: Point3<f64>,

    pub light_index: usize,
    pub cluster_resources_index: Option<ClusterResourcesIndex>,

    pub dimensions: Vector2<i32>,
    pub display_viewport: Viewport<i32>,
}

const DEPTH_RANGE: (f64, f64) = (1.0, 0.0);

#[derive(Debug)]
pub struct WindowEventAccumulator {
    pub mouse_delta: Vector2<f64>,
    pub scroll_delta: f64,
}

impl std::default::Default for WindowEventAccumulator {
    fn default() -> Self {
        Self {
            mouse_delta: Vector2::zero(),
            scroll_delta: 0.0,
        }
    }
}

pub struct Context {
    pub resource_dir: PathBuf,
    pub configuration_path: PathBuf,
    pub configuration: configuration::Root,
    pub gl: gl::Gl,
    pub vr: Option<vr::Context>,
    pub current: ::incremental::Current,
    pub epoch: Instant,
    pub running: bool,
    pub paused: bool,
    pub focus: bool,
    pub tick: u64,
    pub frame: u64,
    pub keyboard_state: KeyboardState,
    pub win_dpi: f64,
    pub win_size: glutin::dpi::PhysicalSize,
    pub clear_color: [f32; 3],
    pub window_mode: WindowMode,
    pub display_mode: u32,
    pub depth_prepass: bool,
    pub target_camera_key: CameraKey,
    pub transition_camera: camera::TransitionCamera,
    pub cameras: CameraMap<camera::SmoothCamera>,
    pub rain_drops: Vec<rain::Particle>,
    pub shader_compiler: ShaderCompiler,

    // File system events
    pub fs_rx: mpsc::Receiver<notify::DebouncedEvent>,
    pub watcher: notify::RecommendedWatcher,

    // Window.
    pub gl_window: glutin::GlWindow,
    pub window_event_accumulator: WindowEventAccumulator,

    // Text rendering.
    pub sans_serif: FontContext,
    pub monospace: FontContext,
    pub overlay_textbox: TextBox,

    // Renderers
    pub depth_renderer: depth_renderer::Renderer,
    pub line_renderer: line_renderer::Renderer,
    pub basic_renderer: basic_renderer::Renderer,
    pub overlay_renderer: overlay_renderer::Renderer,
    pub cluster_renderer: cluster_renderer::Renderer,
    pub text_renderer: text_renderer::Renderer,
    pub cls_renderer: cls_renderer::Renderer,

    // More opengl resources...
    pub resources: Resources,

    // FPS counter
    pub fps_average: filters::MovingAverageF32,
    pub last_frame_start: time::Instant,

    // Per-frame resources
    pub main_parameters_vec: Vec<MainParameters>,
    pub camera_buffer_pool: BufferPool,
    pub light_resources_vec: Vec<light::LightResources>,
    pub light_params_vec: Vec<light::LightParameters>,
    pub cluster_resources_pool: ClusterResourcesPool,
    pub main_resources_pool: MainResourcesPool,
    pub point_lights: Vec<light::PointLight>,
}

impl Context {
    pub fn new(events_loop: &mut glutin::EventsLoop) -> Self {
        let current_dir = std::env::current_dir().unwrap();
        let resource_dir: PathBuf = [current_dir.as_ref(), Path::new("resources")].into_iter().collect();
        let configuration_path = resource_dir.join(configuration::FILE_PATH);

        let configuration: configuration::Root = configuration::read(&configuration_path);

        let (fs_tx, fs_rx) = mpsc::channel();
        let mut watcher = notify::watcher(fs_tx, time::Duration::from_millis(100)).unwrap();
        notify::Watcher::watch(&mut watcher, &resource_dir, notify::RecursiveMode::Recursive).unwrap();

        let gl_window = glutin::GlWindow::new(
            glutin::WindowBuilder::new()
                .with_title("VR Lab - Loading...")
                .with_dimensions(
                    // Jump through some hoops to ensure a physical size, which is
                    // what I want in case I'm recording at a specific resolution.
                    glutin::dpi::PhysicalSize::new(
                        f64::from(configuration.window.width),
                        f64::from(configuration.window.height),
                    )
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

        unsafe { gl_window.make_current().unwrap() };

        let default_camera_transform = camera::CameraTransform {
            position: Point3::new(0.0, 1.0, 1.5),
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

        let gl = unsafe {
            let gl = gl::Gl::load_with(|s| gl_window.context().get_proc_address(s) as *const _);

            println!("OpenGL version {}.", gl.get_string(gl::VERSION));
            let flags = gl.get_context_flags();
            if flags.contains(gl::ContextFlag::DEBUG) {
                println!("OpenGL debugging enabled.");
                gl.enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            }

            // NOTE: This alignment is hardcoded in rendering.rs.
            assert_eq!(256, gl.get_uniform_buffer_offset_alignment());

            // Reverse-Z.
            gl.clip_control(gl::LOWER_LEFT, gl::ZERO_TO_ONE);
            gl.depth_func(gl::GREATER);

            if configuration.global.framebuffer_srgb {
                gl.enable(gl::FRAMEBUFFER_SRGB);
            } else {
                gl.disable(gl::FRAMEBUFFER_SRGB);
            }

            gl
        };

        let sans_serif = FontContext::new(&gl, resource_dir.join("fonts/OpenSans-Regular.fnt"));
        let monospace = FontContext::new(&gl, resource_dir.join("fonts/RobotoMono-Regular.fnt"));

        let mut current = ::incremental::Current::new();

        let mut shader_compiler = ShaderCompiler::new(
            &current,
            shader_compiler::Variables {
                light_space: LightSpace::Wld,
                render_technique: RenderTechnique::Clustered,
                attenuation_mode: AttenuationMode::Interpolated,
                prefix_sum: configuration.prefix_sum,
                clustered_light_shading: configuration.clustered_light_shading,
            },
        );

        let mut rendering_context = RenderingContext {
            gl: &gl,
            resource_dir: &resource_dir,
            current: &mut current,
            shader_compiler: &mut shader_compiler,
        };

        let depth_renderer = depth_renderer::Renderer::new(&mut rendering_context);
        let line_renderer = line_renderer::Renderer::new(&mut rendering_context);
        let basic_renderer = basic_renderer::Renderer::new(&mut rendering_context);
        let overlay_renderer = overlay_renderer::Renderer::new(&mut rendering_context);
        let cluster_renderer = cluster_renderer::Renderer::new(&mut rendering_context);
        let text_renderer = text_renderer::Renderer::new(&mut rendering_context);
        let cls_renderer = cls_renderer::Renderer::new(&mut rendering_context);

        drop(rendering_context);

        let resources = resources::Resources::new(&gl, &resource_dir, &configuration);

        let vr = vr::Context::new(vr::ApplicationType::Scene)
            .map_err(|error| {
                eprintln!("Failed to acquire context: {:?}", error);
            })
            .ok();

        let win_dpi = gl_window.get_hidpi_factor();
        let win_size = gl_window.get_inner_size().unwrap().to_physical(win_dpi);

        Context {
            resource_dir,
            configuration_path,
            configuration,
            gl,
            vr,
            current,
            epoch: Instant::now(),
            running: true,
            paused: false,
            focus: false,
            tick: 0,
            frame: 0,
            keyboard_state: Default::default(),
            win_dpi,
            win_size,
            clear_color: [0.0; 3],
            window_mode: WindowMode::Main,
            display_mode: 1,
            depth_prepass: true,
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
            rain_drops: Vec::new(),
            shader_compiler,

            // File system events
            fs_rx,
            watcher,

            // Window.
            gl_window,
            window_event_accumulator: Default::default(),

            // Text rendering.
            sans_serif,
            monospace,
            overlay_textbox: TextBox::new(13, 10, win_size.width as i32 - 26, win_size.height as i32 - 20),

            // Renderers
            depth_renderer,
            line_renderer,
            basic_renderer,
            overlay_renderer,
            cluster_renderer,
            text_renderer,
            cls_renderer,

            // More opengl resources...
            resources,

            // FPS counter
            fps_average: filters::MovingAverageF32::new(0.0),
            last_frame_start: Instant::now(),

            // Per-frame resources
            camera_buffer_pool: BufferPool::new(),
            light_resources_vec: Vec::new(),
            light_params_vec: Vec::new(),
            main_parameters_vec: Vec::new(),
            cluster_resources_pool: ClusterResourcesPool::new(),
            main_resources_pool: MainResourcesPool::new(),
            point_lights: Vec::new(),
        }
    }

    fn target_camera(&self) -> &camera::SmoothCamera {
        &self.cameras[self.target_camera_key]
    }

    fn target_camera_mut(&mut self) -> &mut camera::SmoothCamera {
        &mut self.cameras[self.target_camera_key]
    }

    pub fn process_events(&mut self, events_loop: &mut glutin::EventsLoop) {
        self.process_file_events();
        self.process_window_events(events_loop);
        self.process_vr_events();
    }

    fn process_file_events(&mut self) {
        let mut configuration_update = false;

        for event in self.fs_rx.try_iter() {
            match event {
                notify::DebouncedEvent::NoticeWrite(path) => {
                    info!(
                        "Noticed write to file {:?}",
                        path.strip_prefix(&self.resource_dir).unwrap().display()
                    );

                    if let Some(source_index) = self.shader_compiler.memory.source_index(&path) {
                        self.shader_compiler
                            .source_mut(source_index)
                            .last_modified
                            .modify(&mut self.current);
                    }

                    if &path == &self.configuration_path {
                        configuration_update = true;
                    }
                }
                _ => {
                    // Don't care.
                }
            }
        }

        if configuration_update {
            // Read from file.
            self.configuration = configuration::read(&self.configuration_path);

            // Apply updates.
            self.cameras.main.properties = self.configuration.main_camera.into();
            self.cameras.debug.properties = self.configuration.debug_camera.into();
            for key in CameraKey::iter() {
                self.cameras[key].maximum_smoothness = self.configuration.camera.maximum_smoothness;
            }

            self.shader_compiler
                .replace_prefix_sum(&mut self.current, self.configuration.prefix_sum);
            self.shader_compiler
                .replace_clustered_light_shading(&mut self.current, self.configuration.clustered_light_shading);

            unsafe {
                let gl = &self.gl;
                if self.configuration.global.framebuffer_srgb {
                    gl.enable(gl::FRAMEBUFFER_SRGB);
                } else {
                    gl.disable(gl::FRAMEBUFFER_SRGB);
                }
            }
        }
    }

    fn process_window_events(&mut self, events_loop: &mut glutin::EventsLoop) {
        let mut new_target_camera_key = self.target_camera_key;
        let new_window_mode = self.window_mode;
        let mut new_light_space = self.shader_compiler.light_space();
        let mut new_attenuation_mode = self.shader_compiler.attenuation_mode();
        let mut new_render_technique = self.shader_compiler.render_technique();
        let mut reset_debug_camera = false;

        events_loop.poll_events(|event| {
            use glutin::Event;
            match event {
                Event::WindowEvent { event, .. } => {
                    use glutin::WindowEvent;
                    match event {
                        WindowEvent::CloseRequested => self.running = false,
                        WindowEvent::HiDpiFactorChanged(val) => {
                            let win_size = self.win_size.to_logical(self.win_dpi);
                            self.win_dpi = val;
                            self.win_size = win_size.to_physical(self.win_dpi);
                        }
                        WindowEvent::Focused(val) => self.focus = val,
                        WindowEvent::Resized(val) => {
                            self.win_size = val.to_physical(self.win_dpi);
                        }
                        _ => (),
                    }
                }
                Event::DeviceEvent { event, .. } => {
                    use glutin::DeviceEvent;
                    match event {
                        DeviceEvent::Key(keyboard_input) => {
                            self.keyboard_state.update(keyboard_input);

                            if let Some(vk) = keyboard_input.virtual_keycode {
                                if keyboard_input.state.is_pressed() && self.focus {
                                    use glutin::VirtualKeyCode;
                                    match vk {
                                        VirtualKeyCode::Tab => {
                                            // Don't trigger when we ALT TAB.
                                            if self.keyboard_state.lalt.is_released() {
                                                new_target_camera_key.wrapping_next_assign();
                                            }
                                        }
                                        VirtualKeyCode::Key1 => {
                                            new_attenuation_mode.wrapping_next_assign();
                                        }
                                        VirtualKeyCode::Key2 => {
                                            // new_window_mode.wrapping_next_assign();
                                            self.display_mode += 1;
                                            if self.display_mode >= 4 {
                                                self.display_mode = 1;
                                            }
                                        }
                                        VirtualKeyCode::Key3 => {
                                            new_render_technique.wrapping_next_assign();
                                        }
                                        VirtualKeyCode::Key4 => {
                                            new_light_space.wrapping_next_assign();
                                        }
                                        VirtualKeyCode::Key5 => {
                                            self.depth_prepass = !self.depth_prepass;
                                        }
                                        VirtualKeyCode::R => {
                                            reset_debug_camera = true;
                                        }
                                        VirtualKeyCode::Backslash => {
                                            self.configuration.virtual_stereo.enabled =
                                                !self.configuration.virtual_stereo.enabled;
                                        }
                                        VirtualKeyCode::C => {
                                            self.target_camera_mut().toggle_smoothness();
                                        }
                                        VirtualKeyCode::Escape => {
                                            self.running = false;
                                        }
                                        VirtualKeyCode::Space => {
                                            self.paused = !self.paused;
                                        }
                                        _ => (),
                                    }
                                }
                            }
                        }
                        DeviceEvent::Motion { axis, value } => {
                            if self.focus {
                                match axis {
                                    0 => self.window_event_accumulator.mouse_delta.x += value,
                                    1 => self.window_event_accumulator.mouse_delta.y += value,
                                    3 => self.window_event_accumulator.scroll_delta += value,
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

        self.window_mode = new_window_mode;

        self.shader_compiler
            .replace_light_space(&mut self.current, new_light_space);
        self.shader_compiler
            .replace_attenuation_mode(&mut self.current, new_attenuation_mode);
        self.shader_compiler
            .replace_render_technique(&mut self.current, new_render_technique);

        if new_target_camera_key != self.target_camera_key {
            self.target_camera_key = new_target_camera_key;
            self.transition_camera.start_transition();
        }

        if reset_debug_camera {
            self.cameras.debug.target_transform = self.cameras.main.target_transform;
            self.transition_camera.start_transition();
        }
    }

    fn process_vr_events(&mut self) {
        if let Some(ref vr) = self.vr {
            while let Some(_event) = vr.system().poll_next_event() {
                // TODO: Handle vr events.
            }
        }
    }

    pub fn simulate(&mut self) {
        // TODO: Refactor names to be consistent with accumulator.
        let mouse_dx = std::mem::replace(&mut self.window_event_accumulator.mouse_delta.x, 0.0);
        let mouse_dy = std::mem::replace(&mut self.window_event_accumulator.mouse_delta.y, 0.0);
        let mouse_dscroll = std::mem::replace(&mut self.window_event_accumulator.scroll_delta, 0.0);

        let delta_time = 1.0 / DESIRED_UPS as f32;

        for key in CameraKey::iter() {
            let is_target = self.target_camera_key == key;
            let delta = camera::CameraDelta {
                time: delta_time,
                position: if is_target && self.focus {
                    Vector3::new(
                        self.keyboard_state.d.to_f32() - self.keyboard_state.a.to_f32(),
                        self.keyboard_state.q.to_f32() - self.keyboard_state.z.to_f32(),
                        self.keyboard_state.s.to_f32() - self.keyboard_state.w.to_f32(),
                    ) * (1.0 + self.keyboard_state.lshift.to_f32() * 3.0)
                } else {
                    Vector3::zero()
                },
                yaw: Rad(if is_target && self.focus { -mouse_dx as f32 } else { 0.0 }),
                pitch: Rad(if is_target && self.focus { -mouse_dy as f32 } else { 0.0 }),
                fovy: Rad(if is_target && self.focus {
                    mouse_dscroll as f32
                } else {
                    0.0
                }),
            };
            self.cameras[key].update(&delta);
        }

        self.transition_camera.update(camera::TransitionCameraUpdate {
            delta_time,
            end_camera: &self.target_camera().current_to_camera(),
        });

        if self.paused == false {
            {
                // let center = self.transition_camera.current_camera.transform.position;
                let center = Vector3::zero();
                let mut rng = rand::thread_rng();
                let p0 = Point3::from_value(-30.0) + center;
                let p1 = Point3::from_value(30.0) + center;

                for rain_drop in self.rain_drops.iter_mut() {
                    rain_drop.update(delta_time, &mut rng, p0, p1);
                }

                for _ in 0..100 {
                    if self.rain_drops.len() < self.configuration.global.rain_drop_max as usize {
                        self.rain_drops.push(rain::Particle::new(&mut rng, p0, p1));
                    }
                    if self.rain_drops.len() > self.configuration.global.rain_drop_max as usize {
                        self.rain_drops
                            .truncate(self.configuration.global.rain_drop_max as usize);
                    }
                }
            }

            self.tick += 1;
        }

        if self.vr.is_some() {
            // Pitch makes me dizzy.
            self.transition_camera.current_camera.transform.pitch = Rad(0.0);
        }
    }

    pub fn render(&mut self) {
        #[derive(Copy, Clone)]
        pub struct EyeData {
            tangents: vr::RawProjection,
            cam_to_hmd: Matrix4<f64>,
        }

        struct StereoData {
            win_size: Vector2<i32>,
            hmd_to_bdy: Matrix4<f64>,
            eyes: EyeMap<EyeData>,
        }

        let stereo_data = self
            .vr
            .as_ref()
            .map(|vr| {
                let mut poses: [vr::sys::TrackedDevicePose_t; vr::sys::k_unMaxTrackedDeviceCount as usize] =
                    unsafe { mem::zeroed() };
                // NOTE: OpenVR will block upon querying the pose for as long as
                // possible but no longer than it takes to submit the new frame. This is
                // done to render the most up-to-date application state as possible.
                vr.compositor().wait_get_poses(&mut poses[..], None).unwrap();

                let win_size = vr.system().get_recommended_render_target_size();

                let hmd_pose = poses[vr::sys::k_unTrackedDeviceIndex_Hmd as usize];
                assert!(hmd_pose.bPoseIsValid, "Received invalid pose from VR.");
                let hmd_to_bdy = Matrix4::from_hmd(hmd_pose.mDeviceToAbsoluteTracking.m).cast().unwrap();

                StereoData {
                    win_size: Vector2::new(win_size.width, win_size.height).cast().unwrap(),
                    hmd_to_bdy,
                    eyes: EyeMap::new(|eye_key| {
                        let eye = Eye::from(eye_key);
                        let cam_to_hmd = Matrix4::from_hmd(vr.system().get_eye_to_head_transform(eye))
                            .cast()
                            .unwrap();
                        EyeData {
                            tangents: vr.system().get_projection_raw(eye),
                            cam_to_hmd: cam_to_hmd,
                        }
                    }),
                }
            })
            .or_else({
                let configuration = &mut self.configuration;
                let win_size = self.win_size;
                move || {
                    if configuration.virtual_stereo.enabled {
                        let win_size = Vector2::new(win_size.width / 2.0, win_size.height)
                            .cast::<f32>()
                            .unwrap();
                        let fovy: Rad<f32> = Deg(90.0).into();
                        let fovx: Rad<f32> = fovy * (win_size.x / win_size.y);
                        let pitch: Rad<f32> = Deg(configuration.virtual_stereo.pitch_deg).into();
                        let yaw: Rad<f32> = Deg(configuration.virtual_stereo.yaw_deg).into();
                        let l = -Rad::tan(yaw + fovx * 0.5);
                        let r = -Rad::tan(yaw - fovx * 0.5);
                        let b = -Rad::tan(pitch + fovy * 0.5);
                        let t = -Rad::tan(pitch - fovy * 0.5);

                        Some(StereoData {
                            win_size: win_size.cast().unwrap(),
                            hmd_to_bdy: Matrix4::from_translation(Vector3::new(0.0, 0.2, 0.0)),
                            eyes: EyeMap {
                                left: EyeData {
                                    tangents: vr::RawProjection { l, r, b, t },
                                    cam_to_hmd: Matrix4::from_translation(Vector3::new(-0.1, 0.01, -0.01)),
                                },
                                right: EyeData {
                                    tangents: vr::RawProjection { l: -r, r: -l, b, t },
                                    cam_to_hmd: Matrix4::from_translation(Vector3::new(0.1, 0.01, -0.01)),
                                },
                            },
                        })
                    } else {
                        None
                    }
                }
            });
        // Space abbreviations:
        //  - self.(wld)
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

        self.main_parameters_vec.clear();
        self.light_params_vec.clear();
        self.cluster_resources_pool.reset();
        self.main_resources_pool.reset();

        {
            self.point_lights.clear();

            for &point_light in self.resources.point_lights.iter() {
                self.point_lights.push(point_light);
            }

            for rain_drop in self.rain_drops.iter() {
                self.point_lights.push(light::PointLight {
                    ambient: light::RGB::new(0.2000, 0.2000, 0.2000),
                    diffuse: light::RGB::new(4.0000, 4.0000, 4.0000),
                    specular: light::RGB::new(1.0000, 1.0000, 1.0000),
                    pos_in_wld: Point3::from_vec(rain_drop.position),
                    attenuation: light::AttenParams {
                        intensity: 0.3,
                        clip_near: 0.5,
                        cutoff: 0.02,
                    }
                    .into(),
                });
            }
        }

        for res in &mut self.light_resources_vec {
            res.dirty = true;
        }

        let mut light_index = None;
        let mut cluster_resources_index = None;

        if self.shader_compiler.light_space() == LightSpace::Wld {
            light_index = Some(self.light_params_vec.len());
            self.light_params_vec.push(light::LightParameters {
                wld_to_lgt: Matrix4::identity(),
                lgt_to_wld: Matrix4::identity(),
            });
        }

        let gl = &self.gl;

        struct C1 {
            camera: camera::Camera,
            hmd_to_wld: Matrix4<f64>,
            wld_to_hmd: Matrix4<f64>,
            cam_pos_in_wld: Point3<f64>,
        }

        impl C1 {
            #[inline]
            pub fn new(camera: camera::Camera, hmd_to_bdy: Matrix4<f64>) -> Self {
                let bdy_to_wld = camera.transform.pos_to_parent().cast::<f64>().unwrap();
                let hmd_to_wld = bdy_to_wld * hmd_to_bdy;
                Self {
                    camera,
                    hmd_to_wld,
                    wld_to_hmd: hmd_to_wld.invert().unwrap(),
                    cam_pos_in_wld: camera.transform.position.cast::<f64>().unwrap(),
                }
            }
        }

        struct C2 {
            cam_to_wld: Matrix4<f64>,
            wld_to_cam: Matrix4<f64>,

            cam_to_clp: Matrix4<f64>,
            clp_to_cam: Matrix4<f64>,
        }

        impl C2 {
            #[inline]
            pub fn new(c1: &C1, cam_to_hmd: Matrix4<f64>, frustrum: Frustrum<f64>) -> Self {
                let C1 { hmd_to_wld, .. } = *c1;
                let cam_to_wld = hmd_to_wld * cam_to_hmd;
                let cam_to_clp = frustrum.perspective(DEPTH_RANGE).cast::<f64>().unwrap();
                Self {
                    cam_to_wld,
                    wld_to_cam: cam_to_wld.invert().unwrap(),

                    cam_to_clp,
                    clp_to_cam: cam_to_clp.invert().unwrap(),
                }
            }
        }

        match stereo_data {
            Some(StereoData {
                win_size,
                hmd_to_bdy,
                eyes,
            }) => {
                let render_c1 = C1::new(self.transition_camera.current_camera, hmd_to_bdy);
                let cluster_c1 = C1::new(self.cameras.main.current_to_camera(), hmd_to_bdy);

                if self.shader_compiler.light_space() == LightSpace::Hmd {
                    light_index = Some(self.light_params_vec.len());
                    self.light_params_vec.push(light::LightParameters {
                        wld_to_lgt: render_c1.wld_to_hmd,
                        lgt_to_wld: render_c1.hmd_to_wld,
                    });
                }

                if self.shader_compiler.render_technique() == RenderTechnique::Clustered
                    && self.configuration.clustered_light_shading.grouping
                        == configuration::ClusteringGrouping::Enclosed
                {
                    cluster_resources_index = Some(self.cluster_resources_pool.next_unused(
                        gl,
                        ClusterParameters {
                            configuration: self.configuration.clustered_light_shading,
                            wld_to_hmd: cluster_c1.wld_to_hmd,
                            hmd_to_wld: cluster_c1.hmd_to_wld,
                        },
                    ));
                }

                for &eye_key in EYE_KEYS.iter() {
                    let EyeData { tangents, cam_to_hmd } = eyes[eye_key];

                    let render_c2 = C2::new(
                        &render_c1,
                        cam_to_hmd,
                        stereo_frustrum(&render_c1.camera.properties, tangents),
                    );
                    let cluster_c2 = C2::new(
                        &cluster_c1,
                        cam_to_hmd,
                        stereo_frustrum(&cluster_c1.camera.properties, tangents),
                    );

                    if self.shader_compiler.light_space() == LightSpace::Cam {
                        light_index = Some(self.light_params_vec.len());
                        self.light_params_vec.push(light::LightParameters {
                            wld_to_lgt: render_c2.wld_to_cam,
                            lgt_to_wld: render_c2.cam_to_wld,
                        });
                    }

                    if self.shader_compiler.render_technique() == RenderTechnique::Clustered
                        && self.configuration.clustered_light_shading.grouping
                            == configuration::ClusteringGrouping::Individual
                    {
                        cluster_resources_index = Some(self.cluster_resources_pool.next_unused(
                            gl,
                            ClusterParameters {
                                configuration: self.configuration.clustered_light_shading,
                                wld_to_hmd: cluster_c1.wld_to_hmd,
                                hmd_to_wld: cluster_c1.hmd_to_wld,
                            },
                        ));
                    }

                    if self.shader_compiler.render_technique() == RenderTechnique::Clustered {
                        let camera_resources_pool =
                            &mut self.cluster_resources_pool[cluster_resources_index.unwrap()].camera_resources_pool;
                        let C1 { ref camera, .. } = cluster_c1;
                        let C2 {
                            wld_to_cam,
                            cam_to_wld,
                            cam_to_clp,
                            clp_to_cam,
                            ..
                        } = cluster_c2;

                        let _ = camera_resources_pool.next_unused(
                            gl,
                            ClusterCameraParameters {
                                frame_dims: win_size,

                                wld_to_cam,
                                cam_to_wld,

                                cam_to_clp,
                                clp_to_cam,

                                frustum: {
                                    // NOTE: Reversed!
                                    let z0 = camera.properties.z1 as f64;
                                    let z1 = camera.properties.z0 as f64;
                                    Frustum {
                                        x0: tangents.l as f64,
                                        x1: tangents.r as f64,
                                        y0: tangents.b as f64,
                                        y1: tangents.t as f64,
                                        z0,
                                        z1,
                                    }
                                },
                            },
                        );
                    }

                    {
                        let C1 { cam_pos_in_wld, .. } = render_c1;
                        let C2 {
                            wld_to_cam,
                            cam_to_wld,
                            cam_to_clp,
                            clp_to_cam,
                            ..
                        } = render_c2;

                        self.main_parameters_vec.push(MainParameters {
                            wld_to_cam,
                            cam_to_wld,

                            cam_to_clp,
                            clp_to_cam,

                            cam_pos_in_wld,

                            light_index: light_index.expect("Programming error: light_index was never set."),
                            cluster_resources_index,

                            dimensions: win_size,
                            display_viewport: {
                                let w = self.win_size.width as i32;
                                let h = self.win_size.height as i32;

                                match eye_key {
                                    vr::Eye::Left => {
                                        Viewport::from_coordinates(Point2::new(0, 0), Point2::new(w / 2, h))
                                    }
                                    vr::Eye::Right => {
                                        Viewport::from_coordinates(Point2::new(w / 2, 0), Point2::new(w, h))
                                    }
                                }
                            },
                        });
                    }
                }
            }
            None => {
                let dimensions = Vector2::new(self.win_size.width as i32, self.win_size.height as i32);
                let render_c1 = C1::new(self.transition_camera.current_camera, Matrix4::identity());
                let cluster_c1 = C1::new(self.cameras.main.current_to_camera(), Matrix4::identity());

                if self.shader_compiler.light_space() == LightSpace::Hmd {
                    light_index = Some(self.light_params_vec.len());
                    self.light_params_vec.push(light::LightParameters {
                        wld_to_lgt: render_c1.wld_to_hmd,
                        lgt_to_wld: render_c1.hmd_to_wld,
                    });
                }

                if self.shader_compiler.render_technique() == RenderTechnique::Clustered
                    && self.configuration.clustered_light_shading.grouping
                        == configuration::ClusteringGrouping::Enclosed
                {
                    cluster_resources_index = Some(self.cluster_resources_pool.next_unused(
                        gl,
                        ClusterParameters {
                            configuration: self.configuration.clustered_light_shading,
                            wld_to_hmd: cluster_c1.wld_to_hmd,
                            hmd_to_wld: cluster_c1.hmd_to_wld,
                        },
                    ));
                }

                let render_c2 = C2::new(
                    &render_c1,
                    Matrix4::identity(),
                    mono_frustrum(&render_c1.camera, dimensions),
                );

                let cluster_c2 = C2::new(
                    &cluster_c1,
                    Matrix4::identity(),
                    mono_frustrum(&cluster_c1.camera, dimensions),
                );

                if self.shader_compiler.light_space() == LightSpace::Cam {
                    light_index = Some(self.light_params_vec.len());
                    self.light_params_vec.push(light::LightParameters {
                        wld_to_lgt: render_c2.wld_to_cam,
                        lgt_to_wld: render_c2.cam_to_wld,
                    });
                }

                if self.shader_compiler.render_technique() == RenderTechnique::Clustered
                    && self.configuration.clustered_light_shading.grouping
                        == configuration::ClusteringGrouping::Individual
                {
                    cluster_resources_index = Some(self.cluster_resources_pool.next_unused(
                        gl,
                        ClusterParameters {
                            configuration: self.configuration.clustered_light_shading,
                            wld_to_hmd: cluster_c1.wld_to_hmd,
                            hmd_to_wld: cluster_c1.hmd_to_wld,
                        },
                    ));
                }

                if self.shader_compiler.render_technique() == RenderTechnique::Clustered {
                    let camera_resources_pool =
                        &mut self.cluster_resources_pool[cluster_resources_index.unwrap()].camera_resources_pool;
                    let C1 { ref camera, .. } = cluster_c1;
                    let C2 {
                        wld_to_cam,
                        cam_to_wld,
                        cam_to_clp,
                        clp_to_cam,
                        ..
                    } = cluster_c2;

                    let _ = camera_resources_pool.next_unused(
                        gl,
                        ClusterCameraParameters {
                            frame_dims: dimensions,

                            wld_to_cam,
                            cam_to_wld,

                            cam_to_clp,
                            clp_to_cam,

                            frustum: {
                                // NOTE: Reversed!
                                let z0 = camera.properties.z1 as f64;
                                let z1 = camera.properties.z0 as f64;
                                let dy = Rad::tan(Rad(Rad::from(cluster_c1.camera.transform.fovy).0 as f64) / 2.0);
                                let dx = dy * dimensions.x as f64 / dimensions.y as f64;

                                Frustum {
                                    x0: -dx,
                                    x1: dx,
                                    y0: -dy,
                                    y1: dy,
                                    z0,
                                    z1,
                                }
                            },
                        },
                    );
                }

                {
                    let C1 { cam_pos_in_wld, .. } = render_c1;

                    let C2 {
                        wld_to_cam,
                        cam_to_wld,
                        cam_to_clp,
                        clp_to_cam,
                        ..
                    } = render_c2;

                    self.main_parameters_vec.push(MainParameters {
                        wld_to_cam,
                        cam_to_wld,

                        cam_to_clp,
                        clp_to_cam,

                        cam_pos_in_wld,

                        light_index: light_index.expect("Programming error: light_index was never set."),
                        cluster_resources_index,

                        dimensions,
                        display_viewport: Viewport::from_dimensions(dimensions),
                    });
                }
            }
        }

        for cluster_resources_index in self.cluster_resources_pool.used_index_iter() {
            // Reborrow
            let gl = &self.gl;
            let cluster_resources = &mut self.cluster_resources_pool[cluster_resources_index];
            cluster_resources.recompute();

            let cluster_count = cluster_resources.computed.cluster_count();
            let blocks_per_dispatch = cluster_count
                .div_ceil(self.configuration.prefix_sum.pass_0_threads * self.configuration.prefix_sum.pass_1_threads);
            let clusters_per_dispatch = self.configuration.prefix_sum.pass_0_threads * blocks_per_dispatch;
            let cluster_dispatch_count = cluster_count.div_ceil(clusters_per_dispatch);

            unsafe {
                let buffer = &mut cluster_resources.cluster_space_buffer;
                let data = ClusterSpaceBuffer::new(
                    cluster_resources.computed.dimensions,
                    cluster_resources.computed.frustum,
                    cluster_resources.computed.wld_to_ccam,
                );

                buffer.invalidate(gl);
                buffer.write(gl, data.value_as_bytes());
                gl.bind_buffer_base(
                    gl::UNIFORM_BUFFER,
                    cls_renderer::CLUSTER_SPACE_BUFFER_BINDING,
                    buffer.name(),
                );
            }

            unsafe {
                let buffer = &mut cluster_resources.cluster_fragment_counts_buffer;
                let byte_count = std::mem::size_of::<u32>() * cluster_resources.computed.cluster_count() as usize;
                buffer.invalidate(gl);
                // buffer.ensure_capacity(gl, byte_count);
                buffer.clear_0u32(gl, byte_count);
            }

            // NOTE: Work around borrow checker.
            for camera_resources_index in cluster_resources.camera_resources_pool.used_index_iter() {
                // Reborrow.
                let gl = &self.gl;
                let cluster_resources = &mut self.cluster_resources_pool[cluster_resources_index];
                let camera_resources = &mut cluster_resources.camera_resources_pool[camera_resources_index];
                let camera_parameters = &camera_resources.parameters;
                let profiler = &mut camera_resources.profilers.render_depth;

                profiler.start(gl, self.frame, self.epoch);

                unsafe {
                    let camera_buffer = CameraBuffer {
                        wld_to_cam: camera_parameters.wld_to_cam.cast().unwrap(),
                        cam_to_wld: camera_parameters.cam_to_wld.cast().unwrap(),

                        cam_to_clp: camera_parameters.cam_to_clp.cast().unwrap(),
                        clp_to_cam: camera_parameters.clp_to_cam.cast().unwrap(),

                        // NOTE: Doesn't matter for depth pass!
                        cam_pos_in_lgt: Vector4::zero(),
                    };

                    let buffer_index = self.camera_buffer_pool.unused(gl);
                    let buffer_name = self.camera_buffer_pool[buffer_index];

                    gl.named_buffer_data(buffer_name, camera_buffer.value_as_bytes(), gl::STREAM_DRAW);
                    gl.bind_buffer_base(gl::UNIFORM_BUFFER, rendering::CAMERA_BUFFER_BINDING, buffer_name);
                }

                let main_resources_index = self.main_resources_pool.next_unused(gl, camera_parameters.frame_dims);

                self.clear_and_render_depth(main_resources_index);

                // Reborrow.
                let gl = &self.gl;
                let cluster_resources = &mut self.cluster_resources_pool[cluster_resources_index];
                let camera_resources = &mut cluster_resources.camera_resources_pool[camera_resources_index];
                let camera_parameters = &camera_resources.parameters;
                let profiler = &mut camera_resources.profilers.render_depth;
                let main_resources = &mut self.main_resources_pool[main_resources_index];

                profiler.stop(gl, self.frame, self.epoch);

                {
                    let profiler = &mut camera_resources.profilers.count_frags;
                    profiler.start(gl, self.frame, self.epoch);

                    unsafe {
                        // gl.bind_framebuffer(gl::FRAMEBUFFER, gl::FramebufferName::Default);
                        let program = &mut self.cls_renderer.fragments_per_cluster_program;
                        program.update(&mut rendering_context!(self));
                        if let ProgramName::Linked(name) = program.name {
                            gl.use_program(name);

                            gl.bind_buffer_base(
                                gl::SHADER_STORAGE_BUFFER,
                                cls_renderer::CLUSTER_FRAGMENT_COUNTS_BUFFER_BINDING,
                                cluster_resources.cluster_fragment_counts_buffer.name(),
                            );

                            // gl.uniform_1i(cls_renderer::DEPTH_SAMPLER_LOC, 0);
                            gl.bind_texture_unit(0, main_resources.depth_texture.name());

                            gl.uniform_2f(
                                cls_renderer::FB_DIMS_LOC,
                                main_resources.dims.cast::<f32>().unwrap().into(),
                            );

                            let clp_to_wld = (camera_parameters.cam_to_wld * camera_parameters.clp_to_cam)
                                .cast::<f32>()
                                .unwrap();

                            gl.uniform_matrix4f(
                                cls_renderer::CLP_TO_WLD_LOC,
                                gl::MajorAxis::Column,
                                clp_to_wld.as_ref(),
                            );

                            gl.memory_barrier(
                                gl::MemoryBarrierFlag::TEXTURE_FETCH | gl::MemoryBarrierFlag::FRAMEBUFFER,
                            );

                            gl.dispatch_compute(
                                main_resources.dims.x.div_ceil(16) as u32,
                                main_resources.dims.y.div_ceil(16) as u32,
                                1,
                            );
                            gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                        }
                    }
                    profiler.stop(gl, self.frame, self.epoch);
                }
            }

            // Reborrow.
            let gl = &self.gl;
            let cluster_resources = &mut self.cluster_resources_pool[cluster_resources_index];

            // We have our fragments per cluster buffer here.

            {
                let profiler = &mut cluster_resources.profilers.compact_clusters;
                profiler.start(gl, self.frame, self.epoch);

                unsafe {
                    let buffer = &mut cluster_resources.offset_buffer;
                    let byte_count = std::mem::size_of::<u32>() * self.configuration.prefix_sum.pass_1_threads as usize;
                    buffer.invalidate(gl);
                    buffer.ensure_capacity(gl, byte_count);
                    buffer.clear_0u32(gl, byte_count);
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::OFFSETS_BUFFER_BINDING,
                        buffer.name(),
                    );
                }

                unsafe {
                    let buffer = &mut cluster_resources.active_cluster_indices_buffer;
                    buffer.invalidate(gl);
                    // buffer.ensure_capacity(gl, byte_count);
                    buffer.clear_0u32(gl, buffer.byte_capacity());
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::ACTIVE_CLUSTER_INDICES_BUFFER_BINDING,
                        buffer.name(),
                    );
                }

                unsafe {
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::DRAW_COMMANDS_BUFFER_BINDING,
                        cluster_resources.draw_commands_buffer.name(),
                    );
                }

                unsafe {
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::COMPUTE_COMMANDS_BUFFER_BINDING,
                        cluster_resources.compute_commands_buffer.name(),
                    );
                }

                unsafe {
                    let program = &mut self.cls_renderer.compact_clusters_0_program;
                    program.update(&mut rendering_context!(self));
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.uniform_1ui(cls_renderer::ITEM_COUNT_LOC, cluster_resources.computed.cluster_count());
                        gl.dispatch_compute(cluster_dispatch_count, 1, 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }

                unsafe {
                    let program = &mut self.cls_renderer.compact_clusters_1_program;
                    program.update(&mut rendering_context!(self));
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.uniform_1ui(cls_renderer::ITEM_COUNT_LOC, cluster_resources.computed.cluster_count());
                        gl.dispatch_compute(1, 1, 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }

                unsafe {
                    let program = &mut self.cls_renderer.compact_clusters_2_program;
                    program.update(&mut rendering_context!(self));
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.uniform_1ui(cls_renderer::ITEM_COUNT_LOC, cluster_resources.computed.cluster_count());
                        gl.dispatch_compute(cluster_dispatch_count, 1, 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }
                profiler.stop(gl, self.frame, self.epoch);
            }

            // We have our active clusters.

            {
                let profiler = &mut cluster_resources.profilers.upload_lights;
                profiler.start(gl, self.frame, self.epoch);

                unsafe {
                    let data: Vec<[f32; 4]> = self
                        .point_lights
                        .iter()
                        .map(|&light| {
                            let pos_in_ccam = cluster_resources
                                .computed
                                .wld_to_ccam
                                .transform_point(light.pos_in_wld.cast().unwrap());
                            let [x, y, z]: [f32; 3] = pos_in_ccam.cast::<f32>().unwrap().into();
                            [x, y, z, light.attenuation.clip_far]
                        })
                        .collect();
                    let bytes = data.vec_as_bytes();
                    let padded_byte_count = bytes.len().ceil_to_multiple(64);

                    let buffer = &mut cluster_resources.light_xyzr_buffer;
                    buffer.invalidate(gl);
                    buffer.ensure_capacity(gl, padded_byte_count);
                    buffer.write(gl, bytes);
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::LIGHT_XYZR_BUFFER_BINDING,
                        buffer.name(),
                    );
                }

                let profiler = &mut cluster_resources.profilers.upload_lights;
                profiler.stop(gl, self.frame, self.epoch);
            }

            {
                let profiler = &mut cluster_resources.profilers.count_lights;
                profiler.start(gl, self.frame, self.epoch);

                unsafe {
                    let buffer = &mut cluster_resources.active_cluster_light_counts_buffer;
                    buffer.invalidate(gl);
                    // buffer.ensure_capacity(gl, byte_count);
                    buffer.clear_0u32(gl, buffer.byte_capacity());
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::ACTIVE_CLUSTER_LIGHT_COUNTS_BUFFER_BINDING,
                        buffer.name(),
                    );
                }

                unsafe {
                    let program = &mut self.cls_renderer.count_lights_program;
                    program.update(&mut rendering_context!(self));
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.uniform_1ui(cls_renderer::LIGHT_COUNT_LOC, self.point_lights.len() as u32);
                        gl.bind_buffer(
                            gl::DISPATCH_INDIRECT_BUFFER,
                            cluster_resources.compute_commands_buffer.name(),
                        );
                        gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE | gl::MemoryBarrierFlag::COMMAND);
                        gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 0);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }
                profiler.stop(gl, self.frame, self.epoch);
            }

            // We have our light counts.

            {
                let profiler = &mut cluster_resources.profilers.light_offsets;
                profiler.start(gl, self.frame, self.epoch);

                unsafe {
                    let buffer = &mut cluster_resources.offset_buffer;
                    let byte_count = std::mem::size_of::<u32>() * self.configuration.prefix_sum.pass_1_threads as usize;
                    buffer.invalidate(gl);
                    buffer.ensure_capacity(gl, byte_count);
                    buffer.clear_0u32(gl, byte_count);
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::OFFSETS_BUFFER_BINDING,
                        buffer.name(),
                    );
                }

                unsafe {
                    let buffer = &mut cluster_resources.active_cluster_light_offsets_buffer;
                    buffer.invalidate(gl);
                    // buffer.ensure_capacity(gl, byte_count);
                    buffer.clear_0u32(gl, buffer.byte_capacity());
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::ACTIVE_CLUSTER_LIGHT_OFFSETS_BUFFER_BINDING,
                        buffer.name(),
                    );
                    gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE);
                }

                unsafe {
                    let program = &mut self.cls_renderer.compact_light_counts_0_program;
                    program.update(&mut rendering_context!(self));
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }

                unsafe {
                    let program = &mut self.cls_renderer.compact_light_counts_1_program;
                    program.update(&mut rendering_context!(self));
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.dispatch_compute(1, 1, 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }

                unsafe {
                    let program = &mut self.cls_renderer.compact_light_counts_2_program;
                    program.update(&mut rendering_context!(self));
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }
                profiler.stop(gl, self.frame, self.epoch);
            }

            // We have our light offsets.

            {
                let profiler = &mut cluster_resources.profilers.assign_lights;
                profiler.start(gl, self.frame, self.epoch);

                unsafe {
                    let buffer = &mut cluster_resources.light_indices_buffer;
                    buffer.invalidate(gl);
                    // buffer.ensure_capacity(gl, byte_count);
                    buffer.clear_0u32(gl, buffer.byte_capacity());
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::LIGHT_INDICES_BUFFER_BINDING,
                        buffer.name(),
                    );
                }

                unsafe {
                    let program = &mut self.cls_renderer.assign_lights_program;
                    program.update(&mut rendering_context!(self));
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.uniform_1ui(cls_renderer::LIGHT_COUNT_LOC, self.point_lights.len() as u32);
                        gl.bind_buffer(
                            gl::DISPATCH_INDIRECT_BUFFER,
                            cluster_resources.compute_commands_buffer.name(),
                        );
                        gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE | gl::MemoryBarrierFlag::COMMAND);
                        gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 0);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }
                profiler.stop(gl, self.frame, self.epoch);
            }
        }

        for main_parameters_index in 0..self.main_parameters_vec.len() {
            let main_params = &self.main_parameters_vec[main_parameters_index];

            // Reborrow.
            let gl = &self.gl;

            let MainParameters {
                ref wld_to_cam,
                ref cam_to_wld,

                ref cam_to_clp,
                ref clp_to_cam,

                cam_pos_in_wld,

                light_index,
                cluster_resources_index,

                dimensions,
                display_viewport,
            } = *main_params;

            let light_params = &self.light_params_vec[light_index];

            // Ensure light resources are available.
            while self.light_resources_vec.len() < light_index + 1 {
                self.light_resources_vec.push(light::LightResources::new(gl));
            }
            let light_resources = &mut self.light_resources_vec[light_index];

            // Ensure light resources are uploaded.
            if light_resources.dirty {
                light_resources.lights.clear();
                light_resources
                    .lights
                    .extend(self.point_lights.iter().map(|&point_light| {
                        light::LightBufferLight::from_point_light(point_light, light_params.wld_to_lgt)
                    }));

                let header = light::LightBufferHeader {
                    wld_to_lgt: light_params.wld_to_lgt.cast().unwrap(),
                    lgt_to_wld: light_params.lgt_to_wld.cast().unwrap(),

                    light_count: Vector4::new(light_resources.lights.len() as u32, 0, 0, 0),
                };

                unsafe {
                    let header_bytes = header.value_as_bytes();
                    let body_bytes = light_resources.lights.vec_as_bytes();

                    gl.named_buffer_reserve(
                        light_resources.buffer_name,
                        header_bytes.len() + body_bytes.len(),
                        gl::STREAM_DRAW,
                    );
                    gl.named_buffer_sub_data(light_resources.buffer_name, 0, header_bytes);
                    gl.named_buffer_sub_data(light_resources.buffer_name, header_bytes.len(), body_bytes);
                }
            }

            // Ensure light resources are bound.
            unsafe {
                // TODO: Make this less global. Should be in basic renderer.
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    basic_renderer::LIGHT_BUFFER_BINDING,
                    light_resources.buffer_name,
                );
            }

            let cam_pos_in_lgt = light_params.wld_to_lgt * cam_pos_in_wld.to_homogeneous();

            unsafe {
                let camera_buffer = CameraBuffer {
                    wld_to_cam: wld_to_cam.cast().unwrap(),
                    cam_to_wld: cam_to_wld.cast().unwrap(),

                    cam_to_clp: cam_to_clp.cast().unwrap(),
                    clp_to_cam: clp_to_cam.cast().unwrap(),

                    cam_pos_in_lgt: cam_pos_in_lgt.cast().unwrap(),
                };

                let buffer_index = self.camera_buffer_pool.unused(gl);
                let buffer_name = self.camera_buffer_pool[buffer_index];

                gl.named_buffer_data(buffer_name, camera_buffer.value_as_bytes(), gl::STREAM_DRAW);
                gl.bind_buffer_base(gl::UNIFORM_BUFFER, rendering::CAMERA_BUFFER_BINDING, buffer_name);
            }

            let main_resources_index = self.main_resources_pool.next_unused(gl, dimensions);

            self.clear_and_render_main(main_resources_index, cluster_resources_index);

            if self.target_camera_key == CameraKey::Debug {
                let corners_in_clp = Frustrum::corners_in_clp(DEPTH_RANGE);
                let vertices: Vec<[f32; 3]> = corners_in_clp
                    .iter()
                    .map(|point| point.cast().unwrap().into())
                    .collect();

                for cluster_resources_index in self.cluster_resources_pool.used_index_iter() {
                    let cluster_resources = &self.cluster_resources_pool[cluster_resources_index];
                    for camera_resources in cluster_resources.camera_resources_pool.used_slice().iter() {
                        self.line_renderer.render(
                            &mut rendering_context!(self),
                            &line_renderer::Parameters {
                                vertices: &vertices[..],
                                indices: &FRUSTRUM_LINE_MESH_INDICES[..],
                                obj_to_wld: &(camera_resources.parameters.cam_to_wld
                                    * camera_resources.parameters.clp_to_cam)
                                    .cast()
                                    .unwrap(),
                            },
                        );
                    }

                    {
                        // Reborrow.
                        let main_params = &self.main_parameters_vec[main_parameters_index];
                        let wld_to_clp = main_params.cam_to_clp * main_params.wld_to_cam;
                        self.render_debug_clusters(&cluster_renderer::Parameters {
                            cluster_resources_index,
                            wld_to_clp,
                        });
                    }
                }
            }

            // Reborrow.
            let main_resources = &mut self.main_resources_pool[main_resources_index];

            unsafe {
                self.gl.blit_named_framebuffer(
                    main_resources.framebuffer_name.into(),
                    gl::FramebufferName::Default,
                    0,
                    0,
                    dimensions.x,
                    dimensions.y,
                    display_viewport.p0().x,
                    display_viewport.p0().y,
                    display_viewport.p1().x,
                    display_viewport.p1().y,
                    gl::BlitMask::COLOR_BUFFER_BIT,
                    gl::NEAREST,
                );
            }
        }

        {
            let dimensions = Vector2::new(self.win_size.width as i32, self.win_size.height as i32);

            self.overlay_textbox.width = dimensions.x - 26;
            self.overlay_textbox.height = dimensions.y - 20;
            self.overlay_textbox.clear();
        }

        self.overlay_textbox.write(
            &self.monospace,
            &format!(
                "\
                 Render Technique: {:<14} | CLS Grouping:     {:<14} | CLS Projection:   {:<14}\n\
                 Attenuation Mode: {:<14} | Lighting Space:   {:<14} | Light Count:      {:<14}\n\
                 ",
                format!("{:?}", self.shader_compiler.render_technique()),
                format!("{:?}", self.configuration.clustered_light_shading.grouping),
                format!("{:?}", self.configuration.clustered_light_shading.projection),
                format!("{:?}", self.shader_compiler.attenuation_mode()),
                format!("{:?}", self.shader_compiler.light_space()),
                self.point_lights.len(),
            ),
        );

        let Self {
            ref mut overlay_textbox,
            ref monospace,
            ..
        } = *self;

        for cluster_resources_index in self.cluster_resources_pool.used_index_iter() {
            let res = &mut self.cluster_resources_pool[cluster_resources_index];
            let dimensions_u32 = res.computed.dimensions;

            overlay_textbox.write(
                &monospace,
                &format!(
                    "[{}] cluster dimensions {{ x: {:3}, y: {:3}, z: {:3} }}\n",
                    cluster_resources_index.to_usize(),
                    dimensions_u32.x,
                    dimensions_u32.y,
                    dimensions_u32.z,
                ),
            );

            for camera_resources_index in res.camera_resources_pool.used_index_iter() {
                let camera_resources = &mut res.camera_resources_pool[camera_resources_index];
                for &stage in &CameraStage::VALUES {
                    let stats = &mut camera_resources.profilers[stage].stats(self.frame);
                    if let Some(stats) = stats {
                        overlay_textbox.write(
                            &monospace,
                            &format!(
                                "[{}][{}] {:<20} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
                                cluster_resources_index.to_usize(),
                                camera_resources_index.to_usize(),
                                stage.title(),
                                stats.cpu_elapsed_min as f64 / 1000.0,
                                stats.cpu_elapsed_avg as f64 / 1000.0,
                                stats.cpu_elapsed_max as f64 / 1000.0,
                                stats.gpu_elapsed_min as f64 / 1000.0,
                                stats.gpu_elapsed_avg as f64 / 1000.0,
                                stats.gpu_elapsed_max as f64 / 1000.0,
                            ),
                        );
                    }
                }
            }

            for &stage in &ClusterStage::VALUES {
                let stats = &mut res.profilers[stage].stats(self.frame);
                if let Some(stats) = stats {
                    overlay_textbox.write(
                        &monospace,
                        &format!(
                            "[{}]    {:<20} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
                            cluster_resources_index.to_usize(),
                            stage.title(),
                            stats.cpu_elapsed_min as f64 / 1000.0,
                            stats.cpu_elapsed_avg as f64 / 1000.0,
                            stats.cpu_elapsed_max as f64 / 1000.0,
                            stats.gpu_elapsed_min as f64 / 1000.0,
                            stats.gpu_elapsed_avg as f64 / 1000.0,
                            stats.gpu_elapsed_max as f64 / 1000.0,
                        ),
                    );
                }
            }
        }

        for (main_resources_index, main_resources) in self.main_resources_pool.used_slice().iter().enumerate() {
            for (name, profiler) in [
                ("depth", &main_resources.depth_pass_profiler),
                ("basic", &main_resources.basic_pass_profiler),
            ]
            .iter()
            {
                let stats = profiler.stats(self.frame);
                if let Some(stats) = stats {
                    overlay_textbox.write(
                        &monospace,
                        &format!(
                            "[{}]    {:<20} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
                            main_resources_index,
                            name,
                            stats.cpu_elapsed_min as f64 / 1000.0,
                            stats.cpu_elapsed_avg as f64 / 1000.0,
                            stats.cpu_elapsed_max as f64 / 1000.0,
                            stats.gpu_elapsed_min as f64 / 1000.0,
                            stats.gpu_elapsed_avg as f64 / 1000.0,
                            stats.gpu_elapsed_max as f64 / 1000.0,
                        ),
                    );
                }
            }
        }

        // Reborrow.
        let gl = &self.gl;

        unsafe {
            let dimensions = Vector2::new(self.win_size.width as i32, self.win_size.height as i32);
            gl.viewport(0, 0, dimensions.x, dimensions.y);
            gl.bind_framebuffer(gl::FRAMEBUFFER, gl::FramebufferName::Default);

            self.render_text();
        }

        // Reborrow.
        let gl = &self.gl;

        self.gl_window.swap_buffers().unwrap();
        self.frame += 1;

        // TODO: Borrow the pool instead.
        self.camera_buffer_pool.reset(gl);

        // std::thread::sleep(time::Duration::from_millis(17));

        {
            let duration = {
                let now = time::Instant::now();
                let duration = now.duration_since(std::mem::replace(&mut self.last_frame_start, now));
                duration
            };
            const NANOS_PER_SEC: f32 = 1_000_000_000.0;
            let fps = NANOS_PER_SEC / (duration.as_secs() as f32 * NANOS_PER_SEC + duration.subsec_nanos() as f32);
            self.fps_average.submit(fps);
            self.gl_window.window().set_title(&format!(
                "VR Lab - {:?} - {:?} - {:02.1} FPS",
                self.target_camera_key,
                self.shader_compiler.render_technique(),
                self.fps_average.compute()
            ));
        }
    }

    fn clear_and_render_depth(&mut self, main_resources_index: MainResourcesIndex) {
        let Self {
            ref gl,
            ref clear_color,
            ref mut main_resources_pool,
            ..
        } = *self;

        let main_resources = &mut main_resources_pool[main_resources_index];

        unsafe {
            gl.viewport(0, 0, main_resources.dims.x, main_resources.dims.y);
            gl.bind_framebuffer(gl::FRAMEBUFFER, main_resources.framebuffer_name);
            gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
            // Reverse-Z.
            gl.clear_depth(0.0);
            gl.clear(gl::ClearFlag::COLOR_BUFFER | gl::ClearFlag::DEPTH_BUFFER);
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
        }

        self.render_depth();
    }

    fn clear_and_render_main(
        &mut self,
        main_resources_index: MainResourcesIndex,
        cluster_resources_index: Option<ClusterResourcesIndex>,
    ) {
        let Self {
            ref gl,
            ref clear_color,
            ..
        } = *self;

        let main_resources = &mut self.main_resources_pool[main_resources_index];

        unsafe {
            gl.viewport(0, 0, main_resources.dims.x, main_resources.dims.y);
            gl.bind_framebuffer(gl::FRAMEBUFFER, main_resources.framebuffer_name);
            gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
            // Reverse-Z.
            gl.clear_depth(0.0);
            gl.clear(gl::ClearFlag::COLOR_BUFFER | gl::ClearFlag::DEPTH_BUFFER);
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
        }

        if self.depth_prepass {
            let main_resources = &mut self.main_resources_pool[main_resources_index];
            let profiler = &mut main_resources.depth_pass_profiler;
            profiler.start(gl, self.frame, self.epoch);

            self.render_depth();

            let Self { ref gl, .. } = *self;

            unsafe {
                gl.depth_func(gl::GEQUAL);
                gl.depth_mask(gl::FALSE);
            }

            // Reborrow.
            let main_resources = &mut self.main_resources_pool[main_resources_index];
            let profiler = &mut main_resources.depth_pass_profiler;

            profiler.stop(gl, self.frame, self.epoch);
        }

        {
            // Reborrow
            let gl = &self.gl;
            let main_resources = &mut self.main_resources_pool[main_resources_index];
            let profiler = &mut main_resources.basic_pass_profiler;

            profiler.start(gl, self.frame, self.epoch);

            self.render_main(&basic_renderer::Parameters {
                mode: match self.target_camera_key {
                    CameraKey::Main => 0,
                    CameraKey::Debug => self.display_mode,
                },
                cluster_resources_index,
            });

            // Reborrow.
            let gl = &self.gl;
            let main_resources = &mut self.main_resources_pool[main_resources_index];
            let profiler = &mut main_resources.basic_pass_profiler;

            if self.depth_prepass {
                unsafe {
                    gl.depth_func(gl::GREATER);
                    gl.depth_mask(gl::TRUE);
                }
            }

            profiler.stop(gl, self.frame, self.epoch);
        }
    }
}

fn main() {
    env_logger::init();

    let mut events_loop = glutin::EventsLoop::new();

    let mut context = Context::new(&mut events_loop);

    while context.running {
        context.render();
        context.process_events(&mut events_loop);
        context.simulate();
    }

    // Save state.
    {
        use std::fs;
        use std::io;
        use std::io::Write;
        let mut file = io::BufWriter::new(fs::File::create("state.bin").unwrap());
        for key in CameraKey::iter() {
            let camera = context.cameras[key].current_to_camera();
            file.write_all(camera.value_as_bytes()).unwrap();
        }
    }
}

fn gen_texture_t(name: gl::TextureName) -> vr::sys::Texture_t {
    // NOTE(mickvangelderen): The handle is not actually a pointer in
    // OpenGL's case, it's just the texture name.
    vr::sys::Texture_t {
        handle: name.to_u32() as usize as *const c_void as *mut c_void,
        eType: vr::sys::ETextureType_TextureType_OpenGL,
        eColorSpace: vr::sys::EColorSpace_ColorSpace_Gamma, // TODO(mickvangelderen): IDK
    }
}

fn mono_frustrum(camera: &camera::Camera, dimensions: Vector2<i32>) -> Frustrum<f64> {
    let z0 = camera.properties.z0 as f64;
    let z1 = camera.properties.z1 as f64;
    let dy = -z0 * Rad::tan(Rad(Rad::from(camera.transform.fovy).0 as f64) / 2.0);
    let dx = dy * dimensions.x as f64 / dimensions.y as f64;
    Frustrum::<f64> {
        x0: -dx,
        x1: dx,
        y0: -dy,
        y1: dy,
        z0,
        z1,
    }
}

fn stereo_frustrum(camera_properties: &camera::CameraProperties, tangents: vr::RawProjection) -> Frustrum<f64> {
    let vr::RawProjection { l, r, b, t } = tangents;
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
