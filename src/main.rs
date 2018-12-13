#![allow(non_snake_case)]

pub mod vr;

use glutin::GlContext;
use openvr_sys::*;

use std::ffi::CStr;
use std::mem;
use std::os::raw::*;
use std::ptr;

unsafe fn get_string(name: u32) -> &'static str {
    CStr::from_ptr(gl::GetString(name) as *const i8)
        .to_str()
        .unwrap()
}

unsafe fn vr_get_generic_interface(
    pchInterfaceVersion: &[u8],
) -> Result<*mut c_void, EVRInitError> {
    // NOTE(mickvangelderen): WHAT WHAT WAHT WHY HOW WHERE DOC?
    let mut magic = Vec::from(b"FnTable:".as_ref());
    magic.extend(pchInterfaceVersion);

    let mut err = EVRInitError_VRInitError_None;
    let p = VR_GetGenericInterface(magic.as_ptr() as *const c_char, &mut err);
    if err == EVRInitError_VRInitError_None {
        Ok(p as *mut c_void)
    } else {
        Err(err)
    }
}

unsafe fn vr_init(eApplicationType: EVRApplicationType) -> Result<COpenVRContext, EVRInitError> {
    let mut err = EVRInitError_VRInitError_None;
    let _token = VR_InitInternal(&mut err, eApplicationType);
    if err == EVRInitError_VRInitError_None {
        Ok(COpenVRContext {
            m_pVRSystem: vr_get_generic_interface(IVRSystem_Version.as_ref())? as isize,
            m_pVRChaperone: vr_get_generic_interface(IVRChaperone_Version.as_ref())? as isize,
            m_pVRChaperoneSetup: vr_get_generic_interface(IVRChaperoneSetup_Version.as_ref())?
                as isize,
            m_pVRCompositor: vr_get_generic_interface(IVRCompositor_Version.as_ref())? as isize,
            m_pVROverlay: vr_get_generic_interface(IVROverlay_Version.as_ref())? as isize,
            m_pVRResources: vr_get_generic_interface(IVRResources_Version.as_ref())? as isize,
            m_pVRRenderModels: vr_get_generic_interface(IVRRenderModels_Version.as_ref())? as isize,
            m_pVRExtendedDisplay: vr_get_generic_interface(IVRExtendedDisplay_Version.as_ref())?
                as isize,
            m_pVRSettings: vr_get_generic_interface(IVRSettings_Version.as_ref())? as isize,
            m_pVRApplications: vr_get_generic_interface(IVRApplications_Version.as_ref())? as isize,
            m_pVRTrackedCamera: vr_get_generic_interface(IVRTrackedCamera_Version.as_ref())?
                as isize,
            m_pVRScreenshots: vr_get_generic_interface(IVRScreenshots_Version.as_ref())? as isize,
            m_pVRDriverManager: vr_get_generic_interface(IVRDriverManager_Version.as_ref())?
                as isize,
            m_pVRInput: vr_get_generic_interface(IVRInput_Version.as_ref())? as isize,
            m_pVRIOBuffer: vr_get_generic_interface(IVRIOBuffer_Version.as_ref())? as isize,
            m_pVRSpatialAnchors: vr_get_generic_interface(IVRSpatialAnchors_Version.as_ref())?
                as isize,
        })
    } else {
        Err(err)
    }
}

unsafe fn vr_shutdown() {
    VR_ShutdownInternal();
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let gl_window = glutin::GlWindow::new(
        glutin::WindowBuilder::new()
            .with_title("Hello world!")
            .with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0)),
        glutin::ContextBuilder::new(),
        &events_loop,
    )
    .unwrap();

    unsafe { gl_window.make_current().unwrap() };

    let _ = gl::load_with(|s| gl_window.context().get_proc_address(s) as *const _);

    unsafe {
        println!("OpenGL version {}", get_string(gl::VERSION));
    }

    // === VR ===
    let vr_context = unsafe {
        let vr_context = vr_init(EVRApplicationType_VRApplication_Scene).unwrap_or_else(|err| {
            panic!(
                "{}",
                CStr::from_ptr(VR_GetVRInitErrorAsEnglishDescription(err))
                    .to_str()
                    .unwrap()
            );
        });
        vr_context
    };

    let vr_system;
    let vr_compositor;
    unsafe {
        vr_system = {
            let p = vr_context.m_pVRSystem as *const VR_IVRSystem_FnTable;
            if p == ptr::null_mut() {
                panic!("m_pVRSystem is null");
            }
            &*p
        };

        vr_compositor = {
            let p = vr_context.m_pVRCompositor as *const VR_IVRCompositor_FnTable;
            if p == ptr::null_mut() {
                panic!("m_pVRCompositor is null");
            }
            &*p
        }
    }
    // --- VR ---

    unsafe { gl_window.make_current().unwrap() };

    unsafe {
        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
        gl::CompileShader(vs);

        let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
        gl::CompileShader(fs);

        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        gl::UseProgram(program);

        let mut vb = mem::uninitialized();
        gl::GenBuffers(1, &mut vb);
        gl::BindBuffer(gl::ARRAY_BUFFER, vb);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(&VERTEX_DATA) as isize,
            VERTEX_DATA.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        let mut vao = mem::uninitialized();
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let pos_attrib = gl::GetAttribLocation(program, b"position\0".as_ptr() as *const _);
        let color_attrib = gl::GetAttribLocation(program, b"color\0".as_ptr() as *const _);
        gl::VertexAttribPointer(
            pos_attrib as u32,
            2,
            gl::FLOAT,
            0,
            5 * mem::size_of::<f32>() as i32,
            ptr::null(),
        );
        gl::VertexAttribPointer(
            color_attrib as u32,
            3,
            gl::FLOAT,
            0,
            5 * mem::size_of::<f32>() as i32,
            (2 * mem::size_of::<f32>()) as *const () as *const _,
        );
        gl::EnableVertexAttribArray(pos_attrib as u32);
        gl::EnableVertexAttribArray(color_attrib as u32);
    }

    // === VR ===

    let mut running = true;
    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => running = false,
                glutin::WindowEvent::Resized(logical_size) => {
                    let dpi_factor = gl_window.get_hidpi_factor();
                    gl_window.resize(logical_size.to_physical(dpi_factor));
                }
                _ => (),
            },
            _ => (),
        });

        // draw everything here
        unsafe {
            gl::ClearColor(0.6, 0.7, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        // === VR ===
        unsafe {
            let mut event: VREvent_t = mem::zeroed();
            while vr_system.PollNextEvent.unwrap()(&mut event, mem::size_of::<VREvent_t>() as u32) {
                let event_type = vr::EventType::from_u32(event.eventType).unwrap();

                println!(
                    "event {:?}, device {}, age {}",
                    event_type, event.trackedDeviceIndex, event.eventAgeSeconds
                );

                match event_type {
                    vr::EventType::TrackedDeviceActivated => {
                        println!("Device {} detached.", event.trackedDeviceIndex);
                    }
                    vr::EventType::TrackedDeviceUpdated => {}
                    _ => {}
                }
            }
        }
        // --- VR ---

        gl_window.swap_buffers().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(17));
    }

    // === VR ===
    unsafe {
        vr_shutdown();
    }
    // --- VR ---
}

static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 0.5, 0.0, 1.0, 0.0, 0.5, -0.5, 0.0, 0.0, 1.0,
];

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
attribute vec2 position;
attribute vec3 color;
varying vec3 v_color;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_color = color;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
varying vec3 v_color;
void main() {
    gl_FragColor = vec4(v_color, 1.0);
}
\0";
