// TODO: Kerning Pairs

use crate::*;

use std::path::PathBuf;

#[derive(Debug)]
pub struct Page {
    pub file_path: PathBuf,
    pub texture_name: gl::TextureName,
}

#[derive(Debug)]
pub struct Character {
    pub id: char,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub offset_x: i16,
    pub offset_y: i16,
    pub advance_x: i16,
    pub page: i8,
    pub channel: i8,
}

#[derive(Debug)]
pub struct Meta {
    // info block
    pub font_size: u16,
    pub info_bit_field: u8,
    pub char_set: u8,
    pub stretch_y: u16,
    pub super_sampling_level: i8,
    pub padding_py: u8,
    pub padding_px: u8,
    pub padding_ny: u8,
    pub padding_nx: u8,
    pub spacing_x: u8,
    pub spacing_y: u8,
    pub outline: u8,
    // font name
    pub font_name: String,
    // common block
    pub line_y: u16,
    pub base: u16,
    pub scale_x: u16,
    pub scale_y: u16,
    pub pages: u16,
    pub common_bit_field: u8,
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug)]
pub struct TextRenderingContext {
    file_path: PathBuf,
    meta: Meta,
    characters: Vec<Character>,
    pages: Vec<Page>,
}

impl TextRenderingContext {
    pub fn new(gl: &gl::Gl, path: impl Into<PathBuf>) -> Self {
        let file_path = path.into();
        let dir_path = file_path.parent().unwrap();

        let mut buffer = Vec::new();
        {
            let mut file = std::fs::File::open(&file_path).unwrap();
            std::io::Read::read_to_end(&mut file, &mut buffer).unwrap();
        }

        let bmfont = bmfont::BMFont::new(&buffer[..]);

        let meta = Meta {
            // info block
            font_size: bmfont.info.font_size.to_ne(),
            info_bit_field: bmfont.info.bit_field,
            char_set: bmfont.info.char_set,
            stretch_y: bmfont.info.stretch_y.to_ne(),
            super_sampling_level: bmfont.info.super_sampling_level,
            padding_py: bmfont.info.padding_py,
            padding_px: bmfont.info.padding_px,
            padding_ny: bmfont.info.padding_ny,
            padding_nx: bmfont.info.padding_nx,
            spacing_x: bmfont.info.spacing_x,
            spacing_y: bmfont.info.spacing_y,
            outline: bmfont.info.outline,
            // font name
            font_name: String::from(bmfont.font_name.to_str().unwrap()),
            // common block
            line_y: bmfont.common.line_y.to_ne(),
            base: bmfont.common.base.to_ne(),
            scale_x: bmfont.common.scale_x.to_ne(),
            scale_y: bmfont.common.scale_y.to_ne(),
            pages: bmfont.common.pages.to_ne(),
            common_bit_field: bmfont.common.bit_field,
            alpha: bmfont.common.alpha,
            red: bmfont.common.red,
            green: bmfont.common.green,
            blue: bmfont.common.blue,
        };

        let characters: Vec<Character> = bmfont
            .chars
            .iter()
            .map(|block| Character {
                id: std::char::from_u32(block.id.to_ne()).unwrap(),
                x: block.x.to_ne(),
                y: block.x.to_ne(),
                width: block.width.to_ne(),
                height: block.height.to_ne(),
                offset_x: block.offset_x.to_ne(),
                offset_y: block.offset_y.to_ne(),
                advance_x: block.advance_x.to_ne(),
                page: block.page,
                channel: block.channel,
            })
            .collect();

        let pages: Vec<Page> = bmfont
            .pages
            .iter()
            .map(|cstr| {
                let file_path = dir_path.join(cstr.to_str().unwrap());
                let texture_name = unsafe {
                    let name = gl.create_texture(gl::TEXTURE_2D);

                    let img = image::open(&file_path)
                        .expect("Failed to load image.")
                        .flipv()
                        .to_rgba();

                    gl.texture_storage_2d(
                        &name,
                        1,
                        gl::RGBA8,
                        img.width() as i32,
                        img.height() as i32,
                    );

                    gl.texture_sub_image_2d(
                        &name,
                        0,
                        0,
                        0,
                        img.width() as i32,
                        img.height() as i32,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        img.as_ptr() as *const std::ffi::c_void,
                    );

                    gl.texture_parameteri(name, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
                    gl.texture_parameteri(name, gl::TEXTURE_MAG_FILTER, gl::NEAREST);

                    name
                };
                Page { file_path, texture_name }
            })
            .collect();

        Self {
            file_path,
            meta,
            characters,
            pages,
        }
    }
}
