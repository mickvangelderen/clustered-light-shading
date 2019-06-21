#![feature(euclidean_division)]
#![allow(non_snake_case)]

// Has to go first.
#[macro_use]
mod macros;

pub(crate) use gl_typed as gl;
pub(crate) use incremental as ic;
pub(crate) use log::*;
pub(crate) use regex::{Regex, RegexBuilder};
#[allow(unused_imports)]
pub(crate) use std::convert::{TryFrom, TryInto};
#[allow(unused_imports)]
pub(crate) use std::num::{NonZeroU32, NonZeroU64};

mod basic_renderer;
mod bounding_box;
pub mod camera;
mod cgmath_ext;
pub mod clamp;
mod cls;
mod cluster_renderer;
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
mod mono_stereo;
mod overlay_renderer;
mod rendering;
mod resources;
mod timings;
mod viewport;
mod window_mode;
mod world;

use crate::bounding_box::*;
use crate::cgmath_ext::*;
use crate::frustrum::*;
use crate::gl_ext::*;
// use crate::mono_stereo::*;
use crate::rendering::*;
use crate::resources::Resources;
// use crate::timings::*;
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
use openvr::enums::Enum;
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
    pub width: i32,
    pub height: i32,
    // Main frame resources.
    pub framebuffer_name: gl::NonDefaultFramebufferName,
    pub color_texture: Texture<gl::TEXTURE_2D, gl::RGBA16F>,
    pub depth_texture: Texture<gl::TEXTURE_2D, gl::DEPTH24_STENCIL8>,
    pub nor_in_cam_texture: Texture<gl::TEXTURE_2D, gl::R11F_G11F_B10F>,
}

