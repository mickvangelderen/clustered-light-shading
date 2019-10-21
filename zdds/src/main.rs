use std::io;
use std::path::Path;

pub mod dds;
pub mod num;

use dds::*;
use gl_typed as gl;

fn load_texture(gl: &gl::Gl, file_path: impl AsRef<Path>) -> io::Result<gl::TextureName> {
    let file = std::fs::File::open(file_path).unwrap();
    let mut reader = std::io::BufReader::new(file);
    let raw_file = RawFile::parse(&mut reader).unwrap();

    unsafe {
        let name = gl.create_texture(gl::TEXTURE_2D);
        // NOTE(mickvangelderen): No dsa for compressed textures??
        gl.bind_texture(gl::TEXTURE_2D, name);
        for (layer_index, layer) in raw_file.layers.iter().enumerate() {
            let internal_format: gl::InternalFormat = match raw_file.header.pixel_format.four_cc {
                dds::FOURCC_DXT1 => gl::COMPRESSED_RGBA_S3TC_DXT1_EXT.into(),
                dds::FOURCC_DXT3 => gl::COMPRESSED_RGBA_S3TC_DXT3_EXT.into(),
                dds::FOURCC_DXT5 => gl::COMPRESSED_RGBA_S3TC_DXT5_EXT.into(),
                other => panic!("Unsupported four_cc: {:?}.", other),
            };
            gl.compressed_tex_image_2d(
                gl::TEXTURE_2D,
                layer_index as i32,
                internal_format,
                layer.width as i32,
                layer.height as i32,
                &raw_file.bytes[layer.byte_offset..(layer.byte_offset + layer.byte_count)],
            );
        }

        Ok(name)
    }
}

fn load_textures(gl: &gl::Gl, dir_path: impl AsRef<Path>) -> io::Result<Vec<gl::TextureName>> {
    Ok(std::fs::read_dir(dir_path)?.into_iter().filter_map(|entry| {
        let file_path = entry.unwrap().path();
        match file_path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("dds") => Some(load_texture(gl, file_path).unwrap()),
            _ => None,
        }
    }).collect())
}

fn main() {
    let mut event_loop = glutin::EventsLoop::new();

    let mut window = create_window(
        &mut event_loop,
        &WindowConfiguration {
            vsync: true,
            rgb_bits: 24,
            alpha_bits: 8,
            srgb: false,
            width: 1280,
            height: 720,
        },
    )
    .unwrap();

    let gl = create_gl(
        &mut window,
        &GlConfiguration {
            framebuffer_srgb: false,
        },
    );

    let textures = load_textures(&gl, "resources/bistro/Textures").unwrap();

    let mut running = true;

    while running {
        event_loop.poll_events(|event| {
            use glutin::Event;
            match event {
                Event::WindowEvent { event, .. } => {
                    use glutin::WindowEvent;
                    match event {
                        WindowEvent::CloseRequested => running = false,
                        // WindowEvent::HiDpiFactorChanged(val) => {
                        //     let size = self.win_size.to_logical(self.win_dpi);
                        //     self.win_dpi = val;
                        //     self.win_size = size.to_physical(val);
                        // }
                        // WindowEvent::Focused(val) => self.focus = val,
                        // WindowEvent::Resized(val) => {
                        //     self.win_size = val.to_physical(self.win_dpi);
                        // }
                        _ => {}
                    }
                }
                // Event::DeviceEvent { event, .. } => {
                //     use glutin::DeviceEvent;
                //     match event {
                //         DeviceEvent::Key(keyboard_input) => {
                //             frame_events.push(FrameEvent::DeviceKey(keyboard_input));
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

        window.swap_buffers().unwrap();
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct GlConfiguration {
    pub framebuffer_srgb: bool,
}

pub fn create_gl(gl_window: &glutin::GlWindow, cfg: &GlConfiguration) -> gl::Gl {
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

        if cfg.framebuffer_srgb {
            gl.enable(gl::FRAMEBUFFER_SRGB);
        } else {
            gl.disable(gl::FRAMEBUFFER_SRGB);
        }

        gl
    }
}
