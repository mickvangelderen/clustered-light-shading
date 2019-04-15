use gl_typed as gl;

pub trait TextureName2DExt: Sized {
    fn new(gl: &gl::Gl) -> Option<Self>;
    fn update<'a>(&self, gl: &gl::Gl, update: Texture2DUpdate<'a>);
}

impl TextureName2DExt for gl::TextureName {
    fn new(gl: &gl::Gl) -> Option<Self> {
        unsafe {
            let mut names: [Option<gl::TextureName>; 1] = std::mem::uninitialized();
            gl.gen_textures(&mut names);
            names[0]
        }
    }

    #[inline]
    fn update<'a>(&self, gl: &gl::Gl, update: Texture2DUpdate<'a>) {
        unsafe {
            gl.bind_texture(gl::TEXTURE_2D, *self);

            if let Some(Texture2DUpdateData {
                format,
                width,
                height,
                bytes,
            }) = update.data
            {
                gl.tex_image_2d(
                    gl::TEXTURE_2D,
                    0,
                    format.internal_format(),
                    width as i32,
                    height as i32,
                    format.format(),
                    format.component_type(),
                    match bytes {
                        Some(slice) => slice.as_ptr() as *const std::ffi::c_void,
                        None => std::ptr::null(),
                    },
                );
            }

            if let Some(max_level) = update.max_level {
                gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, max_level as i32);
            }

            if let Some(min_filter) = update.min_filter {
                gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter);
            }

            if let Some(mag_filter) = update.mag_filter {
                gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter);
            }

            if let Some(wrap) = update.wrap_s {
                gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap);
            }

            if let Some(wrap) = update.wrap_t {
                gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap);
            }

            gl.unbind_texture(gl::TEXTURE_2D);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Texture2DUpdateData<'a> {
    pub format: TextureFormat,
    pub width: usize,
    pub height: usize,
    pub bytes: Option<&'a [u8]>,
}

#[derive(Debug, Copy, Clone)]
pub struct Texture2DUpdate<'a> {
    data: Option<Texture2DUpdateData<'a>>,
    max_level: Option<u32>,
    min_filter: Option<gl::TextureMinFilter>,
    mag_filter: Option<gl::TextureMagFilter>,
    wrap_s: Option<gl::TextureWrap>,
    wrap_t: Option<gl::TextureWrap>,
}

impl<'a> Texture2DUpdate<'a> {
    #[inline]
    pub fn new() -> Self {
        Texture2DUpdate {
            data: None,
            max_level: None,
            min_filter: None,
            mag_filter: None,
            wrap_s: None,
            wrap_t: None,
        }
    }

    #[inline]
    pub fn data(
        mut self,
        format: TextureFormat,
        width: usize,
        height: usize,
        bytes: Option<&'a [u8]>,
    ) -> Self {
        self.data = Some(Texture2DUpdateData {
            format,
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
}

#[derive(Debug, Copy, Clone)]
pub enum TextureFormat {
    RGBA8,
    RGB8,
}

impl TextureFormat {
    #[inline]
    pub fn internal_format(self) -> gl::InternalFormat {
        match self {
            TextureFormat::RGBA8 => gl::RGBA8.into(),
            TextureFormat::RGB8 => gl::RGB8.into(),
        }
    }

    #[inline]
    pub fn format(self) -> gl::Format {
        match self {
            TextureFormat::RGBA8 => gl::RGBA.into(),
            TextureFormat::RGB8 => gl::RGB.into(),
        }
    }

    #[inline]
    pub fn component_type(self) -> gl::ComponentFormat {
        match self {
            TextureFormat::RGBA8 => gl::UNSIGNED_BYTE.into(),
            TextureFormat::RGB8 => gl::UNSIGNED_BYTE.into(),
        }
    }
}