impl MainResources {
    pub fn new(gl: &gl::Gl, width: i32, height: i32) -> Self {
        unsafe {
            // Textures.
            let texture_update = TextureUpdate::new()
                .data(width, height, None)
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
                width,
                height,
                framebuffer_name,
                color_texture,
                depth_texture,
                nor_in_cam_texture,
            }
        }
    }

    pub fn resize(&mut self, gl: &gl::Gl, width: i32, height: i32) {
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;

            let texture_update = TextureUpdate::new().data(width, height, None);
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

        let win_dpi = gl_window.get_hidpi_factor();
        let win_size = gl_window.get_inner_size().unwrap().to_physical(win_dpi);

        World {
            running: true,
            focus: false,
            win_dpi,
            win_size,
            keyboard_state: KeyboardState::default(),
            resource_dir,
            configuration_path,
            tick: 0,
            clear_color: [0.0, 0.0, 0.0],
            window_mode: WindowMode::Main,
            depth_prepass: true,
            light_space: ic::Leaf::clean(&mut global, LightSpace::Cam),
            light_space_regex: LightSpace::regex(),
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
            global,
        }
    };

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

        // Reverse-Z.
        gl.clip_control(gl::LOWER_LEFT, gl::ZERO_TO_ONE);
        gl.depth_func(gl::GREATER);

        if configuration.global.framebuffer_srgb {
            gl.enable(gl::FRAMEBUFFER_SRGB);
        } else {
            gl.disable(gl::FRAMEBUFFER_SRGB);
        }
    }

    let mut depth_renderer = depth_renderer::Renderer::new(&gl, &mut world);
    let mut line_renderer = line_renderer::Renderer::new(&gl, &mut world);
    let mut basic_renderer = basic_renderer::Renderer::new(&gl, &mut world);
    let mut overlay_renderer = overlay_renderer::Renderer::new(&gl, &mut world);
    let mut cluster_renderer = cluster_renderer::Renderer::new(&gl, &mut world);

    let depth_timer = QueryTimer::new(&gl);
    let basic_timer = QueryTimer::new(&gl);

    let resources = resources::Resources::new(&gl, &world.resource_dir, &configuration);

    let vr_context = vr::Context::new(vr::ApplicationType::Scene)
        .map_err(|error| {
            eprintln!(
                "Failed to acquire context: {:?}",
                vr::InitError::from_unchecked(error).unwrap()
            );
        })
        .ok();

    let mut fps_average = filters::MovingAverageF32::new(0.0);
    let mut last_frame_start = time::Instant::now();

    // let mut light_buffer_vec: Vec<LightBuffer> = Vec::new();
    // let mut tile_buffer_vec = Vec::new();
    // let mut cluster_buffer_vec = Vec::new();
    // let mut camera_buffer_vec: Vec<CameraBuffer> = Vec::new();
    // let mut main_resources_vec: Vec<MainResources> = Vec::new();
    // let mut debug_resources_vec = Vec::new();

    let mut light_buffer_pool = BufferPool::new();
    let mut cluster_buffer_pool = BufferPool::new();
    let mut camera_buffer_pool = BufferPool::new();
    let mut main_resources_vec = Vec::new();

    while world.running {
        pub struct EyeData {
            tangents: [f32; 4],
            cam_to_hmd: Matrix4<f64>,
        }

        struct StereoData<'a> {
            vr_context: &'a vr::Context,
            hmd_to_bdy: Matrix4<f64>,
            eyes: EyeMap<EyeData>,
        }

        let stereo_data = vr_context.as_ref().map(|vr_context| {
            let mut poses: [vr::sys::TrackedDevicePose_t; vr::sys::k_unMaxTrackedDeviceCount as usize] =
                unsafe { mem::zeroed() };
            // NOTE: OpenVR will block upon querying the pose for as long as
            // possible but no longer than it takes to submit the new frame. This is
            // done to render the most up-to-date application state as possible.
            vr_context.compositor().wait_get_poses(&mut poses[..], None).unwrap();

            let hmd_pose = poses[vr::sys::k_unTrackedDeviceIndex_Hmd as usize];
            assert!(hmd_pose.bPoseIsValid, "Received invalid pose from VR.");
            let hmd_to_bdy = Matrix4::from_hmd(hmd_pose.mDeviceToAbsoluteTracking.m).cast().unwrap();

            StereoData {
                vr_context,
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
        });

        process_fs_events(&fs_rx, &gl, &mut world, &mut configuration);
        process_window_events(&mut events_loop, &vr_context, &mut world);
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

        let mut main_resources_index = 0;
        let mut debug_resources_index = 0;

        fn mono_frustrum(camera: &camera::Camera, viewport: Viewport<i32>) -> Frustrum<f64> {
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

        fn stereo_frustrum(camera_properties: &camera::CameraProperties, tangents: [f32; 4]) -> Frustrum<f64> {
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

        let compute_and_upload_light_positions = |lighting_buffer_name, wld_to_lgt| unsafe {
            let mut point_lights: [light::PointLightBufferEntry; rendering::POINT_LIGHT_CAPACITY as usize] =
                std::mem::uninitialized();
            for i in 0..rendering::POINT_LIGHT_CAPACITY as usize {
                point_lights[i] = light::PointLightBufferEntry::from_point_light(resources.point_lights[i], wld_to_lgt);
            }
            let lighting_buffer = light::LightingBuffer { point_lights };
            gl.named_buffer_data(lighting_buffer_name, lighting_buffer.value_as_bytes(), gl::STREAM_DRAW);
            gl.bind_buffer_base(
                gl::UNIFORM_BUFFER,
                rendering::LIGHT_BUFFER_BINDING,
                lighting_buffer_name,
            );
        };

        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        enum RealLightSpace {
            Wld,
            Hmd,
            Cls,
            Cam,
        }

        let real_light_space = match world.render_technique.value {
            RenderTechnique::Clustered => RealLightSpace::Cls,
            _ => match world.light_space.value {
                LightSpace::Wld => RealLightSpace::Wld,
                LightSpace::Hmd => RealLightSpace::Hmd,
                LightSpace::Cam => RealLightSpace::Cam,
            },
        };

        if real_light_space == RealLightSpace::Wld {
            let buffer_index = light_buffer_pool.unused(&gl);
            compute_and_upload_light_positions(light_buffer_pool[buffer_index], Matrix4::identity());
        }

        if world.render_technique.value == RenderTechnique::Clustered {}

        match stereo_data {
            Some(StereoData {
                vr_context,
                hmd_to_bdy,
                eyes,
            }) => {
                match world.render_technique.value {
                    RenderTechnique::Naive => {
                        //
                    }
                    RenderTechnique::Tiled => {
                        //
                    }
                    RenderTechnique::Clustered => {
                        // compute cls space.
                        let cluster_camera = &world.cameras.main;
                        let bdy_to_wld = cluster_camera.current_transform.pos_to_parent();
                        let cam_mats = eyes.map(|eye| {
                            let frustrum = stereo_frustrum(&cluster_camera.properties, eye.tangents);
                            let cam_to_clp = frustrum.perspective(DEPTH_RANGE).cast::<f64>().unwrap();
                            let clp_to_cam = cam_to_clp.invert().unwrap();
                        });
                        unimplemented!();
                        // let buffer_index = light_buffer_pool.unused(&gl);
                        // compute_and_upload_light_positions(light_buffer_pool[buffer_index], wld_to_cls);
                    }
                }

                let render_camera = &world.transition_camera.current_camera;
                let bdy_to_wld = render_camera.transform.pos_to_parent().cast::<f64>().unwrap();

                let hmd_to_wld = bdy_to_wld * hmd_to_bdy;
                let wld_to_hmd = hmd_to_wld.invert().unwrap();

                if real_light_space == RealLightSpace::Hmd {
                    let buffer_index = light_buffer_pool.unused(&gl);
                    compute_and_upload_light_positions(light_buffer_pool[buffer_index], wld_to_hmd.cast().unwrap());
                }

                for &eye in EYE_KEYS.iter() {
                    let EyeData { tangents, cam_to_hmd } = eyes[eye];

                    let cam_to_wld = hmd_to_wld * cam_to_hmd;
                    let wld_to_cam = cam_to_wld.invert().unwrap();

                    let frustrum = stereo_frustrum(&render_camera.properties, tangents);
                    let cam_to_clp = frustrum.perspective(DEPTH_RANGE).cast::<f64>().unwrap();
                    let clp_to_cam = cam_to_clp.invert().unwrap();

                    if real_light_space == RealLightSpace::Cam {
                        let buffer_index = light_buffer_pool.unused(&gl);
                        compute_and_upload_light_positions(light_buffer_pool[buffer_index], wld_to_cam.cast().unwrap());
                    }

                    // set cam_pos_in_lgt
                    let cam_pos_in_wld = render_camera.transform.position.cast::<f64>().unwrap();

                    let wld_to_lgt = match world.light_space.value {
                        LightSpace::Wld => Matrix4::identity(),
                        LightSpace::Hmd => wld_to_hmd,
                        LightSpace::Cam => wld_to_cam,
                    };

                    // rendering requires
                    // obj_to_wld
                    // wld_to_cam
                    // cam_to_clp
                    // wld_to_lgt
                    //
                    // clp_to_cam
                    //
                    // TODO: render main
                    //             depth_renderer.render(&gl, &mut world, &resources);
                }
            }
            None => {
                let render_camera = &world.transition_camera.current_camera;
                let viewport =
                    Viewport::from_dimensions(Vector2::new(world.win_size.width as i32, world.win_size.height as i32));
                let cam_to_wld = render_camera.transform.pos_to_parent().cast::<f64>().unwrap();
                let wld_to_cam = render_camera.transform.pos_from_parent().cast::<f64>().unwrap();

                let frustrum = mono_frustrum(render_camera, viewport);
                let cam_to_clp = frustrum.perspective(DEPTH_RANGE).cast::<f64>().unwrap();
                let clp_to_cam = cam_to_clp.invert().unwrap();

                if real_light_space == RealLightSpace::Cam || real_light_space == RealLightSpace::Hmd {
                    let buffer_index = light_buffer_pool.unused(&gl);
                    compute_and_upload_light_positions(light_buffer_pool[buffer_index], wld_to_cam.cast().unwrap());
                }

                let wld_to_lgt = match world.light_space.value {
                    LightSpace::Wld => Matrix4::identity(),
                    LightSpace::Hmd => wld_to_cam,
                    LightSpace::Cam => wld_to_cam,
                };
                let lgt_to_wld = wld_to_lgt.invert().unwrap();
                let cam_pos_in_wld = render_camera.transform.position.cast::<f64>().unwrap().extend(1.0);
                let cam_pos_in_lgt = wld_to_lgt * cam_pos_in_wld;

                unsafe {
                    let buffer_index = camera_buffer_pool.unused(&gl);

                    let camera_buffer = CameraBuffer {
                        wld_to_cam: wld_to_cam.cast().unwrap(),
                        cam_to_wld: cam_to_wld.cast().unwrap(),

                        cam_to_clp: cam_to_clp.cast().unwrap(),
                        clp_to_cam: clp_to_cam.cast().unwrap(),

                        wld_to_lgt: wld_to_lgt.cast().unwrap(),
                        lgt_to_wld: lgt_to_wld.cast().unwrap(),

                        cam_pos_in_lgt: cam_pos_in_lgt.cast().unwrap(),
                    };

                    let buffer_name = camera_buffer_pool[buffer_index];

                    gl.bind_buffer_base(gl::UNIFORM_BUFFER, rendering::CAMERA_BUFFER_BINDING, buffer_name);
                    gl.named_buffer_data(buffer_name, camera_buffer.value_as_bytes(), gl::STREAM_DRAW);
                }

                if main_resources_vec.len() <= main_resources_index {
                    let main_resources = MainResources::new(&gl, viewport.dimensions.x, viewport.dimensions.y);
                    main_resources_vec.push(main_resources);
                }

                let main_resources = &mut main_resources_vec[main_resources_index];
                main_resources_index += 1;

                main_resources.resize(&gl, viewport.dimensions.x, viewport.dimensions.y);

                render_main(
                    &gl,
                    &main_resources,
                    &resources,
                    &mut world,
                    &mut depth_renderer,
                    &mut basic_renderer,
                );

                unsafe {
                    gl.blit_named_framebuffer(
                        main_resources.framebuffer_name.into(),
                        gl::FramebufferName::Default,
                        viewport.origin.x,
                        viewport.origin.y,
                        viewport.dimensions.x,
                        viewport.dimensions.y,
                        viewport.origin.x,
                        viewport.origin.y,
                        viewport.dimensions.x,
                        viewport.dimensions.y,
                        gl::BlitMask::COLOR_BUFFER_BIT,
                        gl::LINEAR,
                    );
                }
            }
        }

        // if let Some((index, debug_view_data)) = maybe_debug {
        //     view_resources.bind_index(&gl, index);

        //     debug_view_data.viewport.set(&gl);

        //     cluster_renderer.render(
        //         &gl,
        //         &cluster_renderer::Parameters {
        //             cluster_data: &cluster_data,
        //             configuration: &configuration.clustered_light_shading,
        //         },
        //         &mut world,
        //         &resources,
        //     );

        //     let corners_in_clp = Frustrum::corners_in_clp(DEPTH_RANGE);
        //     let vertices: Vec<[f32; 3]> = corners_in_clp
        //         .iter()
        //         .map(|point| point.cast().unwrap().into())
        //         .collect();

        //     line_renderer.render(
        //         &gl,
        //         &line_renderer::Parameters {
        //             vertices: &vertices[..],
        //             indices: &sun_frustrum_indices[..],
        //             obj_to_wld: &cls_view_data_ext.view_data.clp_to_wld.cast().unwrap(),
        //         },
        //         &mut world,
        //     );
        // }

        gl_window.swap_buffers().unwrap();

        // TODO: Borrow the pool instead.
        light_buffer_pool.reset(&gl);
        cluster_buffer_pool.reset(&gl);
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
                for source in world.sources.iter_mut() {
                    if source.path == path {
                        world.global.mark(&mut source.modified);
                    }
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
    world: &mut World,
) {
    let mut mouse_dx = 0.0;
    let mut mouse_dy = 0.0;
    let mut mouse_dscroll = 0.0;
    let mut new_target_camera_key = world.target_camera_key;
    let mut new_window_mode = world.window_mode;
    let mut new_light_space = world.light_space.value;
    let mut new_attenuation_mode = world.attenuation_mode.value;
    let mut new_render_technique = world.render_technique.value;

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
                                        new_window_mode.wrapping_next_assign();
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
                                    VirtualKeyCode::C => {
                                        world.target_camera_mut().toggle_smoothness();
                                    }
                                    VirtualKeyCode::Escape => {
                                        world.running = false;
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

    {
        let old_light_space = world.light_space.replace(&mut world.global, new_light_space);
        if old_light_space != new_light_space {
            info!("Light space: {:?}", new_light_space);
        }
    }

    {
        let old_attenuation_mode = world.attenuation_mode.replace(&mut world.global, new_attenuation_mode);
        if old_attenuation_mode != new_attenuation_mode {
            info!("Attenuation mode: {:?}", new_attenuation_mode);
        }
    }

    {
        let old_render_technique = world.render_technique.replace(&mut world.global, new_render_technique);
        if old_render_technique != new_render_technique {
            info!("Render technique: {:?}", new_render_technique);
        }
    }

    if new_target_camera_key != world.target_camera_key {
        world.target_camera_key = new_target_camera_key;
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

    if vr_context.is_some() {
        // Pitch makes me dizzy.
        world.transition_camera.current_camera.transform.pitch = Rad(0.0);
    }

    world.tick += 1;
}

pub fn process_vr_events(vr_context: &Option<vr::Context>) {
    if let Some(vr_context) = &vr_context {
        while let Some(_event) = vr_context.system().poll_next_event() {
            // TODO: Handle vr events.
        }
    }
}

pub fn render_main(
    gl: &gl::Gl,
    main_resources: &MainResources,
    resources: &Resources,
    world: &mut World,
    depth_renderer: &mut depth_renderer::Renderer,
    basic_renderer: &mut basic_renderer::Renderer,
) {
    unsafe {
        gl.bind_framebuffer(gl::FRAMEBUFFER, main_resources.framebuffer_name);
        gl.clear_color(world.clear_color[0], world.clear_color[1], world.clear_color[2], 1.0);
        // Reverse-Z.
        gl.clear_depth(0.0);
        gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);
        gl.enable(gl::DEPTH_TEST);
        gl.enable(gl::CULL_FACE);
        gl.cull_face(gl::BACK);
    }

    if world.depth_prepass {
        depth_renderer.render(gl, world, resources);

        unsafe {
            gl.depth_func(gl::GEQUAL);
            gl.depth_mask(gl::FALSE);
        }
    }

    basic_renderer.render(gl, &basic_renderer::Parameters {}, world, resources);

    if world.depth_prepass {
        unsafe {
            gl.depth_func(gl::GREATER);
            gl.depth_mask(gl::TRUE);
        }
    }
}
