// TODO: Kerning Pairs

use crate::*;

use std::path::PathBuf;

#[derive(Debug)]
pub struct Page {
    pub file_path: PathBuf,
    pub texture_name: gl::TextureName,
}

#[derive(Debug, Copy, Clone)]
pub struct GlyphMeta {
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

impl<'a> From<&'a bmfont::CharBlock> for GlyphMeta {
    fn from(block: &'a bmfont::CharBlock) -> Self {
        Self {
            x: block.x.to_ne(),
            y: block.y.to_ne(),
            width: block.width.to_ne(),
            height: block.height.to_ne(),
            offset_x: block.offset_x.to_ne(),
            offset_y: block.offset_y.to_ne(),
            advance_x: block.advance_x.to_ne(),
            page: block.page,
            channel: block.channel,
        }
    }
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
pub struct FontContext {
    pub file_path: PathBuf,
    pub meta: Meta,
    pub notdef: Option<GlyphMeta>,
    pub glyphs: Vec<(char, GlyphMeta)>,
    pub pages: Vec<Page>,
    pub vao: gl::VertexArrayName,
    pub vb: gl::BufferName,
    pub eb: gl::BufferName,
}

impl FontContext {
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

        let notdef: Option<GlyphMeta> = {
            // NOTE: AngelCode's bitmap font generator emits u32::MAX for the notdef glyph id.
            let mut notdefs = bmfont.chars.iter().filter_map(|block| match block.id.to_ne() {
                0 | std::u32::MAX => Some(GlyphMeta::from(block)),
                _ => None,
            });
            let notdef = notdefs.next();
            assert!(notdefs.next().is_none(), "Found more than one notdef glyphs!");
            notdef
        };

        let glyphs: Vec<(char, GlyphMeta)> = bmfont
            .chars
            .iter()
            .filter_map(|block| std::char::from_u32(block.id.to_ne()).map(|id| (id, GlyphMeta::from(block))))
            .collect();


        let texture_name = unsafe {
            let name = gl.create_texture(gl::TEXTURE_2D);
            gl.texture_storage_2d(
                &name,
                meta.pages as i32,
                gl::RGBA8,
                meta.scale_x as i32,
                meta.scale_y as i32,
            );
            gl.texture_parameteri(name, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
            gl.texture_parameteri(name, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
            name
        };

        assert_eq!(meta.pages as usize, bmfont.pages.len());

        let pages: Vec<Page> = bmfont
            .pages
            .iter()
            .enumerate()
            .map(|(page_index, &cstr)| {
                let file_path = dir_path.join(cstr.to_str().unwrap());

                let img = image::open(&file_path)
                    .expect("Failed to load image.")
                    .flipv()
                    .to_rgba();

                assert_eq!(meta.scale_x as u32, img.width());
                assert_eq!(meta.scale_y as u32, img.height());

                unsafe {
                    gl.texture_sub_image_2d(
                        &texture_name,
                        page_index as i32,
                        0,
                        0,
                        meta.scale_x as i32,
                        meta.scale_y as i32,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        img.as_ptr() as *const std::ffi::c_void,
                    );
                };
                Page {
                    file_path,
                    texture_name,
                }
            })
            .collect();

        let (vao, vb, eb) = unsafe {
            let vao = gl.create_vertex_array();
            let vb = gl.create_buffer();
            let eb = gl.create_buffer();

            let buffer_binding = gl::VertexArrayBufferBindingIndex::from_u32(0);

            // Set up attributes.
            gl.vertex_array_attrib_format(vao, rendering::VS_POS_IN_OBJ_LOC, 2, gl::FLOAT, false, 0);
            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_OBJ_LOC);
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_OBJ_LOC, buffer_binding);

            gl.vertex_array_attrib_format(
                vao,
                rendering::VS_POS_IN_TEX_LOC,
                2,
                gl::FLOAT,
                false,
                std::mem::size_of::<[f32; 2]>() as u32,
            );
            gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_TEX_LOC);
            gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_TEX_LOC, buffer_binding);

            // Bind buffers to vao.
            let stride = std::mem::size_of::<[f32; 4]>() as u32;
            gl.vertex_array_vertex_buffer(vao, buffer_binding, vb, 0, stride);
            gl.vertex_array_element_buffer(vao, eb);

            (vao, vb, eb)
        };

        Self {
            file_path,
            meta,
            notdef,
            glyphs,
            pages,
            vao,
            vb,
            eb,
        }
    }

    fn glyph(&self, target: char) -> Option<&GlyphMeta> {
        self.glyphs
            .iter()
            .find(|&&(c, _)| c == target)
            .map(|&(_, ref d)| d)
            .or_else(|| self.notdef.as_ref())
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct TextVertex {
    pub pos_in_obj: [f32; 2],
    pub pos_in_tex: [f32; 2],
}

pub struct TextBox {
    pub offset_x: i32,
    pub offset_y: i32,
    pub width: i32,
    pub height: i32,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub vertices: Vec<TextVertex>,
    pub indices: Vec<u32>,
}

impl TextBox {
    pub fn new(offset_x: i32, offset_y: i32, width: i32, height: i32) -> Self {
        TextBox {
            offset_x,
            offset_y,
            width,
            height,
            cursor_x: offset_x,
            cursor_y: offset_y + height,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cursor_x = self.offset_x;
        self.cursor_y = self.offset_y + self.height;
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn write(&mut self, context: &FontContext, string: &str) {
        for c in string.chars() {
            if c == '\n' {
                self.cursor_y -= context.meta.line_y as i32;
                self.cursor_x = self.offset_x;
                continue;
            }

            if let Some(glyph_meta) = context.glyph(c) {
                let obj_x0_i32 = self.cursor_x + glyph_meta.offset_x as i32;
                let obj_y1_i32 = self.cursor_y - glyph_meta.offset_y as i32;
                let obj_x0 = obj_x0_i32 as f32;
                let obj_y0 = (obj_y1_i32 - glyph_meta.height as i32) as f32;
                let obj_x1 = (obj_x0_i32 + glyph_meta.width as i32) as f32;
                let obj_y1 = obj_y1_i32 as f32;

                let sy = context.meta.scale_y as i32;
                let tex_x0 = glyph_meta.x as f32;
                let tex_y0 = (sy - glyph_meta.y as i32 - glyph_meta.height as i32) as f32;
                let tex_x1 = (glyph_meta.x + glyph_meta.width) as f32;
                let tex_y1 = (sy - glyph_meta.y as i32) as f32;

                let base_index = self.vertices.len() as u32;

                self.vertices.extend(
                    [
                        TextVertex {
                            pos_in_obj: [obj_x0, obj_y0],
                            pos_in_tex: [tex_x0, tex_y0],
                        },
                        TextVertex {
                            pos_in_obj: [obj_x1, obj_y0],
                            pos_in_tex: [tex_x1, tex_y0],
                        },
                        TextVertex {
                            pos_in_obj: [obj_x1, obj_y1],
                            pos_in_tex: [tex_x1, tex_y1],
                        },
                        TextVertex {
                            pos_in_obj: [obj_x0, obj_y1],
                            pos_in_tex: [tex_x0, tex_y1],
                        },
                    ]
                    .iter()
                    .copied(),
                );

                self.indices.extend(
                    [
                        base_index,
                        base_index + 1,
                        base_index + 2,
                        base_index + 2,
                        base_index + 3,
                        base_index,
                    ]
                    .iter()
                    .copied(),
                );

                self.cursor_x += glyph_meta.advance_x as i32;
            }
        }
    }
}
