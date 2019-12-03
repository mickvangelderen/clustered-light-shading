#![allow(non_snake_case)]

// Has to go first.
#[macro_use]
mod macros;

#[macro_use]
extern crate dds;

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
mod bmp;
pub mod cgmath_ext;
mod cls;
pub mod color;
mod cube_mesh;
mod dds_ext;
mod depth_renderer;
mod filters;
mod frame_downloader;
pub mod gl_ext;
mod glutin_ext;
mod keyboard;
mod light;
mod light_renderer;
mod line_renderer;
mod main_resources;
mod math;
mod overlay_renderer;
mod pool;
mod rain;
mod rendering;
mod resources;
mod shader_compiler;
mod symlink;
mod text_renderer;
mod text_rendering;
mod toggle;
mod viewport;
mod window_mode;

use renderer::frustum::Frustum;
use renderer::range::Range3;

use self::cgmath_ext::*;
use self::cls::*;
use self::dds_ext::*;
use self::gl_ext::*;
use self::main_resources::*;
use self::math::CeiledDiv;
use self::pool::Pool;
use self::rendering::*;
use self::resources::Resources;
use self::shader_compiler::{EntryPoint, ShaderCompiler};
use self::text_rendering::{FontContext, TextBox};
use self::viewport::*;
use self::window_mode::*;
use crate::frame_downloader::FrameDownloader;
use crate::symlink::symlink_dir;

use cgmath::*;
use derive::EnumNext;
use glutin_ext::*;
use keyboard::*;
use openvr as vr;
use renderer::camera;
use renderer::profiling::*;
use renderer::*;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
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

#[derive(Debug)]
pub struct CameraParameters {
    pub frustum: Frustum<f64>,

    pub range: Range3<f64>,

    pub wld_to_cam: Matrix4<f64>,
    pub cam_to_wld: Matrix4<f64>,

    pub cam_to_clp: Matrix4<f64>,
    pub clp_to_cam: Matrix4<f64>,

    pub wld_to_clp: Matrix4<f64>,
    pub clp_to_wld: Matrix4<f64>,
}

impl CameraParameters {
    pub fn new(wld_to_cam: Matrix4<f64>, cam_to_wld: Matrix4<f64>, frustum: Frustum<f64>, range: Range3<f64>) -> Self {
        let cam_to_clp = frustum.perspective(&range);
        let clp_to_cam = frustum.inverse_perspective(&range);

        Self {
            frustum,
            range,

            wld_to_cam,
            cam_to_wld,

            cam_to_clp,
            clp_to_cam,

            wld_to_clp: cam_to_clp * wld_to_cam,
            clp_to_wld: cam_to_wld * clp_to_cam,
        }
    }
}

pub struct MainParameters {
    pub camera: CameraParameters,

    pub cam_pos_in_lgt: Point3<f64>,

    pub draw_resources_index: usize,
    pub light_index: usize,
    pub cluster_resources_index: Option<ClusterResourcesIndex>,

    pub dimensions: Vector2<i32>,
    pub display_viewport: Viewport<i32>,
}

const RENDER_RANGE: Range3<f64> = Range3 {
    x0: -1.0,
    x1: 1.0,
    y0: -1.0,
    y1: 1.0,
    z0: 0.0, // NOTE(mickvangelderen): We use reverse-z.
    z1: 1.0,
};

const SEED: [u8; 32] = *b"this is rdm rng seed of 32 bytes";

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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum WindowEvent {
    CloseRequested,
    HiDpiFactorChanged(f64),
    Focused(bool),
    Resized(glutin::dpi::LogicalSize),
}

