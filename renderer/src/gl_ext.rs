use crate::*;

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
    wrap_r: Option<gl::TextureWrap>,
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
            wrap_r: None,
            max_anisotropy: None,
        }
    }

    #[inline]
    pub fn data(mut self, width: i32, height: i32, bytes: Option<&'a [u8]>) -> Self {
        self.data = Some(TextureUpdateData { width, height, bytes });
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
    pub fn wrap_r(mut self, wrap_r: gl::TextureWrap) -> Self {
        self.wrap_r = Some(wrap_r);
        self
    }

    #[inline]
    pub fn max_anisotropy(mut self, max_anisotropy: f32) -> Self {
        self.max_anisotropy = Some(max_anisotropy);
        self
    }
}

macro_rules! impl_texture_format {
    ($(($InternalFormat: ident, $Format: ident, $ComponentFormat: ident),)*) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Copy, Clone)]
        pub enum TextureFormat {
            $(
                $InternalFormat,
            )*
        }

        impl TextureFormat {
            #[inline]
            pub fn internal_format(self) -> gl::InternalFormat {
                match self {
                    $(
                        TextureFormat::$InternalFormat => gl::$InternalFormat.into(),
                    )*
                }
            }

            #[inline]
            pub fn format(self) -> gl::Format {
                match self {
                    $(
                        TextureFormat::$InternalFormat => gl::$Format.into(),
                    )*
                }
            }

            #[inline]
            pub fn component_type(self) -> gl::ComponentFormat {
                match self {
                    $(
                        TextureFormat::$InternalFormat => gl::$ComponentFormat.into(),
                    )*
                }
            }
        }

        $(
            impl From<gl::$InternalFormat> for TextureFormat {
                #[inline]
                fn from(_: gl::$InternalFormat) -> Self {
                    TextureFormat::$InternalFormat
                }
            }
        )*
    };
}

impl_texture_format! {
    (RGBA8, RGBA, UNSIGNED_BYTE),
    (RGB8, RGB, UNSIGNED_BYTE),
    (RG8, RG, UNSIGNED_BYTE),
    (R8, RED, UNSIGNED_BYTE),
    (RG8UI, RG_INTEGER, UNSIGNED_BYTE),
    (DEPTH24_STENCIL8, DEPTH_STENCIL, UNSIGNED_INT_24_8),
    (R11F_G11F_B10F, RGB, FLOAT),
    (RGBA16F, RGBA, FLOAT),
    (RGB16F, RGB, FLOAT),
    (RG16F, RG, FLOAT),
    (R16F, RED, FLOAT),
    (RGBA32F, RGBA, FLOAT),
    (RGB32F, RGB, FLOAT),
    (RG32F, RG, FLOAT),
    (R32F, RED, FLOAT),
}

pub struct Texture<Shape, Format> {
    name: gl::TextureName,
    shape: Shape,
    format: Format,
}

impl<Shape, Format> Texture<Shape, Format> {
    #[inline]
    pub fn name(&self) -> gl::TextureName {
        self.name
    }

    #[inline]
    pub fn drop(self, gl: &gl::Gl) {
        unsafe {
            gl.delete_texture(self.name);
        }
    }
}

impl<Shape, Format> Texture<Shape, Format>
where
    Shape: Copy + Into<gl::TextureTarget>,
    Format: Copy + Into<TextureFormat>,
{
    #[inline]
    pub fn new(gl: &gl::Gl, shape: Shape, format: Format) -> Self {
        unsafe {
            Texture {
                name: gl.create_texture(shape.into()),
                shape,
                format,
            }
        }
    }

    #[inline]
    pub fn update<'a>(&self, gl: &gl::Gl, update: TextureUpdate<'a>) {
        unsafe {
            gl.bind_texture(self.shape.into(), self.name());

            if let Some(TextureUpdateData { width, height, bytes }) = update.data {
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
                gl.texture_parameteri(self.name, gl::TEXTURE_MAX_LEVEL, max_level);
            }

            if let Some(min_filter) = update.min_filter {
                gl.texture_parameteri(self.name, gl::TEXTURE_MIN_FILTER, min_filter);
            }

            if let Some(mag_filter) = update.mag_filter {
                gl.texture_parameteri(self.name, gl::TEXTURE_MAG_FILTER, mag_filter);
            }

            if let Some(wrap) = update.wrap_s {
                gl.texture_parameteri(self.name, gl::TEXTURE_WRAP_S, wrap);
            }

            if let Some(wrap) = update.wrap_t {
                gl.texture_parameteri(self.name, gl::TEXTURE_WRAP_T, wrap);
            }

            if let Some(wrap) = update.wrap_r {
                gl.texture_parameteri(self.name, gl::TEXTURE_WRAP_R, wrap);
            }

            if let Some(max_anisotropy) = update.max_anisotropy {
                gl.texture_parameterf(self.name, gl::TEXTURE_MAX_ANISOTROPY, max_anisotropy);
            }

            gl.unbind_texture(self.shape.into());
        }
    }
}

pub trait BufferNameExt: Sized {
    fn new(gl: &gl::Gl) -> Self;
}

