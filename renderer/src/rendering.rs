use crate::*;

// Capabilities.

macro_rules! capability_declaration {
    () => {
        r"
#version 430 core
#extension GL_ARB_gpu_shader5 : enable
#extension GL_NV_gpu_shader5 : enable
"
    };
}

// Storage buffer bindings.

pub const CAMERA_BUFFER_BINDING: u32 = 1;

macro_rules! buffer_binding_declaration {
    () => {
        r"
#define CAMERA_BUFFER_BINDING 1
"
    };
}

// Attribute locations.

pub const VS_POS_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::from_i32_unchecked(0) };
pub const VS_POS_IN_TEX_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::from_i32_unchecked(1) };
pub const VS_NOR_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::from_i32_unchecked(2) };

macro_rules! attribute_location_declaration {
    () => {
        r"
#define VS_POS_IN_OBJ_LOC 0
#define VS_POS_IN_TEX_LOC 1
#define VS_NOR_IN_OBJ_LOC 2
"
    };
}

pub const COMMON_DECLARATION: &'static str = concat!(
    capability_declaration!(),
    buffer_binding_declaration!(),
    attribute_location_declaration!(),
);

#[derive(Debug)]
#[repr(C, align(256))]
pub struct CameraBuffer {
    pub wld_to_cam: Matrix4<f32>,
    pub cam_to_wld: Matrix4<f32>,

    pub cam_to_clp: Matrix4<f32>,
    pub clp_to_cam: Matrix4<f32>,

    pub cam_pos_in_lgt: Vector4<f32>,
}

pub const CAMERA_BUFFER_DECLARATION: &'static str = r"
layout(std140, binding = CAMERA_BUFFER_BINDING) uniform CameraBuffer {
    mat4 wld_to_cam;
    mat4 cam_to_wld;

    mat4 cam_to_clp;
    mat4 clp_to_cam;

    vec4 cam_pos_in_lgt;
};
";

#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
#[repr(u32)]
pub enum LightSpace {
    Wld = 1,
    Hmd = 2,
    Cam = 3,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
#[repr(u32)]
pub enum RenderTechnique {
    Naive = 1,
    Tiled = 2,
    Clustered = 3,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
#[repr(u32)]
pub enum AttenuationMode {
    Step = 1,
    Linear = 2,
    Physical = 3,
    Interpolated = 4,
    Reduced = 5,
    Smooth = 6,
}

pub struct RenderingContext<'a> {
    pub gl: &'a gl::Gl,
    pub resource_dir: &'a Path,
    pub current: &'a mut incremental::Current,
    pub shader_compiler: &'a mut ShaderCompiler,
}

pub struct Shader {
    name: ShaderName,
    entry_point: EntryPoint,
}

impl Shader {
    pub fn new(gl: &gl::Gl, kind: impl Into<gl::ShaderKind>, entry_point: EntryPoint) -> Self {
        Self {
            name: ShaderName::new(gl, kind.into()),
            entry_point,
        }
    }

    pub fn update(&mut self, context: &mut RenderingContext) -> bool {
        let updated = self.entry_point.update(&mut shader_compilation_context!(context));
        let RenderingContext {
            ref gl,
            ref shader_compiler,
            ..
        } = *context;

        if updated {
            self.name.compile(
                gl,
                [
                    COMMON_DECLARATION,
                    CAMERA_BUFFER_DECLARATION,
                    crate::light::LIGHT_BUFFER_DECLARATION,
                    &self.entry_point.fixed_header,
                    &self.entry_point.contents,
                ]
                .iter(),
            );

            if self.name.is_compiled() {
                let name = shader_compiler.memory.sources[self.entry_point.source_index]
                    .name
                    .to_str()
                    .unwrap();
                info!("Compiled {}.", name);
            } else {
                let log = context.shader_compiler.process_log(&self.name.log(gl));
                error!("Compile error:\n{}", log);
            }
        }

        updated
    }
}

pub struct Program {
    shaders: Vec<Shader>,
    pub name: ProgramName,
}

impl Program {
    pub fn new(gl: &gl::Gl, shaders: Vec<Shader>) -> Self {
        let mut program_name = ProgramName::new(gl);

        program_name.attach(gl, shaders.iter().map(|shader| &shader.name));

        Self {
            shaders,
            name: program_name,
        }
    }

    pub fn update(&mut self, context: &mut RenderingContext) -> bool {
        let updated = self
            .shaders
            .iter_mut()
            .fold(false, |updated, shader| shader.update(context) || updated);

        let RenderingContext {
            ref gl,
            ref shader_compiler,
            ..
        } = *context;

        if updated {
            self.name.link(gl);

            if self.name.is_linked() {
                // NOTE(mickvangelderen) EW!
                let names: String = self
                    .shaders
                    .iter()
                    .flat_map(|shader| {
                        std::iter::once(
                            shader_compiler.memory.sources[shader.entry_point.source_index]
                                .name
                                .to_str()
                                .unwrap(),
                        )
                        .chain(std::iter::once(", "))
                    })
                    .collect();
                info!("Linked [{}].", &names[0..names.len() - 2]);
            } else {
                // Don't repeat messages spewed by shader already.
                if self.shaders.iter().all(|shader| shader.name.is_compiled()) {
                    let log = shader_compiler.process_log(&self.name.log(gl));
                    error!("Link error:\n{}", log);
                }
            }
        }

        updated
    }
}

/// Utility function to create a very common single file vertex and single file fragment shader.
pub fn vs_fs_program(
    context: &mut RenderingContext,
    vs: &'static str,
    fs: &'static str,
    fixed_header: String,
) -> Program {
    let gl = &context.gl;
    Program::new(
        &gl,
        vec![
            Shader::new(
                &gl,
                gl::VERTEX_SHADER,
                EntryPoint::new(&mut shader_compilation_context!(context), vs, fixed_header.clone()),
            ),
            Shader::new(
                gl,
                gl::FRAGMENT_SHADER,
                EntryPoint::new(&mut shader_compilation_context!(context), fs, fixed_header),
            ),
        ],
    )
}

#[derive(Debug, Copy, Clone)]
pub struct BufferPoolIndex(usize);

#[derive(Debug)]
pub struct BufferPool {
    buffers: Vec<gl::BufferName>,
    unused: BufferPoolIndex,
}

impl BufferPool {
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            unused: BufferPoolIndex(0),
        }
    }

    pub fn unused(&mut self, gl: &gl::Gl) -> BufferPoolIndex {
        let index = self.unused;

        self.unused.0 += 1;

        if self.buffers.len() < self.unused.0 {
            unsafe {
                self.buffers.push(gl.create_buffer());
            }

            debug_assert_eq!(self.buffers.len(), self.unused.0);
        }

        index
    }

    pub fn reset(&mut self, gl: &gl::Gl) {
        // Free up unused memory. Should only happen occasionally.
        while self.unused.0 < self.buffers.len() {
            unsafe {
                // Can unwrap since unused can't be less than 0.
                let buffer_name = self.buffers.pop().unwrap();
                gl.delete_buffer(buffer_name);
            }
        }

        self.unused.0 = 0;
    }

    pub fn drop(&mut self, gl: &gl::Gl) {
        while let Some(buffer_name) = self.buffers.pop() {
            unsafe {
                gl.delete_buffer(buffer_name);
            }
        }
    }
}

impl std::ops::Index<BufferPoolIndex> for BufferPool {
    type Output = gl::BufferName;

    fn index(&self, index: BufferPoolIndex) -> &Self::Output {
        &self.buffers[index.0]
    }
}
