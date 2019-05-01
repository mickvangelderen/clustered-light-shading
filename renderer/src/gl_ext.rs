use gl_typed as gl;

#[derive(Debug, Copy, Clone)]
pub struct TextureUpdateData<'a> {
    pub width: i32,
    pub height: i32,
    pub bytes: Option<&'a [u8]>,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureUpdate<'a> {
    data: Option<TextureUpdateData<'a>>,
    max_level: Option<u32>,
    min_filter: Option<gl::TextureMinFilter>,
    mag_filter: Option<gl::TextureMagFilter>,
    wrap_s: Option<gl::TextureWrap>,
    wrap_t: Option<gl::TextureWrap>,
    max_anisotropy: Option<f32>,
}

impl<'a> TextureUpdate<'a> {
    #[inline]
    pub fn new() -> Self {
        TextureUpdate {
            data: None,
            max_level: None,
            min_filter: None,
            mag_filter: None,
            wrap_s: None,
            wrap_t: None,
            max_anisotropy: None,
        }
    }

    #[inline]
    pub fn data(mut self, width: i32, height: i32, bytes: Option<&'a [u8]>) -> Self {
        self.data = Some(TextureUpdateData {
            width,
            height,
            bytes,
        });
        self
    }

    #[inline]
    pub fn max_level(mut self, max_level: u32) -> Self {
        self.max_level = Some(max_level);
        self
    }

    #[inline]
    pub fn min_filter(mut self, min_filter: gl::TextureMinFilter) -> Self {
        self.min_filter = Some(min_filter);
        self
    }

    #[inline]
    pub fn mag_filter(mut self, mag_filter: gl::TextureMagFilter) -> Self {
        self.mag_filter = Some(mag_filter);
        self
    }

    #[inline]
    pub fn wrap_s(mut self, wrap_s: gl::TextureWrap) -> Self {
        self.wrap_s = Some(wrap_s);
        self
    }

    #[inline]
    pub fn wrap_t(mut self, wrap_t: gl::TextureWrap) -> Self {
        self.wrap_t = Some(wrap_t);
        self
    }

    #[inline]
    pub fn max_anisotropy(mut self, max_anisotropy: f32) -> Self {
        self.max_anisotropy = Some(max_anisotropy);
        self
    }
}

impl From<gl::symbols::Rgba8> for TextureFormat {
    #[inline]
    fn from(_: gl::symbols::Rgba8) -> Self {
        TextureFormat::Rgba8
    }
}

impl From<gl::symbols::Rgb8> for TextureFormat {
    #[inline]
    fn from(_: gl::symbols::Rgb8) -> Self {
        TextureFormat::Rgb8
    }
}

impl From<gl::symbols::Depth24Stencil8> for TextureFormat {
    #[inline]
    fn from(_: gl::symbols::Depth24Stencil8) -> Self {
        TextureFormat::Depth24Stencil8
    }
}

impl From<gl::symbols::R11fG11fB10f> for TextureFormat {
    #[inline]
    fn from(_: gl::symbols::R11fG11fB10f) -> Self {
        TextureFormat::R11fG11fB10f
    }
}

impl From<gl::symbols::Rg8ui> for TextureFormat {
    #[inline]
    fn from(_: gl::symbols::Rg8ui) -> Self {
        TextureFormat::Rg8ui
    }
}