impl BufferNameExt for gl::BufferName {
    #[inline]
    fn new(gl: &gl::Gl) -> Self {
        unsafe { gl.create_buffer() }
    }
}

#[derive(Debug)]
pub enum ProgramName {
    Unlinked(gl::ProgramName),
    Linked(gl::ProgramName),
}

impl AsRef<gl::ProgramName> for ProgramName {
    fn as_ref(&self) -> &gl::ProgramName {
        match self {
            ProgramName::Unlinked(name) => name,
            ProgramName::Linked(name) => name,
        }
    }
}

impl ProgramName {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe { ProgramName::Unlinked(gl.create_program()) }
    }

    #[inline]
    pub fn attach<I>(&mut self, gl: &gl::Gl, names: I)
    where
        I: IntoIterator,
        I::Item: AsRef<gl::ShaderName>,
    {
        unsafe {
            for name in names.into_iter() {
                gl.attach_shader(*self.as_ref(), *name.as_ref());
            }
        }
    }

    #[inline]
    pub fn link(&mut self, gl: &gl::Gl) {
        unsafe {
            gl.link_program(*self.as_ref());
            let status = gl.get_programiv(*self.as_ref(), gl::LINK_STATUS);
            // Don't panic from here.
            let name = std::ptr::read(self.as_ref());
            std::ptr::write(
                self,
                match status {
                    gl::LinkStatus::Unlinked => ProgramName::Unlinked(name),
                    gl::LinkStatus::Linked => ProgramName::Linked(name),
                },
            );
        }
    }

    #[inline]
    pub fn log(&self, gl: &gl::Gl) -> String {
        unsafe { gl.get_program_info_log(*self.as_ref()) }
    }

    #[inline]
    pub fn is_linked(&self) -> bool {
        match self {
            ProgramName::Linked(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_unlinked(&self) -> bool {
        match self {
            ProgramName::Unlinked(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum ShaderName {
    Uncompiled(gl::ShaderName),
    Compiled(gl::ShaderName),
}

impl AsRef<gl::ShaderName> for ShaderName {
    fn as_ref(&self) -> &gl::ShaderName {
        match self {
            ShaderName::Uncompiled(name) => name,
            ShaderName::Compiled(name) => name,
        }
    }
}

impl ShaderName {
    #[inline]
    pub fn new<K>(gl: &gl::Gl, kind: K) -> Self
    where
        K: Into<gl::ShaderKind>,
    {
        unsafe { ShaderName::Uncompiled(gl.create_shader(kind.into())) }
    }

    #[inline]
    pub fn compile<I>(&mut self, gl: &gl::Gl, sources: I)
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        unsafe {
            gl.shader_source(*self.as_ref(), sources);
            gl.compile_shader(*self.as_ref());
            let status = gl.get_shaderiv(*self.as_ref(), gl::COMPILE_STATUS);
            // Don't panic from here.
            let name = std::ptr::read(self.as_ref());
            std::ptr::write(
                self,
                match status {
                    gl::CompileStatus::Uncompiled => ShaderName::Uncompiled(name),
                    gl::CompileStatus::Compiled => ShaderName::Compiled(name),
                },
            );
        }
    }

    #[inline]
    pub fn log(&self, gl: &gl::Gl) -> String {
        unsafe { gl.get_shader_info_log(*self.as_ref()) }
    }

    #[inline]
    pub fn is_compiled(&self) -> bool {
        match self {
            ShaderName::Compiled(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_uncompiled(&self) -> bool {
        match self {
            ShaderName::Uncompiled(_) => true,
            _ => false,
        }
    }
}

pub struct QueryPool {
    names: [gl::QueryName; 4],
}

impl QueryPool {
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            Self {
                names: [
                    gl.create_query(gl::TIMESTAMP),
                    gl.create_query(gl::TIMESTAMP),
                    gl.create_query(gl::TIMESTAMP),
                    gl.create_query(gl::TIMESTAMP),
                ],
            }
        }
    }

    pub fn instant(&self, gl: &gl::Gl, tick: u64) -> Option<NonZeroU64> {
        unsafe {
            let name = &self.names[tick as usize % self.names.len()];
            let result = gl.try_query_result_u64(name);
            gl.query_counter(name);
            result
        }
    }
}

// impl QueryTimer {
//     pub fn time(&self, gl: &gl::Gl, tick: u64, f: impl FnOnce()) -> Option<NonZeroU64> {
//         let result = self.begin(gl, tick);
//         f();
//         self.end(gl);
//         result
//     }

//     fn begin(&self, gl: &gl::Gl, tick: u64) -> Option<NonZeroU64> {
//         unsafe {
//             let name = self.names[usize::try_from(tick).unwrap() % self.names.len()];
//             let result = gl.try_query_result_u64(&name);
//             gl.begin_query(gl::TIME_ELAPSED, &name);
//             result
//         }
//     }

//     fn end(&self, gl: &gl::Gl) {
//         unsafe {
//             gl.end_query(gl::TIME_ELAPSED);
//         }
//     }
// }
