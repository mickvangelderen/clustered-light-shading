#[macro_use]
extern crate log;

use gl_typed as gl;
use std::io;
use std::path::{Path, PathBuf};

pub trait FormatExt {
    fn to_gl_internal_format(&self) -> gl::InternalFormat;
}

macro_rules! impl_format {
    ($(
        ($Variant: ident, $gl_internal_format: expr),
    )*) => {
        impl FormatExt for dds::Format {#[inline] fn to_gl_internal_format(&self) -> gl::InternalFormat {
                match *self {
                    $(
                        Self::$Variant => $gl_internal_format.into(),
                    )*
                }
            }
        }
    }
}

impl_format! {
    (BC1_UNORM_RGB,  gl::COMPRESSED_RGB_S3TC_DXT1_EXT ),
    (BC1_UNORM_RGBA, gl::COMPRESSED_RGBA_S3TC_DXT1_EXT),
    (BC2_UNORM_RGBA, gl::COMPRESSED_RGBA_S3TC_DXT3_EXT),
    (BC3_UNORM_RGBA, gl::COMPRESSED_RGBA_S3TC_DXT5_EXT),
    (BC4_UNORM_R,    gl::COMPRESSED_RED_RGTC1         ),
    (BC4_SNORM_R,    gl::COMPRESSED_SIGNED_RED_RGTC1  ),
    (BC5_UNORM_RG,   gl::COMPRESSED_RG_RGTC2          ),
    (BC5_SNORM_RG,   gl::COMPRESSED_SIGNED_RG_RGTC2   ),
}

fn load_texture(gl: &gl::Gl, file_path: impl AsRef<Path>) -> io::Result<gl::TextureName> {
    let file_path = file_path.as_ref();

    info!("Reading {:?}...", &file_path);
    let file = std::fs::File::open(file_path).unwrap();
    let mut reader = std::io::BufReader::new(file);
    let dds = dds::File::parse(&mut reader).unwrap();
    info!("Reading {:?} done.", &file_path);

    unsafe {
        let name = gl.create_texture(gl::TEXTURE_2D);
        // NOTE(mickvangelderen): No dsa for compressed textures??
        gl.bind_texture(gl::TEXTURE_2D, name);
        for (layer_index, layer) in dds.layers.iter().enumerate() {
            info!(
                "Uploading {:?} layer {}/{}...",
                &file_path,
                layer_index + 1,
                dds.layers.len()
            );
            gl.compressed_tex_image_2d(
                gl::TEXTURE_2D,
                layer_index as i32,
                dds.header.pixel_format.to_gl_internal_format(),
                layer.width as i32,
                layer.height as i32,
                &dds.bytes[layer.byte_offset..(layer.byte_offset + layer.byte_count)],
            );
        }
        info!("Uploading {:?} done.", &file_path);

        Ok(name)
    }
}

fn load_textures(gl: &gl::Gl, dir_path: impl AsRef<Path>) -> io::Result<Vec<gl::TextureName>> {
    Ok(std::fs::read_dir(dir_path)?
        .into_iter()
        .filter_map(|entry| {
            let file_path = entry.unwrap().path();
            match file_path.extension().and_then(std::ffi::OsStr::to_str) {
                Some("dds") => Some(load_texture(gl, file_path).unwrap()),
                _ => None,
            }
        })
        .collect())
}

fn find_dxt1_texture(dir_path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    Ok(std::fs::read_dir(dir_path)?
        .into_iter()
        .flat_map(|entry| {
            let file_path = entry.unwrap().path();
            match file_path.extension().and_then(std::ffi::OsStr::to_str) {
                Some("dds") => {
                    let file = std::fs::File::open(&file_path).unwrap();
                    let mut reader = std::io::BufReader::new(file);
                    let dds = dds::File::parse(&mut reader).unwrap();
                    if let dds::Format::BC1_UNORM_RGB = dds.header.pixel_format {
                        Some(PathBuf::from(&file_path))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .collect())
}

fn main() {
    env_logger::init();

    for file_path in find_dxt1_texture("resources/bistro/Textures").unwrap().iter() {
        let file = std::fs::File::open(&file_path).unwrap();
        let mut reader = std::io::BufReader::new(file);
        let dds = dds::File::parse(&mut reader).unwrap();

        assert_eq!(dds::Format::BC1_UNORM_RGB, dds.header.pixel_format);

        for (layer_index, layer) in dds.layers.iter().enumerate().skip(1).take(1) {
            let w = layer.width as usize;
            let h = layer.height as usize;

            let block_counts = ((w + 3) / 4, (h + 3) / 4);

            let blocks = unsafe {
                assert_eq!(
                    block_counts.0 * block_counts.1 * std::mem::size_of::<dds::dxt1::Block>(),
                    layer.byte_count
                );

                std::slice::from_raw_parts(
                    dds.bytes[layer.byte_offset..(layer.byte_offset + layer.byte_count)].as_ptr()
                        as *const dds::dxt1::Block,
                    block_counts.0 * block_counts.1,
                )
            };

            let mut pixels: Vec<[f32; 3]> = (0..(w * h)).into_iter().map(|_| [0.0; 3]).collect();

            for by in 0..block_counts.1 {
                for bx in 0..block_counts.0 {
                    let block_rgb_f32 = blocks[by * block_counts.0 + bx].to_rgb_f32();
                    for ly in 0..4 {
                        let gy = by * 4 + ly;
                        if gy >= h {
                            continue;
                        }
                        for lx in 0..4 {
                            let gx = bx * 4 + lx;
                            if gx >= w {
                                continue;
                            }

                            pixels[gy * w + gx] = block_rgb_f32[ly][lx];
                        }
                    }
                }
            }

            let out_file_path: PathBuf = [
                file_path.parent().unwrap(),
                Path::new(&format!(
                    "{}.{}.ppm",
                    file_path.file_stem().unwrap().to_str().unwrap(),
                    layer_index
                )),
            ]
            .iter()
            .collect();

            dbg!(&out_file_path);
            use std::io::Write;
            let out_file = std::fs::File::create(&out_file_path).unwrap();
            let mut writer = std::io::BufWriter::new(out_file);
            write!(&mut writer, "P6 {} {} 255\n", w, h).unwrap();

            for gy in 0..h {
                for gx in 0..w {
                    let pixel = &pixels[gy * w + gx];
                    writer
                        .write_all(&[
                            (pixel[0] * 255.0 + 0.5) as u8,
                            (pixel[1] * 255.0 + 0.5) as u8,
                            (pixel[2] * 255.0 + 0.5) as u8,
                        ])
                        .unwrap();
                }
            }
        }
    }
}

#[allow(unused)]
fn old_main() {
    env_logger::init();

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

    let _textures = load_textures(&gl, "resources/bistro/Textures").unwrap();

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