impl WindowEvent {
    fn from_glutin(event: glutin::WindowEvent) -> Option<Self> {
        match event {
            glutin::WindowEvent::CloseRequested => Some(WindowEvent::CloseRequested),
            glutin::WindowEvent::HiDpiFactorChanged(x) => Some(WindowEvent::HiDpiFactorChanged(x)),
            glutin::WindowEvent::Focused(x) => Some(WindowEvent::Focused(x)),
            glutin::WindowEvent::Resized(x) => Some(WindowEvent::Resized(x)),
            _ => None,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum FrameEvent {
    WindowEvent(WindowEvent),
    DeviceKey(glutin::KeyboardInput),
    DeviceMotion { axis: glutin::AxisId, value: f64 },
}

type FrameEvents = Vec<FrameEvent>;

pub struct Paths {
    pub current_dir: PathBuf,
    pub resource_dir: PathBuf,
    pub base_profiling_dir: PathBuf,
    pub current_profiling_dir: PathBuf,
    pub frames_dir: PathBuf,
    pub configuration_path: PathBuf,
}

pub struct MainContext {
    pub paths: Paths,
    pub configuration: Configuration,
    pub events_loop: glutin::EventsLoop,
    pub gl_window: glutin::GlWindow,
    pub gl: gl::Gl,
    pub vr: Option<vr::Context>,
    pub fs_watcher: notify::RecommendedWatcher,
    pub fs_rx: mpsc::Receiver<notify::DebouncedEvent>,

    pub record_file: Option<io::BufWriter<fs::File>>,
    pub current: ::incremental::Current,
    pub shader_compiler: ShaderCompiler,
    pub profiling_context: ProfilingContext,
    pub replay_frame_events: Option<Vec<FrameEvents>>,
    pub initial_cameras: CameraMap<camera::Camera>,
    pub initial_win_dpi: f64,
    pub initial_win_size: glutin::dpi::PhysicalSize,

    // Text rendering.
    pub sans_serif: FontContext,
    pub monospace: FontContext,

    // Renderers
    pub depth_renderer: depth_renderer::Renderer,
    pub line_renderer: line_renderer::Renderer,
    pub basic_renderer: basic_renderer::Renderer,
    pub light_renderer: light_renderer::Renderer,
    pub overlay_renderer: overlay_renderer::Renderer,
    pub cluster_renderer: cluster_renderer::Renderer,
    pub text_renderer: text_renderer::Renderer,
    pub cls_renderer: cls_renderer::Renderer,

    // More opengl resources...
    pub resources: Resources,
    pub frame_downloader: FrameDownloader,

    // Per-frame resources
    pub frame_sample_index: SampleIndex,
    pub main_parameters_vec: Vec<MainParameters>,
    pub camera_buffer_pool: BufferPool,
    pub light_resources_vec: Vec<light::LightResources>,
    pub light_params_vec: Vec<light::LightParameters>,
    pub cluster_resources_pool: ClusterResourcesPool,
    pub main_resources_pool: MainResourcesPool,
    pub point_lights: Vec<light::PointLight>,
}

impl MainContext {
    fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap();
        let resource_dir = current_dir.join("resources");

        let configuration_path = resource_dir.join(Configuration::DEFAULT_PATH);
        let configuration = Configuration::read(&configuration_path);

        let mut events_loop = glutin::EventsLoop::new();
        let gl_window = create_window(&mut events_loop, &configuration.window).unwrap();
        let gl = create_gl(&gl_window, &configuration.gl);
        let vr = vr::Context::new(vr::ApplicationType::Scene)
            .map_err(|error| {
                eprintln!("Failed to acquire context: {:?}", error);
            })
            .ok();

        let (fs_tx, fs_rx) = mpsc::channel();
        let mut fs_watcher = notify::watcher(fs_tx, time::Duration::from_millis(100)).unwrap();
        notify::Watcher::watch(&mut fs_watcher, &resource_dir, notify::RecursiveMode::Recursive).unwrap();

        let base_profiling_dir: PathBuf = current_dir.join("profiling");
        let latest_profiling_dir = base_profiling_dir.join("latest");
        let current_profiling_dir =
            base_profiling_dir.join(chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string());
        let frames_dir = current_profiling_dir.join("frames");

        let mut profiling_context = {
            if configuration.profiling.path.is_some() {
                std::fs::create_dir_all(&current_profiling_dir).unwrap();
                std::fs::remove_file(&latest_profiling_dir)
                    .or_else(|error| match error.kind() {
                        std::io::ErrorKind::NotFound => Ok(()),
                        _ => Err(error),
                    })
                    .unwrap();
                symlink_dir(&current_profiling_dir, &latest_profiling_dir).unwrap();
                std::fs::copy(
                    &configuration_path,
                    current_profiling_dir.join(Configuration::DEFAULT_PATH),
                )
                .unwrap();

                // Make sure we can write out the frames.
                std::fs::create_dir_all(&frames_dir).unwrap();
            }

            ProfilingContext::new(&gl, current_profiling_dir.as_path(), &configuration.profiling)
        };

        let mut record_file = match configuration.global.mode {
            ApplicationMode::Record => Some(io::BufWriter::new(
                fs::File::create(&configuration.record.path).unwrap(),
            )),
            _ => None,
        };

        let mut replay_file = match configuration.global.mode {
            ApplicationMode::Replay => Some(io::BufReader::new(fs::File::open(&configuration.replay.path).unwrap())),
            _ => None,
        };

        let default_camera_transform = camera::CameraTransform {
            position: Point3::new(0.0, 1.0, 1.5),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            fovy: Deg(90.0).into(),
        };

        let mut initial_cameras = CameraMap::new(|key| camera::Camera {
            properties: match key {
                CameraKey::Main => configuration.main_camera,
                CameraKey::Debug => configuration.debug_camera,
            }
            .into(),
            transform: default_camera_transform,
        });

        // Load state.
        {
            let read_cameras = |file: &mut std::io::BufReader<std::fs::File>,
                                initial_cameras: &mut CameraMap<camera::Camera>| unsafe {
                for key in CameraKey::iter() {
                    file.read_exact(initial_cameras[key].value_as_bytes_mut())
                        .unwrap_or_else(|_| eprintln!("Failed to read state file."));
                }
            };

            match replay_file.as_mut() {
                Some(file) => {
                    read_cameras(file, &mut initial_cameras);
                }
                None => {
                    match fs::File::open("state.bin") {
                        Ok(file) => {
                            let mut file = io::BufReader::new(file);
                            read_cameras(&mut file, &mut initial_cameras);
                        }
                        Err(_) => {
                            // Whatever.
                        }
                    }
                }
            }
        }

        let replay_frame_events = replay_file.as_mut().map(|file| {
            let mut accumulator = Vec::new();

            while let Ok(events) = bincode::deserialize_from(&mut *file) {
                accumulator.push(events);
            }

            accumulator
        });

        if let Some(file) = record_file.as_mut() {
            for key in CameraKey::iter() {
                let camera = initial_cameras[key];
                file.write_all(camera.value_as_bytes()).unwrap();
            }
        }

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
                profiling: shader_compiler::ProfilingVariables { time_sensitive: false },
                sample_count: configuration.global.sample_count,
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
        let light_renderer = light_renderer::Renderer::new(&mut rendering_context);
        let overlay_renderer = overlay_renderer::Renderer::new(&mut rendering_context);
        let cluster_renderer = cluster_renderer::Renderer::new(&mut rendering_context);
        let text_renderer = text_renderer::Renderer::new(&mut rendering_context);
        let cls_renderer = cls_renderer::Renderer::new(&mut rendering_context);

        drop(rendering_context);

        let resources = resources::Resources::new(&gl, &resource_dir, &configuration);
        let frame_downloader = FrameDownloader::new(&gl);

        let initial_win_dpi = gl_window.get_hidpi_factor();
        let initial_win_size = gl_window.get_inner_size().unwrap().to_physical(initial_win_dpi);

        Self {
            paths: Paths {
                current_dir,
                resource_dir,
                base_profiling_dir,
                current_profiling_dir,
                frames_dir,
                configuration_path,
            },
            configuration,
            events_loop,
            gl_window,
            gl,
            vr,
            fs_watcher,
            fs_rx,
            record_file,
            current,
            shader_compiler,
            replay_frame_events,
            initial_cameras,
            initial_win_dpi,
            initial_win_size,
            sans_serif,
            monospace,
            depth_renderer,
            line_renderer,
            basic_renderer,
            light_renderer,
            overlay_renderer,
            cluster_renderer,
            text_renderer,
            cls_renderer,
            resources,
            frame_downloader,
            frame_sample_index: profiling_context.add_sample("frame"),
            camera_buffer_pool: BufferPool::new(),
            light_resources_vec: Vec::new(),
            light_params_vec: Vec::new(),
            main_parameters_vec: Vec::new(),
            cluster_resources_pool: ClusterResourcesPool::new(),
            main_resources_pool: MainResourcesPool::new(),
            point_lights: Vec::new(),
            profiling_context,
        }
    }
}

pub struct Context<'s> {
    // From MainContext
    pub paths: &'s Paths,
    pub configuration: Configuration,
    pub events_loop: &'s mut glutin::EventsLoop,
    pub gl_window: &'s mut glutin::GlWindow,
    pub gl: &'s gl::Gl,
    pub vr: &'s mut Option<vr::Context>,
    pub fs_rx: &'s mut mpsc::Receiver<notify::DebouncedEvent>,

    pub record_file: &'s mut Option<io::BufWriter<fs::File>>,
    pub current: &'s mut ::incremental::Current,
    pub shader_compiler: &'s mut ShaderCompiler,
    pub profiling_context: &'s mut ProfilingContext,
    pub replay_frame_events: &'s Option<Vec<FrameEvents>>,

    // Text rendering.
    pub sans_serif: &'s mut FontContext,
    pub monospace: &'s mut FontContext,

    // Renderers
    pub depth_renderer: &'s mut depth_renderer::Renderer,
    pub line_renderer: &'s mut line_renderer::Renderer,
    pub basic_renderer: &'s mut basic_renderer::Renderer,
    pub light_renderer: &'s mut light_renderer::Renderer,
    pub overlay_renderer: &'s mut overlay_renderer::Renderer,
    pub cluster_renderer: &'s mut cluster_renderer::Renderer,
    pub text_renderer: &'s mut text_renderer::Renderer,
    pub cls_renderer: &'s mut cls_renderer::Renderer,

    // More opengl resources...
    pub resources: &'s mut Resources,
    pub frame_downloader: &'s mut FrameDownloader,

    // Per-frame resources
    pub frame_sample_index: SampleIndex,
    pub main_parameters_vec: &'s mut Vec<MainParameters>,
    pub camera_buffer_pool: &'s mut BufferPool,
    pub light_resources_vec: &'s mut Vec<light::LightResources>,
    pub light_params_vec: &'s mut Vec<light::LightParameters>,
    pub cluster_resources_pool: &'s mut ClusterResourcesPool,
    pub main_resources_pool: &'s mut MainResourcesPool,
    pub point_lights: &'s mut Vec<light::PointLight>,

    pub rng: StdRng,
    pub overlay_textbox: TextBox,
    pub running: bool,
    pub paused: bool,
    pub focus: bool,
    pub tick: u64,
    pub event_index: usize,
    pub frame_index: FrameIndex,
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

    // Window.
    pub window_event_accumulator: WindowEventAccumulator,

    // FPS counter
    pub fps_average: filters::MovingAverageF32,
    pub last_frame_start: time::Instant,
}

impl<'s> Context<'s> {
    pub fn new(context: &'s mut MainContext) -> Self {
        let MainContext {
            ref paths,
            ref configuration,
            initial_win_dpi,
            initial_win_size,
            ref mut events_loop,
            ref mut gl_window,
            ref gl,
            ref mut vr,
            ref mut fs_rx,
            ref mut record_file,
            ref mut current,
            ref mut shader_compiler,
            ref mut profiling_context,
            ref replay_frame_events,
            ref initial_cameras,
            ref mut sans_serif,
            ref mut monospace,
            ref mut depth_renderer,
            ref mut line_renderer,
            ref mut basic_renderer,
            ref mut light_renderer,
            ref mut overlay_renderer,
            ref mut cluster_renderer,
            ref mut text_renderer,
            ref mut cls_renderer,
            ref mut resources,
            ref mut frame_downloader,
            frame_sample_index,
            ref mut main_parameters_vec,
            ref mut camera_buffer_pool,
            ref mut light_resources_vec,
            ref mut light_params_vec,
            ref mut cluster_resources_pool,
            ref mut main_resources_pool,
            ref mut point_lights,
            ..
        } = *context;

        // Clone starting configuration.
        let configuration = configuration.clone();

        let transition_camera = camera::TransitionCamera {
            start_camera: initial_cameras.main,
            current_camera: initial_cameras.main,
            progress: 0.0,
        };
        let cameras = initial_cameras.map(|camera| camera::SmoothCamera {
            properties: camera.properties,
            current_transform: camera.transform,
            target_transform: camera.transform,
            smooth_enabled: true,
            current_smoothness: configuration.camera.maximum_smoothness,
            maximum_smoothness: configuration.camera.maximum_smoothness,
        });

        Context {
            paths,
            configuration,
            events_loop,
            gl_window,
            gl,
            vr,
            fs_rx,

            record_file,
            current,
            shader_compiler,
            profiling_context,
            replay_frame_events,

            // Text rendering.
            sans_serif,
            monospace,

            // Renderers
            depth_renderer,
            line_renderer,
            basic_renderer,
            light_renderer,
            overlay_renderer,
            cluster_renderer,
            text_renderer,
            cls_renderer,

            // More opengl resources...
            resources,
            frame_downloader,

            // Per-frame resources
            frame_sample_index,
            main_parameters_vec,
            camera_buffer_pool,
            light_resources_vec,
            light_params_vec,
            cluster_resources_pool,
            main_resources_pool,
            point_lights,

            rng: SeedableRng::from_seed(SEED),
            running: true,
            paused: false,
            focus: false,
            tick: 0,
            event_index: 0,
            frame_index: FrameIndex::from_usize(0),
            keyboard_state: Default::default(),
            win_dpi: initial_win_dpi,
            win_size: initial_win_size,
            clear_color: [0.0; 3],
            window_mode: WindowMode::Main,
            display_mode: 1,
            depth_prepass: true,
            target_camera_key: CameraKey::Main,
            transition_camera,
            cameras,
            rain_drops: Vec::new(),
            window_event_accumulator: Default::default(),
            overlay_textbox: TextBox::new(
                13,
                10,
                initial_win_size.width as i32 - 26,
                initial_win_size.height as i32 - 20,
            ),
            fps_average: filters::MovingAverageF32::new(0.0),
            last_frame_start: Instant::now(),
        }
    }

