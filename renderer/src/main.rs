#![feature(euclidean_division)]
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
mod bounding_box;
pub mod camera;
mod cgmath_ext;
pub mod clamp;
mod cls;
mod cls_renderer;
mod cluster_renderer;
mod cluster_shading;
mod configuration;
mod convert;
mod depth_renderer;
mod filters;
pub mod frustrum;
mod gl_ext;
mod glutin_ext;
mod keyboard;
mod light;
mod line_renderer;
mod math;
mod overlay_renderer;
mod profiling;
mod rain;
mod rendering;
mod resources;
mod shader_compiler;
mod text_renderer;
mod text_rendering;
mod viewport;
mod window_mode;
mod world;

use crate::bounding_box::*;
use crate::cgmath_ext::*;
use crate::cluster_shading::*;
use crate::frustrum::*;
use crate::gl_ext::*;
use crate::math::{CeilToMultiple, DivCeil};
use crate::profiling::*;
use crate::shader_compiler::{EntryPoint, ShaderCompiler};
use crate::text_rendering::{FontContext, TextBox};
use crate::rendering::*;
use crate::resources::Resources;
use crate::viewport::*;
use crate::window_mode::*;
use crate::world::*;
// use arrayvec::ArrayVec;
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

pub struct MainResources {
    pub dims: Vector2<i32>,
    // Main frame resources.
    pub framebuffer_name: gl::NonDefaultFramebufferName,
    pub color_texture: Texture<gl::TEXTURE_2D, gl::RGBA16F>,
    pub depth_texture: Texture<gl::TEXTURE_2D, gl::DEPTH24_STENCIL8>,
    pub nor_in_cam_texture: Texture<gl::TEXTURE_2D, gl::R11F_G11F_B10F>,
    // Profiling
    pub depth_pass_profiler: Profiler,
    pub basic_pass_profiler: Profiler,
}

impl MainResources {
    pub fn new(gl: &gl::Gl, dims: Vector2<i32>) -> Self {
        unsafe {
            // Textures.
            let texture_update = TextureUpdate::new()
                .data(dims.x, dims.y, None)
                .min_filter(gl::NEAREST.into())
                .mag_filter(gl::NEAREST.into())
                .max_level(0)
                .wrap_s(gl::CLAMP_TO_EDGE.into())
                .wrap_t(gl::CLAMP_TO_EDGE.into());

            let color_texture = Texture::new(gl, gl::TEXTURE_2D, gl::RGBA16F);
            color_texture.update(gl, texture_update);

            let nor_in_cam_texture = Texture::new(gl, gl::TEXTURE_2D, gl::R11F_G11F_B10F);
            nor_in_cam_texture.update(gl, texture_update);

            let depth_texture = Texture::new(gl, gl::TEXTURE_2D, gl::DEPTH24_STENCIL8);
            depth_texture.update(gl, texture_update);

            // Framebuffers.

            let framebuffer_name = create_framebuffer!(
                gl,
                (gl::DEPTH_STENCIL_ATTACHMENT, depth_texture.name()),
                (gl::COLOR_ATTACHMENT0, color_texture.name()),
                (gl::COLOR_ATTACHMENT1, nor_in_cam_texture.name()),
            );

            // Uniform block buffers,

            MainResources {
                dims,
                framebuffer_name,
                color_texture,
                depth_texture,
                nor_in_cam_texture,
                depth_pass_profiler: Profiler::new(&gl),
                basic_pass_profiler: Profiler::new(&gl),
            }
        }
    }

    pub fn resize(&mut self, gl: &gl::Gl, dims: Vector2<i32>) {
        if self.dims != dims {
            self.dims = dims;

            let texture_update = TextureUpdate::new().data(dims.x, dims.y, None);
            self.color_texture.update(gl, texture_update);
            self.depth_texture.update(gl, texture_update);
            self.nor_in_cam_texture.update(gl, texture_update);
        }
    }

    pub fn drop(self, gl: &gl::Gl) {
        unsafe {
            gl.delete_framebuffer(self.framebuffer_name);
            self.color_texture.drop(gl);
            self.depth_texture.drop(gl);
            self.nor_in_cam_texture.drop(gl);
        }
    }
}

#[derive(Debug, Default)]
pub struct MainData {
}

pub struct MainPool {
    pub resources: Vec<MainResources>,
    pub data: Vec<MainData>,
}

impl MainPool {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn reserve(&mut self, gl: &gl::Gl, dims: Vector2<i32>) -> usize {
        let index = self.data.len();
        self.data.push(MainData::default());

        if self.resources.len() < index + 1 {
            self.resources.push(MainResources::new(&gl, dims));
        }

        let resources = &mut self.resources[index];
        resources.resize(&gl, dims);

