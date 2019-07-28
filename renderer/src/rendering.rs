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

// Constants.

pub const POINT_LIGHT_CAPACITY: u32 = 1000;

macro_rules! constant_declaration {
    () => {
        r"
#define POINT_LIGHT_CAPACITY 1000
"
    };
}

// Storage buffer bindings.

pub const GLOBAL_BUFFER_BINDING: u32 = 0;
pub const CAMERA_BUFFER_BINDING: u32 = 1;
pub const MATERIAL_BUFFER_BINDING: u32 = 2;
pub const LIGHT_BUFFER_BINDING: u32 = 3;
pub const TILE_BUFFER_BINDING: u32 = 4;

macro_rules! buffer_binding_declaration {
    () => {
        r"
#define GLOBAL_BUFFER_BINDING 0
#define CAMERA_BUFFER_BINDING 1
#define MATERIAL_BUFFER_BINDING 2
#define LIGHT_BUFFER_BINDING 3
#define TILE_BUFFER_BINDING 4
"
    };
}

// Attribute locations.

pub const VS_POS_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(0) };
pub const VS_POS_IN_TEX_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(1) };
pub const VS_NOR_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(2) };
pub const VS_TAN_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(3) };

macro_rules! attribute_location_declaration {
    () => {
        r"
#define VS_POS_IN_OBJ_LOC 0
#define VS_POS_IN_TEX_LOC 1
#define VS_NOR_IN_OBJ_LOC 2
#define VS_TAN_IN_OBJ_LOC 3
"
    };
}

pub const COMMON_DECLARATION: &'static str = concat!(
    capability_declaration!(),
    constant_declaration!(),
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

    pub fn update(&mut self, gl: &gl::Gl, world: &mut World) -> bool {
        let updated = self.entry_point.update(world);

        if updated {
            self.name.compile(
                gl,
                [
                    COMMON_DECLARATION,
                    CAMERA_BUFFER_DECLARATION,
                    crate::light::LIGHT_BUFFER_DECLARATION,
                    &self.entry_point.contents,
                ]
                .iter(),
            );

            if self.name.is_compiled() {
                let name = world.shader_compiler.memory.sources[self.entry_point.source_index]
                    .name
                    .to_str()
                    .unwrap();
                info!("Compiled {}.", name);
            } else {
                let log = self.name.log(gl);

                let log = world.gl_log_regex.replace_all(&log, |captures: &regex::Captures| {
                    let i: usize = captures[0].parse().unwrap();
                    world.shader_compiler.memory.sources[i].name.to_str().unwrap()
                });

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

    pub fn update(&mut self, gl: &gl::Gl, world: &mut World) -> bool {
        let updated = self
            .shaders
            .iter_mut()
            .fold(false, |updated, shader| shader.update(gl, world) || updated);

        if updated {
            self.name.link(gl);

            if self.name.is_linked() {
                // NOTE(mickvangelderen) EW!
                let names: String = self
                    .shaders
                    .iter()
                    .flat_map(|shader| {
                        std::iter::once(
                            world.shader_compiler.memory.sources[shader.entry_point.source_index]
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
                    let log = self.name.log(gl);

                    let log = world.gl_log_regex.replace_all(&log, |captures: &regex::Captures| {
                        let i: usize = captures[0].parse().unwrap();
                        world.shader_compiler.memory.sources[i].name.to_str().unwrap()
                    });

                    error!("Link error:\n{}", log);
                }
            }
        }

        updated
    }
}

/// Utility function to create a very common single file vertex and single file fragment shader.
pub fn vs_fs_program(gl: &gl::Gl, world: &mut World, vs: &'static str, fs: &'static str) -> Program {
    Program::new(
        gl,
        vec![
            Shader::new(gl, gl::VERTEX_SHADER, EntryPoint::new(world, vs)),
            Shader::new(gl, gl::FRAGMENT_SHADER, EntryPoint::new(world, fs)),
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

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct DrawCommand {
    pub count: u32,
    pub prim_count: u32,
    pub first_index: u32,
    pub base_vertex: u32,
    pub base_instance: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ComputeCommand {
    pub work_group_x: u32,
    pub work_group_y: u32,
    pub work_group_z: u32,
}

pub mod buffer_usage {
    use super::*;

    pub trait Variant {
        fn value() -> gl::BufferUsage;
    }

    pub enum Static {}
    pub enum Dynamic {}
    pub enum Stream {}

    impl Variant for Static {
        fn value() -> gl::BufferUsage {
            gl::STATIC_DRAW.into()
        }
    }

    impl Variant for Dynamic {
        fn value() -> gl::BufferUsage {
            gl::DYNAMIC_DRAW.into()
        }
    }

    impl Variant for Stream {
        fn value() -> gl::BufferUsage {
            gl::STREAM_DRAW.into()
        }
    }
}

pub struct Buffer<U> {
    name: gl::BufferName,
    byte_capacity: usize,
    usage: std::marker::PhantomData<U>,
}

impl<U> Buffer<U>
where
    U: buffer_usage::Variant,
{
    pub unsafe fn new(gl: &gl::Gl) -> Self {
        Self {
            name: gl.create_buffer(),
            byte_capacity: 0,
            usage: std::marker::PhantomData,
        }
    }

    pub unsafe fn name(&self) -> gl::BufferName {
        self.name
    }

    pub fn byte_capacity(&self) -> usize {
        self.byte_capacity
    }

    pub unsafe fn invalidate(&mut self, gl: &gl::Gl) {
        if self.byte_capacity > 0 {
            // Invalidate buffer using old capacity.
            gl.named_buffer_reserve(self.name, self.byte_capacity, U::value());
        }
    }

    pub unsafe fn ensure_capacity(&mut self, gl: &gl::Gl, byte_capacity: usize) {
        if self.byte_capacity >= byte_capacity {
            return;
        }

        gl.named_buffer_reserve(self.name, byte_capacity, U::value());
        self.byte_capacity = byte_capacity;
    }

    pub unsafe fn write(&mut self, gl: &gl::Gl, bytes: &[u8]) {
        debug_assert!(bytes.len() <= self.byte_capacity);
        gl.named_buffer_data(self.name, bytes, U::value());
    }

    pub unsafe fn write_at(&mut self, gl: &gl::Gl, bytes: &[u8], offset: usize) {
        debug_assert!(offset + bytes.len() <= self.byte_capacity);
        gl.named_buffer_sub_data(self.name, offset, bytes);
    }

    pub unsafe fn clear_0u32(&mut self, gl: &gl::Gl, byte_count: usize) {
        debug_assert!(byte_count <= self.byte_capacity);
        gl.clear_named_buffer_sub_data(self.name, gl::R32UI, 0, byte_count, gl::RED, gl::UNSIGNED_INT, None);
    }
}

impl<U> AsRef<gl::BufferName> for Buffer<U> {
    fn as_ref(&self) -> &gl::BufferName {
        &self.name
    }
}

// impl<U> std::ops::Deref for Buffer<U> {
//     type Target = gl::BufferName;

//     fn deref(&self) -> &Self::Target {
//         &self.name
//     }
// }

pub type StaticBuffer = Buffer<buffer_usage::Static>;
pub type DynamicBuffer = Buffer<buffer_usage::Dynamic>;
pub type StreamBuffer = Buffer<buffer_usage::Stream>;
