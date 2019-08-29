pub use crate::*;

pub const LIGHT_COUNT_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::from_i32_unchecked(3) };

pub struct CountLightsProgram {
    pub program: rendering::Program,
}

impl CountLightsProgram {
    pub fn new(context: &mut RenderingContext) -> Self {
        let gl = &context.gl;
        Self {
            program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(&mut shader_compilation_context!(context), "cls/count_lights.comp"),
                )],
            ),
        }
    }
}