    fn target_camera(&self) -> &camera::SmoothCamera {
        &self.cameras[self.target_camera_key]
    }

    pub fn process_events(&mut self) {
        self.process_file_events();
        self.process_window_events();
        self.process_vr_events();
        self.event_index += 1;
    }

    fn process_file_events(&mut self) {
        let mut configuration_update = false;

        for event in self.fs_rx.try_iter() {
            match event {
                notify::DebouncedEvent::NoticeWrite(path) => {
                    info!(
                        "Noticed write to file {:?}",
                        path.strip_prefix(&self.paths.resource_dir).unwrap().display()
                    );

                    if let Some(source_index) = self.shader_compiler.memory.source_index(&path) {
                        self.shader_compiler
                            .source_mut(source_index)
                            .last_modified
                            .modify(&mut self.current);
                    }

                    if &path == &self.paths.configuration_path {
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
            self.configuration = Configuration::read(&self.paths.configuration_path);

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
            self.shader_compiler
                .replace_sample_count(&mut self.current, self.configuration.global.sample_count);

            unsafe {
                let gl = &self.gl;
                if self.configuration.gl.framebuffer_srgb {
                    gl.enable(gl::FRAMEBUFFER_SRGB);
                } else {
                    gl.disable(gl::FRAMEBUFFER_SRGB);
                }
            }
        }
    }

    fn process_window_events(&mut self) {
        let mut new_target_camera_key = self.target_camera_key;
        let new_window_mode = self.window_mode;
        let mut new_light_space = self.shader_compiler.light_space();
        let mut new_attenuation_mode = self.shader_compiler.attenuation_mode();
        let mut new_render_technique = self.shader_compiler.render_technique();
        let mut reset_debug_camera = false;

        let mut frame_events: Vec<FrameEvent> = Vec::new();

        self.events_loop.poll_events(|event| {
            use glutin::Event;
            match event {
                Event::WindowEvent { event, .. } => {
                    if let Some(event) = WindowEvent::from_glutin(event).map(FrameEvent::WindowEvent) {
                        frame_events.push(event)
                    }
                }
                Event::DeviceEvent { event, .. } => {
                    use glutin::DeviceEvent;
                    match event {
                        DeviceEvent::Key(keyboard_input) => {
                            frame_events.push(FrameEvent::DeviceKey(keyboard_input));
                        }
                        DeviceEvent::Motion { axis, value } => {
                            frame_events.push(FrameEvent::DeviceMotion { axis, value });
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        });

        if let Some(file) = self.record_file.as_mut() {
            bincode::serialize_into(file, &frame_events).unwrap();
        }

        for event in match self.replay_frame_events {
            Some(ref replay_frame_events) => replay_frame_events[self.event_index].iter(),
            None => frame_events.iter(),
        } {
            match *event {
                FrameEvent::WindowEvent(ref event) => match *event {
                    WindowEvent::CloseRequested => self.running = false,
                    WindowEvent::HiDpiFactorChanged(val) => {
                        let size = self.win_size.to_logical(self.win_dpi);
                        self.win_dpi = val;
                        self.win_size = size.to_physical(val);
                    }
                    WindowEvent::Focused(val) => self.focus = val,
                    WindowEvent::Resized(val) => {
                        self.win_size = val.to_physical(self.win_dpi);
                    }
                },
                FrameEvent::DeviceKey(keyboard_input) => {
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
                                    self.cameras[self.target_camera_key].toggle_smoothness();
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
                FrameEvent::DeviceMotion { axis, value } => {
                    if self.focus {
                        match axis {
                            0 => self.window_event_accumulator.mouse_delta.x += value,
                            1 => self.window_event_accumulator.mouse_delta.y += value,
                            3 => self.window_event_accumulator.scroll_delta += value,
                            _ => (),
                        }
                    }
                }
            }
        }

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
            if self.target_camera_key == CameraKey::Debug {
                self.transition_camera.start_transition();
            }
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
                let center = Point3::origin();
                let rng = &mut self.rng;
                let p0 = (center + Vector3::from(self.configuration.rain.bounds_min.to_array()))
                    .cast()
                    .unwrap();
                let p1 = (center + Vector3::from(self.configuration.rain.bounds_max.to_array()))
                    .cast()
                    .unwrap();

                let max_count = self.configuration.rain.max_count as usize;
                let seconds_to_bottom = (p1.y - p0.y) / 17.0; // 17 from rain.rs
                let frames_to_bottom = DESIRED_UPS as f32 * seconds_to_bottom;
                let drops_per_update = max_count as f32 / frames_to_bottom;
                for _ in 0..drops_per_update as usize {
                    if self.rain_drops.len() < max_count {
                        self.rain_drops.push(rain::Particle::new(rng, p0, p1));
                    }
                    if self.rain_drops.len() > max_count {
                        self.rain_drops.truncate(max_count);
                    }
                }

                for rain_drop in self.rain_drops.iter_mut() {
                    rain_drop.update(delta_time, rng, p0, p1);
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
        self.resources.draw_resources_pool.reset();

        {
            self.point_lights.clear();

            for &point_light in self.resources.point_lights.iter() {
                self.point_lights.push(point_light);
            }

            for rain_drop in self.rain_drops.iter() {
                self.point_lights.push(light::PointLight {
                    ambient: light::RGB::new(1.0000, 1.0000, 1.0000),
                    diffuse: {
                        let Vector3 { x, y, z } = rain_drop.tint;
                        light::RGB::new(x, y, z)
                    },
                    specular: light::RGB::new(1.0000, 1.0000, 1.0000),
                    pos_in_wld: Point3::from_vec(rain_drop.position),
                    attenuation: light::AttenParams {
                        intensity: self.configuration.rain.intensity as f32,
                        clip_near: self.configuration.rain.clip_near as f32,
                        cutoff: self.configuration.rain.cut_off as f32,
                    }
                    .into(),
                });
            }
        }

        for res in self.light_resources_vec.iter_mut() {
            res.dirty = true;
        }

        self.profiling_context.begin_frame(self.gl, self.frame_index);

        let profiler_index = self.profiling_context.start(self.gl, self.frame_sample_index);

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
        }

        impl C1 {
            #[inline]
            pub fn new(camera: camera::Camera, hmd_to_bdy: Matrix4<f64>) -> Self {
                let bdy_to_wld = camera.transform.pos_to_parent().cast::<f64>().unwrap();
                let hmd_to_wld = bdy_to_wld * hmd_to_bdy;
                Self {
                    camera,
                    hmd_to_wld,
                }
            }
        }

        struct C2 {
            wld_to_cam: Matrix4<f64>,
            cam_to_wld: Matrix4<f64>,
        }

        impl C2 {
            #[inline]
            pub fn new(c1: &C1, cam_to_hmd: Matrix4<f64>) -> Self {
                let cam_to_wld = c1.hmd_to_wld * cam_to_hmd;
                Self {
                    wld_to_cam: cam_to_wld.invert().unwrap(),
                    cam_to_wld,
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

                if self.shader_compiler.render_technique() == RenderTechnique::Clustered
                    && self.configuration.clustered_light_shading.grouping == ClusteringGrouping::Enclosed
                {
                    cluster_resources_index = Some(self.cluster_resources_pool.next_unused(
                        gl,
                        &mut self.profiling_context,
                        ClusterParameters {
                            configuration: self.configuration.clustered_light_shading,
                            clu_ori_to_wld: cluster_c1.hmd_to_wld,
                        },
                    ));
                }

                for &eye_key in EYE_KEYS.iter() {
                    let EyeData { tangents, cam_to_hmd } = eyes[eye_key];

                    let render_c3 = {
                        let C2 { wld_to_cam, cam_to_wld } = C2::new(&render_c1, cam_to_hmd);
                        CameraParameters::new(
                            wld_to_cam,
                            cam_to_wld,
                            stereo_frustum(&render_c1.camera.properties, tangents),
                            RENDER_RANGE,
                        )
                    };

                    let cluster_c3 = {
                        let C2 { wld_to_cam, cam_to_wld } = C2::new(&cluster_c1, cam_to_hmd);
                        CameraParameters::new(
                            wld_to_cam,
                            cam_to_wld,
                            stereo_frustum(&cluster_c1.camera.properties, tangents),
                            RENDER_RANGE,
                        )
                    };

                    if self.shader_compiler.render_technique() == RenderTechnique::Clustered
                        && self.configuration.clustered_light_shading.grouping == ClusteringGrouping::Individual
                    {
                        cluster_resources_index = Some(self.cluster_resources_pool.next_unused(
                            gl,
                            &mut self.profiling_context,
                            ClusterParameters {
                                configuration: self.configuration.clustered_light_shading,
                                clu_ori_to_wld: cluster_c1.hmd_to_wld,
                            },
                        ));
                    }

                    if self.shader_compiler.render_technique() == RenderTechnique::Clustered {
                        let camera_resources_pool =
                            &mut self.cluster_resources_pool[cluster_resources_index.unwrap()].camera_resources_pool;

                        let draw_resources_index = self.resources.draw_resources_pool.next({
                            let gl = &self.gl;
                            let profiling_context = &mut self.profiling_context;
                            move || resources::DrawResources::new(gl, profiling_context)
                        });

                        let _ = camera_resources_pool.next_unused(
                            gl,
                            &mut self.profiling_context,
                            ClusterCameraParameters {
                                frame_dims: win_size,
                                draw_resources_index,
                                camera: cluster_c3,
                            },
                        );
                    }

                    {
                        let cam_pos_in_wld = Point3::from_vec(render_c3.cam_to_wld[3].truncate());

                        self.main_parameters_vec.push(MainParameters {
                            camera: render_c3,

                            cam_pos_in_lgt: cam_pos_in_wld,

                            draw_resources_index: self.resources.draw_resources_pool.next({
                                let gl = &self.gl;
                                let profiling_context = &mut self.profiling_context;
                                move || resources::DrawResources::new(gl, profiling_context)
                            }),

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

                if self.shader_compiler.render_technique() == RenderTechnique::Clustered
                    && self.configuration.clustered_light_shading.grouping == ClusteringGrouping::Enclosed
                {
                    cluster_resources_index = Some(self.cluster_resources_pool.next_unused(
                        gl,
                        &mut self.profiling_context,
                        ClusterParameters {
                            configuration: self.configuration.clustered_light_shading,
                            clu_ori_to_wld: cluster_c1.hmd_to_wld,
                        },
                    ));
                }

                let render_c3 = {
                    let C2 { wld_to_cam, cam_to_wld } = C2::new(&render_c1, Matrix4::identity());
                    CameraParameters::new(
                        wld_to_cam,
                        cam_to_wld,
                        mono_frustum(&render_c1.camera, dimensions),
                        RENDER_RANGE,
                    )
                };

                let cluster_c3 = {
                    let C2 { wld_to_cam, cam_to_wld } = C2::new(&cluster_c1, Matrix4::identity());
                    CameraParameters::new(
                        wld_to_cam,
                        cam_to_wld,
                        mono_frustum(&cluster_c1.camera, dimensions),
                        RENDER_RANGE,
                    )
                };

                if self.shader_compiler.render_technique() == RenderTechnique::Clustered
                    && self.configuration.clustered_light_shading.grouping == ClusteringGrouping::Individual
                {
                    cluster_resources_index = Some(self.cluster_resources_pool.next_unused(
                        gl,
                        &mut self.profiling_context,
                        ClusterParameters {
                            configuration: self.configuration.clustered_light_shading,
                            clu_ori_to_wld: cluster_c1.hmd_to_wld,
                        },
                    ));
                }

                if self.shader_compiler.render_technique() == RenderTechnique::Clustered {
                    let camera_resources_pool =
                        &mut self.cluster_resources_pool[cluster_resources_index.unwrap()].camera_resources_pool;
                    let draw_resources_index = self.resources.draw_resources_pool.next({
                        let gl = &self.gl;
                        let profiling_context = &mut self.profiling_context;
                        move || resources::DrawResources::new(gl, profiling_context)
                    });

                    let _ = camera_resources_pool.next_unused(
                        gl,
                        &mut self.profiling_context,
                        ClusterCameraParameters {
                            frame_dims: dimensions,
                            draw_resources_index,
                            camera: cluster_c3,
                        },
                    );
                }

                {
                    let cam_pos_in_wld = Point3::from_vec(render_c3.cam_to_wld[3].truncate());

                    let draw_resources_index = self.resources.draw_resources_pool.next({
                        let gl = &self.gl;
                        let profiling_context = &mut self.profiling_context;
                        move || resources::DrawResources::new(gl, profiling_context)
                    });

                    self.main_parameters_vec.push(MainParameters {
                        camera: render_c3,

                        cam_pos_in_lgt: cam_pos_in_wld,

                        draw_resources_index,
                        light_index: light_index.expect("Programming error: light_index was never set."),
                        cluster_resources_index,

                        dimensions,
                        display_viewport: Viewport::from_dimensions(dimensions),
                    });
                }
            }
        }

        for cluster_resources_index in self.cluster_resources_pool.used_index_iter() {
            self.compute_clustering(cluster_resources_index)
        }

        for main_parameters_index in 0..self.main_parameters_vec.len() {
            let main_params = &self.main_parameters_vec[main_parameters_index];

            // Reborrow.
            let gl = &self.gl;

            let MainParameters {
                draw_resources_index,
                light_index,
                cluster_resources_index,

                dimensions,
                display_viewport,
                ..
            } = *main_params;

            let draw_resources = &mut self.resources.draw_resources_pool[draw_resources_index];

            draw_resources.recompute(
                &self.gl,
                &mut self.profiling_context,
                main_params.camera.wld_to_clp,
                if let Some(cluster_resources_index) = cluster_resources_index {
                    self.cluster_resources_pool[cluster_resources_index]
                        .computed
                        .wld_to_clu_cam
                } else {
                    Matrix4::identity()
                },
                &self.resources.scene_file.instances,
                &self.resources.materials,
                &self.resources.scene_file.transforms,
                &self.resources.scene_file.mesh_descriptions,
            );

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

            let profiling_basic_buffer = match self.profiling_context.time_sensitive() {
                true => None,
                false => unsafe {
                    let view = self.profiling_context.begin_basic_buffer(gl);
                    gl.bind_buffer_range(
                        gl::ATOMIC_COUNTER_BUFFER,
                        basic_renderer::BASIC_ATOMIC_BINDING,
                        view.name(),
                        view.byte_offset(),
                        view.byte_count(),
                    );
                    Some(view)
                },
            };

            let main_resources_index = self.main_resources_pool.next_unused(
                gl,
                &mut self.profiling_context,
                dimensions,
                self.configuration.global.sample_count,
            );

            self.clear_and_render_main(main_resources_index, main_parameters_index);

            if let Some(view) = profiling_basic_buffer {
                unsafe {
                    self.profiling_context.end_basic_buffer(self.gl, view);
                }
            }

            if self.configuration.global.render_lights {
                self.render_lights(&light_renderer::Parameters {
                    main_parameters_index,
                    light_resources_index: light_index,
                });
            }

            if self.target_camera_key == CameraKey::Debug {
                let vertices: Vec<[f32; 3]> = RENDER_RANGE
                    .vertices()
                    .iter()
                    .map(|point| point.cast().unwrap().into())
                    .collect();

                for cluster_resources_index in self.cluster_resources_pool.used_index_iter() {
                    // Reborrow
                    let main_params = &self.main_parameters_vec[main_parameters_index];
                    let camera = &main_params.camera;

                    let cluster_resources = &self.cluster_resources_pool[cluster_resources_index];
                    for camera_resources in cluster_resources.camera_resources_pool.used_slice().iter() {
                        self.line_renderer.render(
                            &mut rendering_context!(self),
                            &line_renderer::Parameters {
                                vertices: &vertices[..],
                                indices: &RENDER_RANGE.line_mesh_indices(),
                                obj_to_clp: &(camera.wld_to_clp * camera_resources.parameters.camera.clp_to_wld),
                                color: color::MAGENTA,
                            },
                        );
                    }

                    let cluster_range =
                        Range3::from_vector(cluster_resources.computed.dimensions.cast::<f64>().unwrap());

                    let vertices: Vec<[f32; 3]> = cluster_range
                        .vertices()
                        .iter()
                        .map(|point| point.cast().unwrap().into())
                        .collect();

                    if false {
                        self.line_renderer.render(
                            &mut rendering_context!(self),
                            &line_renderer::Parameters {
                                vertices: &vertices,
                                indices: &cluster_range.line_mesh_indices(),
                                obj_to_clp: &(camera.wld_to_clp/* * cluster_resources.computed.clu_clp_to_wld */),
                                color: color::RED,
                            },
                        );
                    }

                    let clu_cam_to_ren_clp = &(camera.wld_to_clp
                        * self.cluster_resources_pool[cluster_resources_index]
                            .computed
                            .clu_cam_to_wld);
                    self.render_debug_clusters(&cluster_renderer::Parameters {
                        cluster_resources_index,
                        clu_cam_to_ren_clp,
                    });
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
                 {}\
                 Render Technique: {:<14} | CLS Grouping:     {:<14} | CLS Projection:   {:<14}\n\
                 Attenuation Mode: {:<14} | Lighting Space:   {:<14} | Light Count:      {:<14}\n\
                 ",
                match self.configuration.global.mode {
                    ApplicationMode::Normal => "".to_string(),
                    ApplicationMode::Record => format!("Frame {:>4} | Recording...\n", self.frame_index.to_usize()),
                    ApplicationMode::Replay => format!(
                        "Frame {:>4}/{} | Run {:>2}/{}\n",
                        self.frame_index.to_usize(),
                        self.replay_frame_events.as_ref().map(Vec::len).unwrap(),
                        self.profiling_context.run_index().to_usize() + 1,
                        self.configuration.replay.run_count,
                    ),
                },
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

        if self.configuration.profiling.display {
            if let Some(stats) = self.profiling_context.stats(self.frame_sample_index) {
                overlay_textbox.write(
                    &monospace,
                    &format!(
                        "[{:>3}] {:<29} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
                        self.frame_sample_index.to_usize(),
                        "frame",
                        stats.cpu_elapsed_min as f64 / 1000.0,
                        stats.cpu_elapsed_avg as f64 / 1000.0,
                        stats.cpu_elapsed_max as f64 / 1000.0,
                        stats.gpu_elapsed_min as f64 / 1000.0,
                        stats.gpu_elapsed_avg as f64 / 1000.0,
                        stats.gpu_elapsed_max as f64 / 1000.0,
                    ),
                );
            }

            // NOTE: Not that useful always.
            // for draw_resources in self.resources.draw_resources_pool.iter() {
            //     for &(sample_index, sample_name) in &[
            //         (draw_resources.compute_instance_matrices_profiler, "instance matrices"),
            //         (draw_resources.compute_draw_commands_profiler, "draw commands"),
            //     ] {
            //         if let Some(stats) = self.profiling_context.stats(sample_index) {
            //             overlay_textbox.write(
            //                     &monospace,
            //                     &format!(
            //                         "[{:>3}] {:<29} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
            //                         sample_index.to_usize(),
            //                         sample_name,
            //                         stats.cpu_elapsed_min as f64 / 1000.0,
            //                         stats.cpu_elapsed_avg as f64 / 1000.0,
            //                         stats.cpu_elapsed_max as f64 / 1000.0,
            //                         stats.gpu_elapsed_min as f64 / 1000.0,
            //                         stats.gpu_elapsed_avg as f64 / 1000.0,
            //                         stats.gpu_elapsed_max as f64 / 1000.0,
            //                     ),
            //                 );
            //         }
            //     }
            // }

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
                        let sample_index = camera_resources.profilers[stage];
                        if let Some(stats) = self.profiling_context.stats(sample_index) {
                            overlay_textbox.write(
                                    &monospace,
                                    &format!(
                                        "[{}][{}] {:<28} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
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
                    let sample_index = res.profilers[stage];

                    if let Some(stats) = self.profiling_context.stats(sample_index) {
                        overlay_textbox.write(
                                &monospace,
                                &format!(
                                    "[{}]    {:<28} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
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
                for &(name, sample_index) in [
                    ("depth", main_resources.depth_pass_profiler),
                    ("basic", main_resources.basic_pass_profiler),
                ]
                .iter()
                {
                    if let Some(stats) = self.profiling_context.stats(sample_index) {
                        overlay_textbox.write(
                                &monospace,
                                &format!(
                                    "[{}]    {:<28} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
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
        }

        unsafe {
            let dimensions = Vector2::new(self.win_size.width as i32, self.win_size.height as i32);
            self.gl.viewport(0, 0, dimensions.x, dimensions.y);
            self.gl.bind_framebuffer(gl::FRAMEBUFFER, gl::FramebufferName::Default);

            self.render_text();
        }

        if self.profiling_context.run_index().to_usize() == 0 && self.configuration.profiling.record_frames {
            self.frame_downloader.record_frame(
                &self.paths.frames_dir,
                self.gl,
                self.frame_index,
                self.win_size.width as u32,
                self.win_size.height as u32,
                frame_downloader::Format::RGBA,
            );
        }

        self.profiling_context.stop(self.gl, profiler_index);

        self.profiling_context.end_frame(self.frame_index);

        self.gl_window.swap_buffers().unwrap();

        self.frame_index.increment();

        // TODO: Borrow the pool instead.
        self.camera_buffer_pool.reset(self.gl);

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

    fn clear_and_render_depth(&mut self, main_resources_index: MainResourcesIndex, draw_resources_index: usize) {
        let Self {
            ref gl,
            ref clear_color,
            ref mut main_resources_pool,
            ..
        } = *self;

        let main_resources = &mut main_resources_pool[main_resources_index];

        unsafe {
            gl.viewport(0, 0, main_resources.dimensions.x, main_resources.dimensions.y);
            gl.bind_framebuffer(gl::FRAMEBUFFER, main_resources.framebuffer_name);
            gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
            // Reverse-Z.
            gl.clear_depth(0.0);
            gl.clear(gl::ClearFlag::COLOR_BUFFER | gl::ClearFlag::DEPTH_BUFFER);
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
        }

        self.render_depth(draw_resources_index);
    }

    fn clear_and_render_main(&mut self, main_resources_index: MainResourcesIndex, main_parameters_index: usize) {
        let Self {
            ref gl,
            ref clear_color,
            ..
        } = *self;

        let main_resources = &mut self.main_resources_pool[main_resources_index];

        unsafe {
            gl.viewport(0, 0, main_resources.dimensions.x, main_resources.dimensions.y);
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
            let profiler_index = self
                .profiling_context
                .start(gl, self.main_resources_pool[main_resources_index].depth_pass_profiler);

            self.render_depth(self.main_parameters_vec[main_parameters_index].draw_resources_index);

            let Self { ref gl, .. } = *self;

            unsafe {
                gl.depth_func(gl::GEQUAL);
                gl.depth_mask(gl::FALSE);
            }

            self.profiling_context.stop(gl, profiler_index);
        }

        {
            // Reborrow
            let gl = &self.gl;
            let main_resources = &mut self.main_resources_pool[main_resources_index];
            let profiler_index = self.profiling_context.start(gl, main_resources.basic_pass_profiler);

            self.render_main(&basic_renderer::Parameters {
                mode: match self.target_camera_key {
                    CameraKey::Main => 0,
                    CameraKey::Debug => self.display_mode,
                },
                main_parameters_index,
            });

            // Reborrow.
            let gl = &self.gl;

            if self.depth_prepass {
                unsafe {
                    gl.depth_func(gl::GREATER);
                    gl.depth_mask(gl::TRUE);
                }
            }

            self.profiling_context.stop(gl, profiler_index);
        }
    }
}

fn main() {
    env_logger::init();

    let mut context = MainContext::new();

    let mut run_index = RunIndex::from_usize(0);
    let run_count = RunIndex::from_usize(match context.configuration.global.mode {
        ApplicationMode::Normal | ApplicationMode::Record => 1,
        ApplicationMode::Replay => context.configuration.replay.run_count,
    });

    while run_index < run_count {
        let mut context = Context::new(&mut context);

        context.profiling_context.begin_run(run_index);

        context.shader_compiler.replace_profiling(
            &mut context.current,
            shader_compiler::ProfilingVariables {
                time_sensitive: context.profiling_context.time_sensitive(),
            },
        );

        while context.running {
            context.render();
            context.process_events();
            context.simulate();
        }

        context.profiling_context.end_run(run_index);

        match context.configuration.global.mode {
            ApplicationMode::Normal | ApplicationMode::Record => {
                // Save state.
                let mut file = io::BufWriter::new(fs::File::create("state.bin").unwrap());
                for key in CameraKey::iter() {
                    let camera = context.cameras[key].current_to_camera();
                    file.write_all(camera.value_as_bytes()).unwrap();
                }
                break;
            }
            ApplicationMode::Replay => {
                // Do nothing.
            }
        }

        run_index.increment();
    }
}

// FIXME: Use.
#[allow(unused)]
fn gen_texture_t(name: gl::TextureName) -> vr::sys::Texture_t {
    // NOTE(mickvangelderen): The handle is not actually a pointer in
    // OpenGL's case, it's just the texture name.
    vr::sys::Texture_t {
        handle: name.to_u32() as usize as *const c_void as *mut c_void,
        eType: vr::sys::ETextureType_TextureType_OpenGL,
        eColorSpace: vr::sys::EColorSpace_ColorSpace_Gamma, // TODO(mickvangelderen): IDK
    }
}

fn mono_frustum(camera: &camera::Camera, dimensions: Vector2<i32>) -> Frustum<f64> {
    let dy = Rad::tan(Rad(Rad::from(camera.transform.fovy).0 as f64) / 2.0);
    let dx = dy * dimensions.x as f64 / dimensions.y as f64;
    Frustum {
        x0: -dx,
        x1: dx,
        y0: -dy,
        y1: dy,
        z0: camera.properties.z0 as f64,
        z1: camera.properties.z1 as f64,
    }
}

fn stereo_frustum(camera_properties: &camera::CameraProperties, tangents: vr::RawProjection) -> Frustum<f64> {
    let vr::RawProjection { l, r, b, t } = tangents;
    Frustum {
        x0: l as f64,
        x1: r as f64,
        y0: b as f64,
        y1: t as f64,
        z0: camera_properties.z0 as f64,
        z1: camera_properties.z1 as f64,
    }
}
