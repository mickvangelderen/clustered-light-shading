#![allow(non_snake_case)]

mod basic_renderer;
mod camera;
mod convert;
mod filters;
mod frustrum;
mod keyboard_model;
mod world;

use openvr as vr;
use openvr::enums::Enum;
use renderer as log;
use cgmath::*;
use convert::*;
use gl_typed as gl;
use glutin::GlContext;
use notify::Watcher;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::path::PathBuf;
use std::ptr;
use world::Assets;

const DESIRED_UPS: f32 = 90.0;
const DESIRED_FPS: f32 = 90.0;

pub struct World {
    clear_color: [f32; 3],
    camera: camera::Camera,
    pos_from_cam_to_hmd: cgmath::Matrix4<f32>,
    keyboard_model: keyboard_model::KeyboardModel,
}

use std::sync::mpsc;
use std::thread;

fn write_g(name: &str, geo: (Vec<Vector3<f32>>, Vec<[u32; 3]>, Vec<u32>)) -> std::io::Result<()> {
    use std::io::Write;
    let mut bufwriter = std::io::BufWriter::new(std::fs::File::create(format!("{}.obj", name)).unwrap());
    let f = &mut bufwriter;

    for p in geo.0.iter() {
        writeln!(f, "v {} {} {}", p[0], p[1], p[2])?;
    }

    for subdivision in 0..(geo.2.len() - 1) {
        writeln!(f, "o {}_{}", name, subdivision)?;
        let triangles_start = geo.2[subdivision] as usize;
        let triangles_end = geo.2[subdivision + 1] as usize;
        for t in geo.1[triangles_start..triangles_end].iter() {
            writeln!(f, "f {} {} {}", t[0] + 1, t[1] + 1, t[2] + 1)?;
        }
    }

    Ok(())
}

fn write_obj_quads(name: &str, vertices: &[[f32; 3]], quads: &[[u32; 4]]) -> std::io::Result<()> {
    use std::io::Write;
    let mut bufwriter = std::io::BufWriter::new(std::fs::File::create(format!("{}.obj", name)).unwrap());
    let f = &mut bufwriter;

    for p in vertices.iter() {
        writeln!(f, "v {} {} {}", p[0], p[1], p[2])?;
    }

    writeln!(f, "o {}", name)?;
    for q in quads.iter() {
        writeln!(f, "f {} {} {} {}", q[0] + 1, q[1] + 1, q[2] + 1, q[3] + 1)?;
    }

    Ok(())
}

