#![cfg_attr(feature = "profile", feature(custom_inner_attributes))]
#![allow(non_snake_case)]

mod basic_renderer;
mod keyboard_model;
mod camera;
mod convert;
mod frustrum;

use openvr as vr;
use openvr::enums::Enum;

use cgmath::*;
use convert::*;
use gl_typed as gl;
use glutin::GlContext;
use std::mem;
use std::os::raw::c_void;
use std::ptr;

const DESIRED_UPS: f32 = 90.0;

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

pub struct World {
    clear_color: [f32; 3],
    camera: camera::Camera,
    pos_from_cam_to_hmd: cgmath::Matrix4<f32>,
    models: Vec<tobj::Model>,
    #[allow(unused)]
    materials: Vec<tobj::Material>,
    keyboard_model: keyboard_model::KeyboardModel,
}

fn main() {
    flame::start("initialize");
    let obj = tobj::load_obj(&std::path::Path::new("data/keyboard.obj"));
    let (models, materials) = obj.unwrap();

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
        models,
        materials,
        keyboard_model: keyboard_model::KeyboardModel::new(),
    };

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

    unsafe { gl_window.make_current().unwrap() };

    let renderer = unsafe { basic_renderer::Renderer::new(&gl, &world) };

    let mut input_forward = glutin::ElementState::Released;
    let mut input_backward = glutin::ElementState::Released;
    let mut input_left = glutin::ElementState::Released;
    let mut input_right = glutin::ElementState::Released;
    let mut input_up = glutin::ElementState::Released;
    let mut input_down = glutin::ElementState::Released;

    flame::end("initialize");

    let mut running = true;
    while running {
        flame::start("loop");
        flame::start("update");
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
                                world.keyboard_model.process_event(vk, keyboard_input.state);

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
                        DeviceEvent::Motion { axis, value } => {
                            // if window_has_focus {
                            match axis {
                                0 => mouse_dx += value,
                                1 => mouse_dy += value,
                                3 => mouse_dscroll += value,
                                _ => (),
                            }
                            // }
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

        flame::end("update");

        // === VR ===
        flame::start("update vr");
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
        flame::end("update vr");
        // --- VR ---

        // draw everything here
        unsafe {
            flame::start("render mono");
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
                    framebuffer: &gl::DefaultFramebufferName,
                    width: physical_size.width as i32,
                    height: physical_size.height as i32,
                    pos_from_cam_to_clp: pos_from_hmd_to_clp,
                },
                &world,
            );
            flame::end("render mono");
        }

        // === VR ===
        if let Some(ref vr_resources) = vr_resources {
            unsafe {
                flame::start("render stereo left");
                renderer.render(
                    &gl,
                    &basic_renderer::Parameters {
                        framebuffer: &vr_resources.eye_left.framebuffer,
                        width: vr_resources.dims.width as i32,
                        height: vr_resources.dims.height as i32,
                        pos_from_cam_to_clp: vr_resources.eye_left.pos_from_hmd_to_clp
                            * world.pos_from_cam_to_hmd,
                    },
                    &world,
                );
                flame::end("render stereo left");

                flame::start("render stereo right");
                renderer.render(
                    &gl,
                    &basic_renderer::Parameters {
                        framebuffer: &vr_resources.eye_right.framebuffer,
                        width: vr_resources.dims.width as i32,
                        height: vr_resources.dims.height as i32,
                        pos_from_cam_to_clp: vr_resources.eye_right.pos_from_hmd_to_clp
                            * world.pos_from_cam_to_hmd,
                    },
                    &world,
                );
                flame::end("render stereo right");

                // NOTE(mickvangelderen): Binding the color attachments causes SIGSEGV!!!
                {
                    flame::start("submit stereo left");
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
                    flame::end("submit stereo left");
                }
                {
                    flame::start("submit stereo right");
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
                    flame::end("submit stereo right");
                }
            }
        }
        // --- VR ---

        flame::start("swap");
        gl_window.swap_buffers().unwrap();
        flame::end("swap");

        // std::thread::sleep(std::time::Duration::from_millis(17));
        flame::end("loop");
    }

    #[cfg(feature = "profile")]
    flame::dump_html(&mut std::fs::File::create("log/flame-graph.html").unwrap()).unwrap();
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
            (n0.expect("Failed to acquire texture name."), n1.expect("Failed to acquire texture name."))
        };

        gl.bind_texture(gl::TEXTURE_2D, &color_texture);
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

        gl.bind_texture(gl::TEXTURE_2D, &depth_texture);
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

        gl.bind_texture(gl::TEXTURE_2D, &gl::Unbind);

        gl.bind_framebuffer(gl::FRAMEBUFFER, &framebuffer);
        {
            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                &color_texture,
                0,
            );

            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::TEXTURE_2D,
                &depth_texture,
                0,
            );

            assert!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE.into()
            );
        }
        gl.bind_framebuffer(gl::FRAMEBUFFER, &gl::DefaultFramebufferName);

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
            handle: self.color_texture.as_u32() as usize as *const c_void as *mut c_void,
            eType: vr::sys::ETextureType_TextureType_OpenGL,
            eColorSpace: vr::sys::EColorSpace_ColorSpace_Gamma, // TODO(mickvangelderen): IDK
        }
    }
}