impl From<gl::symbols::Rg32f> for TextureFormat {
    #[inline]
    fn from(_: gl::symbols::Rg32f) -> Self {
        TextureFormat::Rg32f
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TextureFormat {
    Rgba8,
    Rgb8,
    Depth24Stencil8,
    R11fG11fB10f,
    Rg8ui,
    Rg32f,
}

impl TextureFormat {
    #[inline]
    pub fn internal_format(self) -> gl::InternalFormat {
        match self {
            TextureFormat::Rgba8 => gl::RGBA8.into(),
            TextureFormat::Rgb8 => gl::RGB8.into(),
            TextureFormat::Depth24Stencil8 => gl::DEPTH24_STENCIL8.into(),
            TextureFormat::R11fG11fB10f => gl::R11F_G11F_B10F.into(),
            TextureFormat::Rg8ui => gl::RG8UI.into(),
            TextureFormat::Rg32f => gl::RG32F.into(),
        }
    }
    #[inline]
    pub fn format(self) -> gl::Format {
        match self {
            TextureFormat::Rgba8 => gl::RGBA.into(),
            TextureFormat::Rgb8 => gl::RGB.into(),
            TextureFormat::Depth24Stencil8 => gl::DEPTH_STENCIL.into(),
            TextureFormat::R11fG11fB10f => gl::RGB.into(),
            TextureFormat::Rg8ui => gl::RG_INTEGER.into(),
            TextureFormat::Rg32f => gl::RG.into(),
        }
    }

    #[inline]
    pub fn component_type(self) -> gl::ComponentFormat {
        match self {
            TextureFormat::Rgba8 => gl::UNSIGNED_BYTE.into(),
            TextureFormat::Rgb8 => gl::UNSIGNED_BYTE.into(),
            TextureFormat::Depth24Stencil8 => gl::UNSIGNED_INT_24_8.into(),
            TextureFormat::R11fG11fB10f => gl::FLOAT.into(),
            TextureFormat::Rg8ui => gl::UNSIGNED_BYTE.into(),
            TextureFormat::Rg32f => gl::FLOAT.into(),
        }
    }
}

pub struct Texture<Shape, Format> {
    name: gl::TextureName,
    shape: Shape,
    format: Format,
}

impl<Shape, Format> Texture<Shape, Format> {
    #[inline]
    pub fn new(gl: &gl::Gl, shape: Shape, format: Format) -> Option<Self> {
        unsafe {
            let mut names: [Option<gl::TextureName>; 1] = std::mem::uninitialized();
            gl.gen_textures(&mut names);
            names[0].map(|name| Texture {
                name,
                shape,
                format,
            })
        }
    }

    #[inline]
    pub fn name(&self) -> gl::TextureName {
        self.name
    }

    #[inline]
    pub fn drop(self, gl: &gl::Gl) {
        unsafe {
            let mut names = [Some(self.name)];
            gl.delete_textures(&mut names);
        }
    }
}

// FIXME: WORK WITH DIFFERENT DIMENSIONS
impl<Shape, Format> Texture<Shape, Format>
where
    Shape: Copy + Into<gl::TextureTargetGE2D>,
    Format: Copy + Into<TextureFormat>,
{
    #[inline]
    pub fn update<'a>(&self, gl: &gl::Gl, update: TextureUpdate<'a>) {
        unsafe {
            gl.bind_texture(self.shape.into(), self.name());

            if let Some(TextureUpdateData {
                width,
                height,
                bytes,
            }) = update.data
            {
                gl.tex_image_2d(
                    self.shape.into(),
                    0,
                    self.format.into().internal_format(),
                    width,
                    height,
                    self.format.into().format(),
                    self.format.into().component_type(),
                    match bytes {
                        Some(slice) => slice.as_ptr() as *const std::ffi::c_void,
                        None => std::ptr::null(),
                    },
                );
            }

            if let Some(max_level) = update.max_level {
                gl.tex_parameter_i(self.shape.into(), gl::TEXTURE_MAX_LEVEL, max_level as i32);
            }

            if let Some(min_filter) = update.min_filter {
                gl.tex_parameter_i(self.shape.into(), gl::TEXTURE_MIN_FILTER, min_filter);
            }

            if let Some(mag_filter) = update.mag_filter {
                gl.tex_parameter_i(self.shape.into(), gl::TEXTURE_MAG_FILTER, mag_filter);
            }

            if let Some(wrap) = update.wrap_s {
                gl.tex_parameter_i(self.shape.into(), gl::TEXTURE_WRAP_S, wrap);
            }

            if let Some(wrap) = update.wrap_t {
                gl.tex_parameter_i(self.shape.into(), gl::TEXTURE_WRAP_T, wrap);
            }

            if let Some(max_anisotropy) = update.max_anisotropy {
                gl.tex_parameter_f(self.shape.into(), gl::TEXTURE_MAX_ANISOTROPY, max_anisotropy);
            }

            gl.unbind_texture(self.shape.into());
        }
    }
}
