use gl_typed as gl;
use renderer::profiling_by_value::*;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct WindowConfiguration {
    pub vsync: bool,
    pub rgb_bits: u8,
    pub alpha_bits: u8,
    pub srgb: bool,
    pub width: u32,
    pub height: u32,
}

fn create_window(
    event_loop: &mut glutin::EventsLoop,
    cfg: &WindowConfiguration,
) -> Result<glutin::GlWindow, glutin::CreationError> {
    // Jump through some hoops to ensure a physical size, which is
    // what I want in case I'm recording at a specific resolution.
    let dimensions = glutin::dpi::PhysicalSize::new(f64::from(cfg.width), f64::from(cfg.height))
        .to_logical(event_loop.get_primary_monitor().get_hidpi_factor());

    let mut gl_window = glutin::GlWindow::new(
        glutin::WindowBuilder::new()
            .with_title("VR Lab - Loading...")
            .with_dimensions(dimensions)
            .with_maximized(false),
        glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)))
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl_debug_flag(cfg!(debug_assertions))
            .with_vsync(cfg.vsync)
            // .with_multisampling(16)
            .with_pixel_format(cfg.rgb_bits, cfg.alpha_bits)
            .with_srgb(cfg.srgb)
            .with_double_buffer(Some(true)),
        &event_loop,
    )?;

    unsafe { glutin::GlContext::make_current(&mut gl_window).unwrap() };

    Ok(gl_window)
}

fn create_gl(gl_window: &mut glutin::GlWindow) -> gl::Gl {
    unsafe {
        let gl = gl::Gl::load_with(|s| glutin::GlContext::get_proc_address(gl_window.context(), s) as *const _);

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

        // if configuration.global.framebuffer_srgb {
        //     gl.enable(gl::FRAMEBUFFER_SRGB);
        // } else {
        //     gl.disable(gl::FRAMEBUFFER_SRGB);
        // }

        gl
    }
}

fn main() {
    let mut event_loop = glutin::EventsLoop::new();
    let mut gl_window = create_window(
        &mut event_loop,
        &WindowConfiguration {
            vsync: true,
            rgb_bits: 24,
            alpha_bits: 8,
            srgb: true,
            width: 1280,
            height: 720,
        },
    )
    .unwrap();
    let gl = create_gl(&mut gl_window);
    let mut profiler = MainProfiler::default();

    for run_index in 0..1 {
        {
            let context = run(RunContext {
                event_loop: &mut event_loop,
                gl_window: &mut gl_window,
                gl: &gl,
                profiler: profiler.begin_run(run_index),
            });
            profiler = context.profiler.end_run();
        }
    }
}

struct RunContext<'s> {
    pub event_loop: &'s mut glutin::EventsLoop,
    pub gl_window: &'s mut glutin::GlWindow,
    pub gl: &'s gl::Gl,
    pub profiler: RunProfiler,
}

fn run(context: RunContext) -> RunContext {
    let RunContext {
        event_loop,
        gl_window,
        gl,
        mut profiler,
    } = context;

    let mut running = true;
    let mut simulate_index = 0;
    let mut render_index = 0;

    loop {
        simulate(SimulateContext {
            event_loop,
            running: &mut running,
            simulate_index: &simulate_index,
        });
        simulate_index += 1;

        if false == running {
            break;
        }

        {
            let context = render(RenderContext {
                gl_window,
                gl,
                profiler: profiler.begin_frame(render_index),
            });
            profiler = context.profiler.end_frame();
            render_index += 1;
        }
    }

    RunContext {
        event_loop,
        gl_window,
        gl,
        profiler,
    }
}

struct SimulateContext<'s> {
    pub event_loop: &'s mut glutin::EventsLoop,
    pub running: &'s mut bool,
    pub simulate_index: &'s usize,
}

fn simulate(context: SimulateContext) {
    let SimulateContext {
        event_loop,
        running,
        simulate_index,
    } = context;

    event_loop.poll_events(|event| {
        use glutin::Event;
        match event {
            Event::WindowEvent { event, .. } => {
                use glutin::WindowEvent;
                match event {
                    WindowEvent::CloseRequested => *running = false,
                    // WindowEvent::HiDpiFactorChanged(val) => {
                    //     let win_size = self.win_size.to_logical(self.win_dpi);
                    //     self.win_dpi = val;
                    //     self.win_size = win_size.to_physical(self.win_dpi);
                    // }
                    // WindowEvent::Focused(val) => *focus
                    // WindowEvent::Resized(val) => {
                    //     self.win_size = val.to_physical(self.win_dpi);
                    // }
                    _ => (),
                }
            }
            // Event::DeviceEvent { event, .. } => {
            //     use glutin::DeviceEvent;
            //     match event {
            //         DeviceEvent::Key(keyboard_input) => {
            //             frame_events.push(FrameEvent::DeviceKey(keyboard_input));
            //             self.keyboard_state.update(keyboard_input);
            //         }
            //         DeviceEvent::Motion { axis, value } => {
            //             frame_events.push(FrameEvent::DeviceMotion { axis, value });
            //         }
            //         _ => (),
            //     }
            // }
            _ => (),
        }
    });
}

struct RenderContext<'s> {
    pub gl_window: &'s mut glutin::GlWindow,
    pub gl: &'s gl::Gl,
    pub profiler: FrameProfiler,
}

fn render(context: RenderContext) -> RenderContext {
    let RenderContext {
        gl_window,
        gl,
        profiler,
    } = context;

    unsafe {
        let mut x = profiler.frame_index();
        let r = (x % 256) as f32 / 255.0;
        x = x / 256;
        let g = (x % 256) as f32 / 255.0;
        x = x / 256;
        let b = (x % 256) as f32 / 255.0;
        let a = 1.0;
        gl.clear_color(r, g, b, a);
        gl.clear(gl::ClearFlag::COLOR_BUFFER | gl::ClearFlag::DEPTH_BUFFER);
    }

    gl_window.swap_buffers().unwrap();

    RenderContext {
        gl_window,
        gl,
        profiler,
    }
}