        index
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

const DEPTH_RANGE: (f64, f64) = (1.0, 0.0);

fn main() {
    env_logger::init();

    let current_dir = std::env::current_dir().unwrap();
    let resource_dir: PathBuf = [current_dir.as_ref(), Path::new("resources")].into_iter().collect();
    let configuration_path = resource_dir.join(configuration::FILE_PATH);

    let (fs_tx, fs_rx) = mpsc::channel();

    let mut watcher = notify::watcher(fs_tx, time::Duration::from_millis(100)).unwrap();

    notify::Watcher::watch(&mut watcher, &resource_dir, notify::RecursiveMode::Recursive).unwrap();

    let mut configuration: configuration::Root = configuration::read(&configuration_path);

    let mut events_loop = glutin::EventsLoop::new();

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

    let mut world = {
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

        let mut current = ::incremental::Current::new();

        let win_dpi = gl_window.get_hidpi_factor();
        let win_size = gl_window.get_inner_size().unwrap().to_physical(win_dpi);

        let shader_compiler = ShaderCompiler::new(
            &current,
            shader_compiler::Variables {
                light_space: LightSpace::Wld,
                render_technique: RenderTechnique::Clustered,
                attenuation_mode: AttenuationMode::Interpolated,
                prefix_sum: configuration.prefix_sum,
                clustered_light_shading: configuration.clustered_light_shading,
            },
        );

        World {
            epoch: Instant::now(),
            running: true,
            focus: false,
            win_dpi,
            win_size,
            keyboard_state: KeyboardState::default(),
            resource_dir,
            configuration_path,
            tick: 0,
            frame: 0,
            paused: false,
            clear_color: [0.0, 0.0, 0.0],
            window_mode: WindowMode::Main,
            display_mode: 1,
            depth_prepass: true,
            gl_log_regex: RegexBuilder::new(r"^\d+").multi_line(true).build().unwrap(),
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
            current,
            rain_drops: Vec::new(),
            shader_compiler,
        }
    };

    unsafe { gl_window.make_current().unwrap() };

    let gl = unsafe { gl::Gl::load_with(|s| gl_window.context().get_proc_address(s) as *const _) };

    unsafe {
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
    }

    let mut sans_serif = FontContext::new(&gl, world.resource_dir.join("fonts/OpenSans-Regular.fnt"));

    let mut monospace = FontContext::new(&gl, world.resource_dir.join("fonts/RobotoMono-Regular.fnt"));

    let mut depth_renderer = depth_renderer::Renderer::new(&gl, &mut world);
    let mut line_renderer = line_renderer::Renderer::new(&gl, &mut world);
    let mut basic_renderer = basic_renderer::Renderer::new(&gl, &mut world);
    let mut overlay_renderer = overlay_renderer::Renderer::new(&gl, &mut world);
    let mut cluster_renderer = cluster_renderer::Renderer::new(&gl, &mut world);
    let mut text_renderer = text_renderer::Renderer::new(&gl, &mut world);
    let mut cls_renderer = cls_renderer::Renderer::new(&gl, &mut world);
    let mut count_lights_program = cls::count_lights::CountLightsProgram::new(&gl, &mut world);
    let mut assign_lights_program = cls::assign_lights::AssignLightsProgram::new(&gl, &mut world);

    let resources = resources::Resources::new(&gl, &world.resource_dir, &configuration);

    let vr_context = vr::Context::new(vr::ApplicationType::Scene)
        .map_err(|error| {
            eprintln!("Failed to acquire context: {:?}", error);
        })
        .ok();

    let mut fps_average = filters::MovingAverageF32::new(0.0);
    let mut last_frame_start = time::Instant::now();

    let mut camera_buffer_pool = BufferPool::new();

    // TODO: Re-use
    let mut light_resources_vec: Vec<light::LightResources> = Vec::new();
    let mut light_params_vec: Vec<light::LightParameters> = Vec::new();

    let mut cluster_resources_vec: Vec<ClusterResources> = Vec::new();
    let mut cluster_data_vec: Vec<ClusterData> = Vec::new();

    let mut main_pool = MainPool::new();

    let mut point_lights = Vec::new();

    let mut overlay_textbox = TextBox::new(
        13,
        10,
        world.win_size.width as i32 - 26,
        world.win_size.height as i32 - 20,
    );

    while world.running {
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

        let stereo_data = vr_context
            .as_ref()
            .map(|vr_context| {
                let mut poses: [vr::sys::TrackedDevicePose_t; vr::sys::k_unMaxTrackedDeviceCount as usize] =
                    unsafe { mem::zeroed() };
                // NOTE: OpenVR will block upon querying the pose for as long as
                // possible but no longer than it takes to submit the new frame. This is
                // done to render the most up-to-date application state as possible.
                vr_context.compositor().wait_get_poses(&mut poses[..], None).unwrap();

                let win_size = vr_context.system().get_recommended_render_target_size();

                let hmd_pose = poses[vr::sys::k_unTrackedDeviceIndex_Hmd as usize];
                assert!(hmd_pose.bPoseIsValid, "Received invalid pose from VR.");
                let hmd_to_bdy = Matrix4::from_hmd(hmd_pose.mDeviceToAbsoluteTracking.m).cast().unwrap();

                StereoData {
                    win_size: Vector2::new(win_size.width, win_size.height).cast().unwrap(),
                    hmd_to_bdy,
                    eyes: EyeMap::new(|eye_key| {
                        let eye = Eye::from(eye_key);
                        let cam_to_hmd = Matrix4::from_hmd(vr_context.system().get_eye_to_head_transform(eye))
                            .cast()
                            .unwrap();
                        EyeData {
                            tangents: vr_context.system().get_projection_raw(eye),
                            cam_to_hmd: cam_to_hmd,
                        }
                    }),
                }
            })
            .or_else(|| {
                if configuration.virtual_stereo.enabled {
                    let win_size = Vector2::new(world.win_size.width / 2.0, world.win_size.height)
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
            });

        process_fs_events(&fs_rx, &gl, &mut world, &mut configuration);
        process_window_events(&mut events_loop, &vr_context, &mut configuration, &mut world);
        process_vr_events(&vr_context);

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

        light_params_vec.clear();
        cluster_data_vec.clear();
        main_pool.clear();

        {
            point_lights.clear();

            for &point_light in resources.point_lights.iter() {
                point_lights.push(point_light);
            }

            for rain_drop in world.rain_drops.iter() {
                point_lights.push(light::PointLight {
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

        if world.shader_compiler.light_space() == LightSpace::Wld {
            light_params_vec.push(light::LightParameters {
                wld_to_lgt: Matrix4::identity(),
                lgt_to_wld: Matrix4::identity(),
            });
        }

        if world.shader_compiler.render_technique() == RenderTechnique::Clustered {
            let cluster_camera = &world.cameras.main;
            let bdy_to_wld = cluster_camera.current_transform.pos_to_parent().cast::<f64>().unwrap();
            let wld_to_bdy = cluster_camera
                .current_transform
                .pos_from_parent()
                .cast::<f64>()
                .unwrap();

            let cluster_resources_index = cluster_data_vec.len();

            if cluster_resources_vec.len() < cluster_resources_index + 1 {
                cluster_resources_vec.push(ClusterResources::new(&gl, &configuration.clustered_light_shading));
                debug_assert_eq!(cluster_resources_vec.len(), cluster_resources_index + 1);
            }

            let cluster_resources = &mut cluster_resources_vec[cluster_resources_index];

            cluster_resources.clear();
            let hmd_to_wld;
            let wld_to_hmd;

            match stereo_data.as_ref() {
                Some(&StereoData {
                    hmd_to_bdy,
                    eyes,
                    win_size,
                }) => {
                    hmd_to_wld = bdy_to_wld * hmd_to_bdy;
                    wld_to_hmd = hmd_to_wld.invert().unwrap();

                    for &eye in EYE_KEYS.iter() {
                        let EyeData { tangents, cam_to_hmd } = eyes[eye];

                        let cam_to_wld = cam_to_hmd * hmd_to_wld;
                        let wld_to_cam = cam_to_wld.invert().unwrap();

                        let frustrum = stereo_frustrum(&cluster_camera.properties, tangents);
                        let cam_to_clp = frustrum.perspective(DEPTH_RANGE).cast::<f64>().unwrap();
                        let clp_to_cam = cam_to_clp.invert().unwrap();

                        let clp_to_hmd = cam_to_hmd * clp_to_cam;
                        let hmd_to_clp = clp_to_hmd.invert().unwrap();

                        cluster_resources.add_camera(
                            &gl,
                            ClusterCameraData {
                                frame_dims: win_size,

                                wld_to_cam,
                                cam_to_wld,

                                cam_to_clp,
                                clp_to_cam,

                                hmd_to_clp,
                                clp_to_hmd,

                                wld_to_hmd,
                                hmd_to_wld,
                            },
                        );
                    }
                }
                None => {
                    // hmd_to_bdy = bdy_to_hmd = I
                    hmd_to_wld = bdy_to_wld;
                    wld_to_hmd = wld_to_bdy;

                    let frame_dims = Vector2::new(world.win_size.width as i32, world.win_size.height as i32);
                    let frustrum = mono_frustrum(&cluster_camera.current_to_camera(), frame_dims);
                    let cam_to_clp = frustrum.perspective(DEPTH_RANGE).cast::<f64>().unwrap();
                    let clp_to_cam = cam_to_clp.invert().unwrap();

                    // cam_to_hmd = hmd_to_cam = I
                    let clp_to_hmd = clp_to_cam;
                    let hmd_to_clp = cam_to_clp;

                    cluster_resources.add_camera(
                        &gl,
                        ClusterCameraData {
                            frame_dims,

                            wld_to_cam: wld_to_bdy,
                            cam_to_wld: bdy_to_wld,

                            cam_to_clp,
                            clp_to_cam,

                            hmd_to_clp,
                            clp_to_hmd,

                            wld_to_hmd,
                            hmd_to_wld,
                        },
                    );
                }
            }

            let cluster_data = ClusterData::new(
                &configuration.clustered_light_shading,
                cluster_resources
                    .camera_data
                    .iter()
                    .map(|&ClusterCameraData { clp_to_hmd, .. }| clp_to_hmd),
                wld_to_hmd,
                hmd_to_wld,
            );

            let cluster_count = cluster_data.cluster_count();
            let blocks_per_dispatch =
                cluster_count.div_ceil(configuration.prefix_sum.pass_0_threads * configuration.prefix_sum.pass_1_threads);
            let clusters_per_dispatch = configuration.prefix_sum.pass_0_threads * blocks_per_dispatch;
            let cluster_dispatch_count = cluster_count.div_ceil(clusters_per_dispatch);

            unsafe {
                let buffer = &mut cluster_resources.cluster_fragment_counts_buffer;
                let byte_count = std::mem::size_of::<u32>() * cluster_data.cluster_count() as usize;
                buffer.invalidate(&gl);
                // buffer.ensure_capacity(&gl, byte_count);
                buffer.clear_0u32(&gl, byte_count);
            }

            for (camera_index, camera) in cluster_resources.camera_data.iter_mut().enumerate() {
                let camera_resources = &mut cluster_resources.camera_res[camera_index];
                let main_index = main_pool.reserve(&gl, camera.frame_dims);
                let main_resources = &mut main_pool.resources[main_index];

                {
                    let profiler = &mut camera_resources.profilers.render_depth;
                    profiler.start(&gl, world.frame, world.epoch);

                    unsafe {
                        let camera_buffer = CameraBuffer {
                            wld_to_cam: camera.wld_to_cam.cast().unwrap(),
                            cam_to_wld: camera.cam_to_wld.cast().unwrap(),

                            cam_to_clp: camera.cam_to_clp.cast().unwrap(),
                            clp_to_cam: camera.clp_to_cam.cast().unwrap(),

                            // NOTE: Doesn't matter for depth pass!
                            cam_pos_in_lgt: Vector4::zero(),
                        };

                        let buffer_index = camera_buffer_pool.unused(&gl);
                        let buffer_name = camera_buffer_pool[buffer_index];

                        gl.named_buffer_data(buffer_name, camera_buffer.value_as_bytes(), gl::STREAM_DRAW);
                        gl.bind_buffer_base(gl::UNIFORM_BUFFER, rendering::CAMERA_BUFFER_BINDING, buffer_name);
                    }

                    render_depth(&gl, main_resources, &resources, &mut world, &mut depth_renderer);

                    profiler.stop(&gl, world.frame, world.epoch);
                }

                {
                    let profiler = &mut camera_resources.profilers.count_frags;
                    profiler.start(&gl, world.frame, world.epoch);

                    unsafe {
                        // gl.bind_framebuffer(gl::FRAMEBUFFER, gl::FramebufferName::Default);
                        let program = &mut cls_renderer.fragments_per_cluster_program;
                        program.update(&gl, &mut world);
                        if let ProgramName::Linked(name) = program.name {
                            gl.use_program(name);

                            gl.bind_buffer_base(
                                gl::SHADER_STORAGE_BUFFER,
                                cls_renderer::CLUSTER_FRAGMENT_COUNTS_BINDING,
                                cluster_resources.cluster_fragment_counts_buffer.name(),
                            );

                            // gl.uniform_1i(cls_renderer::DEPTH_SAMPLER_LOC, 0);
                            gl.bind_texture_unit(0, main_resources.depth_texture.name());

                            gl.uniform_2f(
                                cls_renderer::FB_DIMS_LOC,
                                main_resources.dims.cast::<f32>().unwrap().into(),
                            );

                            let clp_to_cls = (cluster_data.wld_to_cls * camera.cam_to_wld * camera.clp_to_cam)
                                .cast::<f32>()
                                .unwrap();

                            gl.uniform_matrix4f(
                                cls_renderer::CLP_TO_CLS_LOC,
                                gl::MajorAxis::Column,
                                clp_to_cls.as_ref(),
                            );

                            gl.uniform_3ui(cls_renderer::CLUSTER_DIMS_LOC, cluster_data.dimensions.into());

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
                    profiler.stop(&gl, world.frame, world.epoch);
                }
            }

            // We have our fragments per cluster buffer here.

            {
                let profiler = &mut cluster_resources.profilers.compact_clusters;
                profiler.start(&gl, world.frame, world.epoch);

                unsafe {
                    let buffer = &mut cluster_resources.offset_buffer;
                    let byte_count = std::mem::size_of::<u32>() * configuration.prefix_sum.pass_1_threads as usize;
                    buffer.invalidate(&gl);
                    buffer.ensure_capacity(&gl, byte_count);
                    buffer.clear_0u32(&gl, byte_count);
                    gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, cls_renderer::OFFSET_BINDING, buffer.name());
                }

                unsafe {
                    let buffer = &mut cluster_resources.active_cluster_indices_buffer;
                    buffer.invalidate(&gl);
                    // buffer.ensure_capacity(&gl, byte_count);
                    buffer.clear_0u32(&gl, buffer.byte_capacity());
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::ACTIVE_CLUSTER_INDICES_BINDING,
                        buffer.name(),
                    );
                }

                unsafe {
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::DRAW_COMMAND_BINDING,
                        cluster_resources.draw_command_buffer.name(),
                    );
                }

                unsafe {
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::COMPUTE_COMMAND_BINDING,
                        cluster_resources.compute_commands_buffer.name(),
                    );
                }

                unsafe {
                    let program = &mut cls_renderer.compact_clusters_0_program;
                    program.update(&gl, &mut world);
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.uniform_1ui(cls_renderer::ITEM_COUNT_LOC, cluster_data.cluster_count());
                        gl.dispatch_compute(cluster_dispatch_count, 1, 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }

                unsafe {
                    let program = &mut cls_renderer.compact_clusters_1_program;
                    program.update(&gl, &mut world);
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.uniform_1ui(cls_renderer::ITEM_COUNT_LOC, cluster_data.cluster_count());
                        gl.dispatch_compute(1, 1, 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }

                unsafe {
                    let program = &mut cls_renderer.compact_clusters_2_program;
                    program.update(&gl, &mut world);
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.uniform_1ui(cls_renderer::ITEM_COUNT_LOC, cluster_data.cluster_count());
                        gl.dispatch_compute(cluster_dispatch_count, 1, 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }
                profiler.stop(&gl, world.frame, world.epoch);
            }

            // We have our active clusters.

            {
                let profiler = &mut cluster_resources.profilers.upload_lights;
                profiler.start(&gl, world.frame, world.epoch);

                unsafe {
                    let data: Vec<[f32; 4]> = point_lights
                        .iter()
                        .map(|&light| {
                            let pos_in_hmd = wld_to_hmd.transform_point(light.pos_in_wld.cast().unwrap());
                            let [x, y, z]: [f32; 3] = pos_in_hmd.cast::<f32>().unwrap().into();
                            [x, y, z, light.attenuation.clip_far]
                        })
                        .collect();
                    let bytes = data.vec_as_bytes();
                    let padded_byte_count = bytes.len().ceil_to_multiple(64);

                    let buffer = &mut cluster_resources.light_xyzr_buffer;
                    buffer.invalidate(&gl);
                    buffer.ensure_capacity(&gl, padded_byte_count);
                    buffer.write(&gl, bytes);
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::LIGHT_XYZR_BINDING,
                        buffer.name(),
                    );
                }

                profiler.stop(&gl, world.frame, world.epoch);
            }

            {
                let profiler = &mut cluster_resources.profilers.count_lights;
                profiler.start(&gl, world.frame, world.epoch);

                unsafe {
                    let buffer = &mut cluster_resources.active_cluster_light_counts_buffer;
                    buffer.invalidate(&gl);
                    // buffer.ensure_capacity(&gl, byte_count);
                    buffer.clear_0u32(&gl, buffer.byte_capacity());
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::ACTIVE_CLUSTER_LIGHT_COUNTS_BINDING,
                        buffer.name(),
                    );
                }

                unsafe {
                    let program = &mut count_lights_program.program;
                    program.update(&gl, &mut world);
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.uniform_3ui(cls::count_lights::CLUSTER_DIMS_LOC, cluster_data.dimensions.into());
                        gl.uniform_3f(
                            cls::count_lights::SCALE_LOC,
                            cluster_data.scale_from_cls_to_hmd.cast().unwrap().into(),
                        );
                        gl.uniform_3f(
                            cls::count_lights::TRANSLATION_LOC,
                            cluster_data.trans_from_cls_to_hmd.cast().unwrap().into(),
                        );
                        gl.uniform_1ui(cls::count_lights::LIGHT_COUNT_LOC, point_lights.len() as u32);
                        gl.bind_buffer(
                            gl::DISPATCH_INDIRECT_BUFFER,
                            cluster_resources.compute_commands_buffer.name(),
                        );
                        gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE | gl::MemoryBarrierFlag::COMMAND);
                        gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 0);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }
                profiler.stop(&gl, world.frame, world.epoch);
            }

            // We have our light counts.

            {
                let profiler = &mut cluster_resources.profilers.light_offsets;
                profiler.start(&gl, world.frame, world.epoch);

                unsafe {
                    let buffer = &mut cluster_resources.offset_buffer;
                    let byte_count = std::mem::size_of::<u32>() * configuration.prefix_sum.pass_1_threads as usize;
                    buffer.invalidate(&gl);
                    buffer.ensure_capacity(&gl, byte_count);
                    buffer.clear_0u32(&gl, byte_count);
                    gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, cls_renderer::OFFSET_BINDING, buffer.name());
                }

                unsafe {
                    let buffer = &mut cluster_resources.active_cluster_light_offsets_buffer;
                    buffer.invalidate(&gl);
                    // buffer.ensure_capacity(&gl, byte_count);
                    buffer.clear_0u32(&gl, buffer.byte_capacity());
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::ACTIVE_CLUSTER_LIGHT_OFFSETS_BINDING,
                        buffer.name(),
                    );
                    gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE);
                }

                unsafe {
                    let program = &mut cls_renderer.compact_light_counts_0_program;
                    program.update(&gl, &mut world);
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }

                unsafe {
                    let program = &mut cls_renderer.compact_light_counts_1_program;
                    program.update(&gl, &mut world);
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.dispatch_compute(1, 1, 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }

                unsafe {
                    let program = &mut cls_renderer.compact_light_counts_2_program;
                    program.update(&gl, &mut world);
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 1);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }
                profiler.stop(&gl, world.frame, world.epoch);
            }

            // We have our light offsets.

            {
                let profiler = &mut cluster_resources.profilers.assign_lights;
                profiler.start(&gl, world.frame, world.epoch);

                unsafe {
                    let buffer = &mut cluster_resources.light_indices_buffer;
                    buffer.invalidate(&gl);
                    // buffer.ensure_capacity(&gl, byte_count);
                    buffer.clear_0u32(&gl, buffer.byte_capacity());
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        cls_renderer::LIGHT_INDICES_BINDING,
                        buffer.name(),
                    );
                }

                unsafe {
                    let program = &mut assign_lights_program.program;
                    program.update(&gl, &mut world);
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);
                        gl.uniform_3ui(cls::assign_lights::CLUSTER_DIMS_LOC, cluster_data.dimensions.into());
                        gl.uniform_3f(
                            cls::assign_lights::SCALE_LOC,
                            cluster_data.scale_from_cls_to_hmd.cast().unwrap().into(),
                        );
                        gl.uniform_3f(
                            cls::assign_lights::TRANSLATION_LOC,
                            cluster_data.trans_from_cls_to_hmd.cast().unwrap().into(),
                        );
                        gl.uniform_1ui(cls::assign_lights::LIGHT_COUNT_LOC, point_lights.len() as u32);
                        gl.bind_buffer(
                            gl::DISPATCH_INDIRECT_BUFFER,
                            cluster_resources.compute_commands_buffer.name(),
                        );
                        gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE | gl::MemoryBarrierFlag::COMMAND);
                        gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 0);
                        gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                    }
                }
                profiler.stop(&gl, world.frame, world.epoch);
            }

            cluster_data_vec.push(cluster_data);
        }

        struct MainParameters {
            pub wld_to_cam: Matrix4<f64>,
            pub cam_to_wld: Matrix4<f64>,

            pub cam_to_clp: Matrix4<f64>,
            pub clp_to_cam: Matrix4<f64>,

            pub cam_pos_in_wld: Point3<f64>,

            pub light_index: usize,

            pub dimensions: Vector2<i32>,
            pub display_viewport: Viewport<i32>,
        }

        let mut main_parameters = Vec::new();

        match stereo_data {
            Some(StereoData {
                win_size,
                hmd_to_bdy,
                eyes,
            }) => {
                let render_camera = world.transition_camera.current_camera;
                let bdy_to_wld = render_camera.transform.pos_to_parent().cast::<f64>().unwrap();

                let hmd_to_wld = bdy_to_wld * hmd_to_bdy;
                let wld_to_hmd = hmd_to_wld.invert().unwrap();

                let cam_pos_in_wld = render_camera.transform.position.cast::<f64>().unwrap();

                if world.shader_compiler.light_space() == LightSpace::Hmd {
                    light_params_vec.push(light::LightParameters {
                        wld_to_lgt: wld_to_hmd,
                        lgt_to_wld: hmd_to_wld,
                    });
                }

                for &eye_key in EYE_KEYS.iter() {
                    let EyeData { tangents, cam_to_hmd } = eyes[eye_key];

                    let cam_to_wld = hmd_to_wld * cam_to_hmd;
                    let wld_to_cam = cam_to_wld.invert().unwrap();

                    let frustrum = stereo_frustrum(&render_camera.properties, tangents);
                    let cam_to_clp = frustrum.perspective(DEPTH_RANGE).cast::<f64>().unwrap();
                    let clp_to_cam = cam_to_clp.invert().unwrap();

                    if world.shader_compiler.light_space() == LightSpace::Cam {
                        light_params_vec.push(light::LightParameters {
                            wld_to_lgt: wld_to_cam,
                            lgt_to_wld: cam_to_wld,
                        });
                    }

                    main_parameters.push(MainParameters {
                        wld_to_cam,
                        cam_to_wld,

                        cam_to_clp,
                        clp_to_cam,

                        cam_pos_in_wld,

                        light_index: light_params_vec.len() - 1,

                        dimensions: win_size,
                        display_viewport: {
                            let w = world.win_size.width as i32;
                            let h = world.win_size.height as i32;

                            match eye_key {
                                vr::Eye::Left => Viewport::from_coordinates(Point2::new(0, 0), Point2::new(w / 2, h)),
                                vr::Eye::Right => Viewport::from_coordinates(Point2::new(w / 2, 0), Point2::new(w, h)),
                            }
                        },
                    });
                }
            }
            None => {
                let dimensions = Vector2::new(world.win_size.width as i32, world.win_size.height as i32);

                let render_camera = &world.transition_camera.current_camera;
                let cam_to_wld = render_camera.transform.pos_to_parent().cast::<f64>().unwrap();
                let wld_to_cam = render_camera.transform.pos_from_parent().cast::<f64>().unwrap();

                let frustrum = mono_frustrum(render_camera, dimensions);
                let cam_to_clp = frustrum.perspective(DEPTH_RANGE).cast::<f64>().unwrap();
                let clp_to_cam = cam_to_clp.invert().unwrap();

                if world.shader_compiler.light_space() == LightSpace::Hmd
                    || world.shader_compiler.light_space() == LightSpace::Cam
                {
                    light_params_vec.push(light::LightParameters {
                        wld_to_lgt: wld_to_cam,
                        lgt_to_wld: cam_to_wld,
                    });
                }

                let cam_pos_in_wld = render_camera.transform.position.cast::<f64>().unwrap();

                main_parameters.push(MainParameters {
                    wld_to_cam,
                    cam_to_wld,

                    cam_to_clp,
                    clp_to_cam,

                    cam_pos_in_wld,

                    light_index: light_params_vec.len() - 1,

                    dimensions,
                    display_viewport: Viewport::from_dimensions(dimensions),
                });
            }
        }

        for res in light_resources_vec.iter_mut() {
            res.dirty = true;
        }

        let mut bound_light_index = None;

        for main_params in main_parameters.iter() {
            let MainParameters {
                ref wld_to_cam,
                ref cam_to_wld,

                ref cam_to_clp,
                ref clp_to_cam,

                cam_pos_in_wld,

                light_index,

                dimensions,
                display_viewport,
            } = *main_params;

            let light_params = &light_params_vec[light_index];

            if bound_light_index != Some(light_index) {
                // Ensure light resources are available.
                while light_resources_vec.len() < light_index + 1 {
                    light_resources_vec.push(light::LightResources::new(&gl));
                }
                let light_resources = &mut light_resources_vec[light_index];

                // Ensure light resources are uploaded.
                if light_resources.dirty {
                    light_resources.lights.clear();
                    light_resources.lights.extend(point_lights.iter().map(|&point_light| {
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
                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        rendering::LIGHT_BUFFER_BINDING,
                        light_resources.buffer_name,
                    );
                    bound_light_index = Some(light_index);
                }
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

                let buffer_index = camera_buffer_pool.unused(&gl);
                let buffer_name = camera_buffer_pool[buffer_index];

                gl.named_buffer_data(buffer_name, camera_buffer.value_as_bytes(), gl::STREAM_DRAW);
                gl.bind_buffer_base(gl::UNIFORM_BUFFER, rendering::CAMERA_BUFFER_BINDING, buffer_name);
            }

            let main_index = main_pool.reserve(&gl, dimensions);
            let main_resources = &mut main_pool.resources[main_index];

            let cluster_parameters = if world.shader_compiler.variables.render_technique == RenderTechnique::Clustered {
                Some(basic_renderer::ClusterParameters {
                    // FIXME: Think harder about this.
                    data: &cluster_data_vec[0],
                    resources: &cluster_resources_vec[0],
                })
            } else {
                None
            };

            render_main(
                &gl,
                main_resources,
                &resources,
                &mut world,
                cluster_parameters,
                &mut depth_renderer,
                &mut basic_renderer,
            );

            if world.target_camera_key == CameraKey::Debug {
                let corners_in_clp = Frustrum::corners_in_clp(DEPTH_RANGE);
                let vertices: Vec<[f32; 3]> = corners_in_clp
                    .iter()
                    .map(|point| point.cast().unwrap().into())
                    .collect();

                for cluster_index in 0..cluster_data_vec.len() {
                    let cluster_data = &cluster_data_vec[cluster_index];
                    let cluster_resources = &cluster_resources_vec[cluster_index];

                    for camera in cluster_resources.camera_data.iter() {
                        line_renderer.render(
                            &gl,
                            &line_renderer::Parameters {
                                vertices: &vertices[..],
                                indices: &FRUSTRUM_LINE_MESH_INDICES[..],
                                obj_to_wld: &(camera.hmd_to_wld * camera.clp_to_hmd).cast().unwrap(),
                            },
                            &mut world,
                        );
                    }

                    cluster_renderer.render(
                        &gl,
                        &cluster_renderer::Parameters {
                            cluster_resources: &cluster_resources,
                            cluster_data: &cluster_data,
                            configuration: &configuration.clustered_light_shading,
                            cls_to_clp: (cam_to_clp * wld_to_cam * cluster_data.cls_to_wld).cast().unwrap(),
                        },
                        &mut world,
                        &resources,
                    );
                }
            }

            unsafe {
                gl.blit_named_framebuffer(
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
            let dimensions = Vector2::new(world.win_size.width as i32, world.win_size.height as i32);

            overlay_textbox.width = dimensions.x - 26;
            overlay_textbox.height = dimensions.y - 20;
            overlay_textbox.clear();
        }

        overlay_textbox.write(
            &monospace,
            &format!(
                "\
                 Attenuation Mode: {:?}\n\
                 Render Technique: {:?}\n\
                 Lighting Space:   {:?}\n\
                 Light Count:      {}\n\
                 ",
                world.shader_compiler.attenuation_mode(),
                world.shader_compiler.render_technique(),
                world.shader_compiler.light_space(),
                point_lights.len(),
            ),
        );

        for cluster_index in 0..cluster_data_vec.len() {
            let res = &mut cluster_resources_vec[cluster_index];
            let data = &cluster_data_vec[cluster_index];
            let dimensions_u32 = data.dimensions;

            overlay_textbox.write(
                &monospace,
                &format!(
                    "[{}] cluster dimensions {{ x: {:3}, y: {:3}, z: {:3} }}\n",
                    cluster_index, dimensions_u32.x, dimensions_u32.y, dimensions_u32.z,
                ),
            );

            for (camera_index, _camera) in res.camera_data.iter().enumerate() {
                let camera_resources = &mut res.camera_res[camera_index];
                for &stage in &CameraStage::VALUES {
                    let stats = &mut camera_resources.profilers[stage].stats(world.frame);
                    if let Some(stats) = stats {
                        overlay_textbox.write(
                            &monospace,
                            &format!(
                                "[{}][{}] {:<20} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
                                cluster_index,
                                camera_index,
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
                let stats = &mut res.profilers[stage].stats(world.frame);
                if let Some(stats) = stats {
                    overlay_textbox.write(
                        &monospace,
                        &format!(
                            "[{}]    {:<20} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
                            cluster_index,
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

        for (main_index, main_resources) in main_pool.resources.iter().enumerate() {
            for (name, profiler) in [
                ("depth", &main_resources.depth_pass_profiler),
                ("basic", &main_resources.basic_pass_profiler),
            ]
            .iter()
            {
                let stats = profiler.stats(world.frame);
                if let Some(stats) = stats {
                    overlay_textbox.write(
                        &monospace,
                        &format!(
                            "[{}]    {:<20} | CPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs | GPU {:>7.1}μs < {:>7.1}μs < {:>7.1}μs\n",
                            main_index,
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

        unsafe {
            let dimensions = Vector2::new(world.win_size.width as i32, world.win_size.height as i32);
            gl.viewport(0, 0, dimensions.x, dimensions.y);
            gl.bind_framebuffer(gl::FRAMEBUFFER, gl::FramebufferName::Default);

            text_renderer.render(&gl, &mut world, &monospace, &overlay_textbox);
        }

        gl_window.swap_buffers().unwrap();
        world.frame += 1;

        // TODO: Borrow the pool instead.
        camera_buffer_pool.reset(&gl);

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
                "VR Lab - {:?} - {:?} - {:02.1} FPS",
                world.target_camera_key,
                world.shader_compiler.render_technique(),
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

fn process_fs_events(
    fs_rx: &mpsc::Receiver<notify::DebouncedEvent>,
    gl: &gl::Gl,
    world: &mut World,
    configuration: &mut configuration::Root,
) {
    let mut configuration_update = false;

    for event in fs_rx.try_iter() {
        match event {
            notify::DebouncedEvent::NoticeWrite(path) => {
                info!(
                    "Noticed write to file {:?}",
                    path.strip_prefix(&world.resource_dir).unwrap().display()
                );

                if let Some(source_index) = world.shader_compiler.memory.source_index(&path) {
                    world
                        .shader_compiler
                        .source_mut(source_index)
                        .last_modified
                        .modify(&mut world.current);
                }

                if &path == &world.configuration_path {
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
        *configuration = configuration::read(&world.configuration_path);

        // Apply updates.
        world.cameras.main.properties = configuration.main_camera.into();
        world.cameras.debug.properties = configuration.debug_camera.into();
        for key in CameraKey::iter() {
            world.cameras[key].maximum_smoothness = configuration.camera.maximum_smoothness;
        }

        world
            .shader_compiler
            .replace_prefix_sum(&mut world.current, configuration.prefix_sum);

        unsafe {
            if configuration.global.framebuffer_srgb {
                gl.enable(gl::FRAMEBUFFER_SRGB);
            } else {
                gl.disable(gl::FRAMEBUFFER_SRGB);
            }
        }
    }
}

pub fn process_window_events(
    events_loop: &mut glutin::EventsLoop,
    vr_context: &Option<vr::Context>,
    configuration: &mut configuration::Root,
    world: &mut World,
) {
    let mut mouse_dx = 0.0;
    let mut mouse_dy = 0.0;
    let mut mouse_dscroll = 0.0;
    let mut new_target_camera_key = world.target_camera_key;
    let mut new_window_mode = world.window_mode;
    let mut new_light_space = world.shader_compiler.light_space();
    let mut new_attenuation_mode = world.shader_compiler.attenuation_mode();
    let mut new_render_technique = world.shader_compiler.render_technique();
    let mut reset_debug_camera = false;

    events_loop.poll_events(|event| {
        use glutin::Event;
        match event {
            Event::WindowEvent { event, .. } => {
                use glutin::WindowEvent;
                match event {
                    WindowEvent::CloseRequested => world.running = false,
                    WindowEvent::HiDpiFactorChanged(val) => {
                        let win_size = world.win_size.to_logical(world.win_dpi);
                        world.win_dpi = val;
                        world.win_size = win_size.to_physical(world.win_dpi);
                    }
                    WindowEvent::Focused(val) => world.focus = val,
                    WindowEvent::Resized(val) => {
                        world.win_size = val.to_physical(world.win_dpi);
                    }
                    _ => (),
                }
            }
            Event::DeviceEvent { event, .. } => {
                use glutin::DeviceEvent;
                match event {
                    DeviceEvent::Key(keyboard_input) => {
                        world.keyboard_state.update(keyboard_input);

                        if let Some(vk) = keyboard_input.virtual_keycode {
                            if keyboard_input.state.is_pressed() && world.focus {
                                use glutin::VirtualKeyCode;
                                match vk {
                                    VirtualKeyCode::Tab => {
                                        // Don't trigger when we ALT TAB.
                                        if world.keyboard_state.lalt.is_released() {
                                            new_target_camera_key.wrapping_next_assign();
                                        }
                                    }
                                    VirtualKeyCode::Key1 => {
                                        new_attenuation_mode.wrapping_next_assign();
                                    }
                                    VirtualKeyCode::Key2 => {
                                        // new_window_mode.wrapping_next_assign();
                                        world.display_mode += 1;
                                        if world.display_mode >= 4 {
                                            world.display_mode = 1;
                                        }
                                    }
                                    VirtualKeyCode::Key3 => {
                                        new_render_technique.wrapping_next_assign();
                                    }
                                    VirtualKeyCode::Key4 => {
                                        new_light_space.wrapping_next_assign();
                                    }
                                    VirtualKeyCode::Key5 => {
                                        world.depth_prepass = !world.depth_prepass;
                                    }
                                    VirtualKeyCode::R => {
                                        reset_debug_camera = true;
                                    }
                                    VirtualKeyCode::Backslash => {
                                        configuration.virtual_stereo.enabled = !configuration.virtual_stereo.enabled;
                                    }
                                    VirtualKeyCode::C => {
                                        world.target_camera_mut().toggle_smoothness();
                                    }
                                    VirtualKeyCode::Escape => {
                                        world.running = false;
                                    }
                                    VirtualKeyCode::Space => {
                                        world.paused = !world.paused;
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                    DeviceEvent::Motion { axis, value } => {
                        if world.focus {
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

    let delta_time = 1.0 / DESIRED_UPS as f32;

    world.window_mode = new_window_mode;

    world
        .shader_compiler
        .replace_light_space(&mut world.current, new_light_space);
    world
        .shader_compiler
        .replace_attenuation_mode(&mut world.current, new_attenuation_mode);
    world
        .shader_compiler
        .replace_render_technique(&mut world.current, new_render_technique);

    if new_target_camera_key != world.target_camera_key {
        world.target_camera_key = new_target_camera_key;
        world.transition_camera.start_transition();
    }

    if reset_debug_camera {
        world.cameras.debug.target_transform = world.cameras.main.target_transform;
        world.transition_camera.start_transition();
    }

    for key in CameraKey::iter() {
        let is_target = world.target_camera_key == key;
        let delta = camera::CameraDelta {
            time: delta_time,
            position: if is_target && world.focus {
                Vector3::new(
                    world.keyboard_state.d.to_f32() - world.keyboard_state.a.to_f32(),
                    world.keyboard_state.q.to_f32() - world.keyboard_state.z.to_f32(),
                    world.keyboard_state.s.to_f32() - world.keyboard_state.w.to_f32(),
                ) * (1.0 + world.keyboard_state.lshift.to_f32() * 3.0)
            } else {
                Vector3::zero()
            },
            yaw: Rad(if is_target && world.focus {
                -mouse_dx as f32
            } else {
                0.0
            }),
            pitch: Rad(if is_target && world.focus {
                -mouse_dy as f32
            } else {
                0.0
            }),
            fovy: Rad(if is_target && world.focus {
                mouse_dscroll as f32
            } else {
                0.0
            }),
        };
        world.cameras[key].update(&delta);
    }

    world.transition_camera.update(camera::TransitionCameraUpdate {
        delta_time,
        end_camera: &world.target_camera().current_to_camera(),
    });

    if world.paused == false {
        {
            // let center = world.transition_camera.current_camera.transform.position;
            let center = Vector3::zero();
            let mut rng = rand::thread_rng();
            let p0 = Point3::from_value(-30.0) + center;
            let p1 = Point3::from_value(30.0) + center;

            for rain_drop in world.rain_drops.iter_mut() {
                rain_drop.update(delta_time, &mut rng, p0, p1);
            }

            for _ in 0..100 {
                if world.rain_drops.len() < configuration.global.rain_drop_max as usize {
                    world.rain_drops.push(rain::Particle::new(&mut rng, p0, p1));
                }
                if world.rain_drops.len() > configuration.global.rain_drop_max as usize {
                    world.rain_drops.truncate(configuration.global.rain_drop_max as usize);
                }
            }
        }

        world.tick += 1;
    }

    if vr_context.is_some() {
        // Pitch makes me dizzy.
        world.transition_camera.current_camera.transform.pitch = Rad(0.0);
    }
}

pub fn process_vr_events(vr_context: &Option<vr::Context>) {
    if let Some(vr_context) = &vr_context {
        while let Some(_event) = vr_context.system().poll_next_event() {
            // TODO: Handle vr events.
        }
    }
}

pub fn render_depth(
    gl: &gl::Gl,
    main_resources: &mut MainResources,
    resources: &Resources,
    world: &mut World,
    depth_renderer: &mut depth_renderer::Renderer,
) {
    unsafe {
        gl.viewport(0, 0, main_resources.dims.x, main_resources.dims.y);
        gl.bind_framebuffer(gl::FRAMEBUFFER, main_resources.framebuffer_name);
        gl.clear_color(world.clear_color[0], world.clear_color[1], world.clear_color[2], 1.0);
        // Reverse-Z.
        gl.clear_depth(0.0);
        gl.clear(gl::ClearFlag::COLOR_BUFFER | gl::ClearFlag::DEPTH_BUFFER);
        gl.enable(gl::DEPTH_TEST);
        gl.enable(gl::CULL_FACE);
        gl.cull_face(gl::BACK);
    }

    depth_renderer.render(gl, world, resources);
}

pub fn render_main(
    gl: &gl::Gl,
    main_resources: &mut MainResources,
    resources: &Resources,
    world: &mut World,
    cluster_parameters: Option<basic_renderer::ClusterParameters>,
    depth_renderer: &mut depth_renderer::Renderer,
    basic_renderer: &mut basic_renderer::Renderer,
) {
    unsafe {
        gl.viewport(0, 0, main_resources.dims.x, main_resources.dims.y);
        gl.bind_framebuffer(gl::FRAMEBUFFER, main_resources.framebuffer_name);
        gl.clear_color(world.clear_color[0], world.clear_color[1], world.clear_color[2], 1.0);
        // Reverse-Z.
        gl.clear_depth(0.0);
        gl.clear(gl::ClearFlag::COLOR_BUFFER | gl::ClearFlag::DEPTH_BUFFER);
        gl.enable(gl::DEPTH_TEST);
        gl.enable(gl::CULL_FACE);
        gl.cull_face(gl::BACK);
    }

    if world.depth_prepass {
        let profiler = &mut main_resources.depth_pass_profiler;
        profiler.start(&gl, world.frame, world.epoch);

        depth_renderer.render(gl, world, resources);

        unsafe {
            gl.depth_func(gl::GEQUAL);
            gl.depth_mask(gl::FALSE);
        }

        profiler.stop(&gl, world.frame, world.epoch);
    }

    let basic_params = &basic_renderer::Parameters {
        mode: match world.target_camera_key {
            CameraKey::Main => 0,
            CameraKey::Debug => world.display_mode,
        },
        cluster: cluster_parameters,
    };

    {
        let profiler = &mut main_resources.basic_pass_profiler;
        profiler.start(&gl, world.frame, world.epoch);

        basic_renderer.render(gl, basic_params, world, resources);
        if world.depth_prepass {
            unsafe {
                gl.depth_func(gl::GREATER);
                gl.depth_mask(gl::TRUE);
            }
        }

        profiler.stop(&gl, world.frame, world.epoch);
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