fn main() {
    // write_g("sphere", geogen::generate_iso_sphere(1.0, 4)).unwrap();

    let radius = 1.0;
    for subdivisions in 0..=4 {
        let spherical = geogen::generate_cubic_sphere_vertices(radius, subdivisions);
        let mut projected = geogen::generate_cube_vertices(radius, subdivisions);
        for vertex in projected.iter_mut() {
            *vertex = Vector3::from(*vertex).normalize_to(radius).into();
        }
        let quads = geogen::generate_cube_quads(subdivisions);
        write_obj_quads(&format!("cubic_sphere_{}", subdivisions), &spherical, &quads).unwrap();
        write_obj_quads(&format!("cube_projected_onto_sphere_{}", subdivisions), &projected, &quads).unwrap();
    }

    let current_dir = std::env::current_dir().unwrap();

    let basic_renderer_vs_path: PathBuf = [
        current_dir.as_ref(),
        Path::new("data"),
        Path::new("basic_renderer.vert"),
    ]
    .into_iter()
    .collect();

    let basic_renderer_fs_path: PathBuf = [
        current_dir.as_ref(),
        Path::new("data"),
        Path::new("basic_renderer.frag"),
    ]
    .into_iter()
    .collect();

    let start_instant = std::time::Instant::now();

    let (tx_log, rx_log) = mpsc::channel::<log::Entry>();

    let timing_thread = thread::Builder::new()
        .name("log".to_string())
        .spawn(move || {
            use std::fs;
            use std::io;
            use std::io::Write;

            let mut file = io::BufWriter::new(fs::File::create("log/log.bin").unwrap());

            for entry in rx_log.iter() {
                file.write_all(&entry.into_ne_bytes()).unwrap();
            }

            file.flush().unwrap();
        })
        .unwrap();

    let (tx_fs, rx_fs) = mpsc::channel();

    let mut watcher = notify::raw_watcher(tx_fs).unwrap();

    watcher
        .watch("data", notify::RecursiveMode::Recursive)
        .unwrap();

    let mut world = World {
        clear_color: [0.0, 0.0, 0.0],
        camera: camera::Camera {
            position: Vector3::new(0.0, 0.5, 1.0),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            fovy: Deg(90.0).into(),
            positional_velocity: 2.0,
            angular_velocity: 0.8,
            zoom_velocity: 1.0,
        },
        pos_from_cam_to_hmd: Matrix4::from_translation(Vector3::zero()),
        keyboard_model: keyboard_model::KeyboardModel::new(),
    };

    let mut events_loop = glutin::EventsLoop::new();
    let gl_window = glutin::GlWindow::new(
        glutin::WindowBuilder::new()
            .with_title("Hello world!")
            .with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0)),
        glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)))
            .with_gl_profile(glutin::GlProfile::Core)
            // We do not wan't vsync since it will cause our loop to sync to the
            // desktop display frequency which is probably lower than the HMD's
            // 90Hz.
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
        println!("OpenGL version {}", gl.get_string(gl::VERSION));
    }

    let mut renderer = unsafe {
        let mut renderer = basic_renderer::Renderer::new(&gl, &world);
        let vs_bytes = std::fs::read(&basic_renderer_vs_path).unwrap();
        let fs_bytes = std::fs::read(&basic_renderer_fs_path).unwrap();
        renderer.update(
            &gl,
            basic_renderer::Update {
                vertex_shader: Some(&vs_bytes),
                fragment_shader: Some(&fs_bytes),
            },
        );
        renderer
    };

    let mut assets = Assets::new(&gl, &renderer);

    // === VR ===
    let vr_resources = match vr::Context::new(vr::ApplicationType::Scene) {
        Ok(context) => unsafe {
            let dims = context.system().get_recommended_render_target_size();
            println!("Recommender render target size: {:?}", dims);
            let eye_left = EyeResources::new(&gl, &context, vr::Eye::Left, dims);
            let eye_right = EyeResources::new(&gl, &context, vr::Eye::Right, dims);

            Some(VrResources {
                context,
                dims,
                eye_left,
                eye_right,
            })
        },
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

    let mut fps_average = filters::MovingAverageF32::new(DESIRED_FPS);
    let mut last_frame_start = std::time::Instant::now();

    let mut running = true;
    while running {
        // File watch events.
        for event in rx_fs.try_iter() {
            if let Some(ref path) = event.path {
                match path {
                    path if path == &basic_renderer_vs_path => unsafe {
                        let vs_bytes = std::fs::read(&path).unwrap();
                        renderer.update(
                            &gl,
                            basic_renderer::Update {
                                vertex_shader: Some(&vs_bytes),
                                fragment_shader: None,
                            },
                        );
                    },
                    path if path == &basic_renderer_fs_path => unsafe {
                        let fs_bytes = std::fs::read(&path).unwrap();
                        renderer.update(
                            &gl,
                            basic_renderer::Update {
                                vertex_shader: None,
                                fragment_shader: Some(&fs_bytes),
                            },
                        );
                    },
                    _ => {}
                }
            }
        }

        let simulation_start_nanos = start_instant.elapsed().as_nanos() as u64;

        let mut mouse_dx = 0.0;
        let mut mouse_dy = 0.0;
        let mut mouse_dscroll = 0.0;

        events_loop.poll_events(|event| {
            use glutin::Event;
            match event {
                Event::WindowEvent { event, .. } => {
                    use glutin::WindowEvent;
                    match event {
                        WindowEvent::CloseRequested => running = false,
                        WindowEvent::HiDpiFactorChanged(val) => win_dpi = val,
                        WindowEvent::Focused(val) => focus = val,
                        WindowEvent::Resized(val) => win_size = val,
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

                                if focus {
                                    use glutin::VirtualKeyCode;
                                    match vk {
                                        VirtualKeyCode::W => input_forward = keyboard_input.state,
                                        VirtualKeyCode::S => input_backward = keyboard_input.state,
                                        VirtualKeyCode::A => input_left = keyboard_input.state,
                                        VirtualKeyCode::D => input_right = keyboard_input.state,
                                        VirtualKeyCode::Q => input_up = keyboard_input.state,
                                        VirtualKeyCode::Z => input_down = keyboard_input.state,
                                        VirtualKeyCode::Escape => running = false,
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

        use glutin::ElementState;

        let delta_time = 1.0 / DESIRED_UPS as f32;

        world.camera.update(&camera::CameraUpdate {
            delta_time,
            delta_position: Vector3 {
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
            },
            delta_yaw: Rad(mouse_dx as f32),
            delta_pitch: Rad(mouse_dy as f32),
            delta_scroll: mouse_dscroll as f32,
        });

        world.keyboard_model.simulate(delta_time);

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
        unsafe {
            let physical_size = win_size.to_physical(win_dpi);

            let frustrum = {
                let z0 = 0.2;
                let dy = z0 * Rad::tan(Rad::from(world.camera.fovy) / 2.0);
                let dx = dy * physical_size.width as f32 / physical_size.height as f32;
                frustrum::Frustrum {
                    x0: -dx,
                    x1: dx,
                    y0: -dy,
                    y1: dy,
                    z0,
                    z1: 100.0,
                }
            };

            let pos_from_hmd_to_clp = Matrix4::from(Perspective {
                left: frustrum.x0,
                right: frustrum.x1,
                bottom: frustrum.y0,
                top: frustrum.y1,
                near: frustrum.z0,
                far: frustrum.z1,
            });

            renderer.render(
                &gl,
                &basic_renderer::Parameters {
                    framebuffer: None,
                    width: physical_size.width as i32,
                    height: physical_size.height as i32,
                    pos_from_cam_to_clp: pos_from_hmd_to_clp,
                },
                &world,
                &assets,
            );
        }

        // === VR ===
        if let Some(ref vr_resources) = vr_resources {
            unsafe {
                renderer.render(
                    &gl,
                    &basic_renderer::Parameters {
                        framebuffer: Some(vr_resources.eye_left.framebuffer),
                        width: vr_resources.dims.width as i32,
                        height: vr_resources.dims.height as i32,
                        pos_from_cam_to_clp: vr_resources.eye_left.pos_from_hmd_to_clp
                            * world.pos_from_cam_to_hmd,
                    },
                    &world,
                    &assets,
                );

                renderer.render(
                    &gl,
                    &basic_renderer::Parameters {
                        framebuffer: Some(vr_resources.eye_right.framebuffer),
                        width: vr_resources.dims.width as i32,
                        height: vr_resources.dims.height as i32,
                        pos_from_cam_to_clp: vr_resources.eye_right.pos_from_hmd_to_clp
                            * world.pos_from_cam_to_hmd,
                    },
                    &world,
                    &assets,
                );

                // NOTE(mickvangelderen): Binding the color attachments causes SIGSEGV!!!
                {
                    let mut texture_t = vr_resources.eye_left.gen_texture_t();
                    vr_resources
                        .compositor()
                        .submit(
                            vr_resources.eye_left.eye,
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
                {
                    let mut texture_t = vr_resources.eye_right.gen_texture_t();
                    vr_resources
                        .compositor()
                        .submit(
                            vr_resources.eye_right.eye,
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
    eye: vr::Eye,
    pos_from_hmd_to_clp: Matrix4<f32>,
    framebuffer: gl::FramebufferName,
    color_texture: gl::TextureName,
    #[allow(unused)]
    depth_texture: gl::TextureName,
}

impl EyeResources {
    unsafe fn new(gl: &gl::Gl, vr: &vr::Context, eye: vr::Eye, dims: vr::Dimensions) -> Self {
        // VR.
        let pos_from_eye_to_clp: Matrix4<f32> = vr
            .system()
            .get_projection_matrix(eye, 0.2, 200.0)
            .hmd_into();
        let pos_from_eye_to_hmd: Matrix4<f32> =
            vr.system().get_eye_to_head_transform(eye).hmd_into();
        let pos_from_hmd_to_clp = pos_from_eye_to_clp * pos_from_eye_to_hmd.invert().unwrap();

        // OpenGL.
        let framebuffer = {
            let mut names: [Option<gl::FramebufferName>; 1] = mem::uninitialized();
            gl.gen_framebuffers(&mut names);
            let [name] = names;
            name.expect("Failed to acquire framebuffer name.")
        };
        let (color_texture, depth_texture) = {
            let mut names: [Option<gl::TextureName>; 2] = mem::uninitialized();
            gl.gen_textures(&mut names);
            let [n0, n1] = names;
            (
                n0.expect("Failed to acquire texture name."),
                n1.expect("Failed to acquire texture name."),
            )
        };

        gl.bind_texture(gl::TEXTURE_2D, color_texture);
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

        gl.bind_texture(gl::TEXTURE_2D, depth_texture);
        {
            gl.tex_image_2d(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH24_STENCIL8,
                dims.width as i32,
                dims.height as i32,
                gl::DEPTH_STENCIL,
                gl::UNSIGNED_INT_24_8,
                ptr::null(),
            );
            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
        }

        gl.unbind_texture(gl::TEXTURE_2D);

        gl.bind_framebuffer(gl::FRAMEBUFFER, Some(framebuffer));
        {
            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                color_texture,
                0,
            );

            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::TEXTURE_2D,
                depth_texture,
                0,
            );

            assert!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE.into()
            );
        }

        gl.bind_framebuffer(gl::FRAMEBUFFER, None);

        EyeResources {
            eye,
            pos_from_hmd_to_clp,
            framebuffer,
            color_texture,
            depth_texture,
        }
    }

    fn gen_texture_t(&self) -> vr::sys::Texture_t {
        // NOTE(mickvangelderen): The handle is not actually a pointer in
        // OpenGL's case, it's just the texture name.
        vr::sys::Texture_t {
            handle: self.color_texture.into_u32() as usize as *const c_void as *mut c_void,
            eType: vr::sys::ETextureType_TextureType_OpenGL,
            eColorSpace: vr::sys::EColorSpace_ColorSpace_Gamma, // TODO(mickvangelderen): IDK
        }
    }
}
