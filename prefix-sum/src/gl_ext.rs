use crate::*;

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
