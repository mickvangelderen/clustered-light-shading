use glutin::PossiblyCurrent;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct WindowConfiguration {
    pub vsync: bool,
    pub rgb_bits: u8,
    pub alpha_bits: u8,
    pub srgb: bool,
    pub width: u32,
    pub height: u32,
}

pub fn create_window(
    event_loop: &mut glutin::EventsLoop,
    cfg: &WindowConfiguration,
) -> Result<glutin::WindowedContext<PossiblyCurrent>, glutin::CreationError> {
    // Jump through some hoops to ensure a physical size, which is
    // what I want in case I'm recording at a specific resolution.
    let dimensions = glutin::dpi::PhysicalSize::new(f64::from(cfg.width), f64::from(cfg.height))
        .to_logical(event_loop.get_primary_monitor().get_hidpi_factor());
    let gl_window = glutin::ContextBuilder::new()
    .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)))
    .with_gl_profile(glutin::GlProfile::Core)
    .with_gl_debug_flag(cfg!(debug_assertions))
    .with_vsync(cfg.vsync)
    .with_pixel_format(cfg.rgb_bits, cfg.alpha_bits)
    .with_srgb(cfg.srgb)
    .with_double_buffer(Some(true))
    .build_windowed(
        glutin::WindowBuilder::new()
            .with_title("VR Lab - Loading...")
            .with_dimensions(dimensions)
            .with_maximized(false),
        event_loop,
    )?;

    let gl_window = unsafe { gl_window.make_current().unwrap() };

    Ok(gl_window)
}
