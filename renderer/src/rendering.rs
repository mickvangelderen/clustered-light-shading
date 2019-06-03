use crate::convert::*;
use crate::gl_ext::*;
use crate::rendering;
use cgmath::*;
use gl_typed as gl;

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

pub const POINT_LIGHT_CAPACITY: u32 = 8;

macro_rules! constant_declaration {
    () => {
        r"
#define POINT_LIGHT_CAPACITY 8
"
    };
}

// Storage buffer bindings.

pub const GLOBAL_DATA_BINDING: u32 = 0;
pub const VIEW_DATA_BINDING: u32 = 1;
pub const MATERIAL_DATA_BINDING: u32 = 2;
pub const AO_SAMPLE_BUFFER_BINDING: u32 = 3;
pub const LIGHTING_BUFFER_BINDING: u32 = 4;
pub const CLS_BUFFER_BINDING: u32 = 5;

macro_rules! buffer_binding_declaration {
    () => {
        r"
#define GLOBAL_DATA_BINDING 0
#define VIEW_DATA_BINDING 1
#define MATERIAL_DATA_BINDING 2
#define AO_SAMPLE_BUFFER_BINDING 3
#define LIGHTING_BUFFER_BINDING 4
#define CLS_BUFFER_BINDING 5
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

#[derive(Debug, Copy, Clone)]
#[repr(C, align(256))]
pub struct GlobalData {
    pub light_pos_from_wld_to_cam: Matrix4<f32>,
    pub light_pos_from_cam_to_wld: Matrix4<f32>,

    pub light_pos_from_cam_to_clp: Matrix4<f32>,
    pub light_pos_from_clp_to_cam: Matrix4<f32>,

    pub light_pos_from_wld_to_clp: Matrix4<f32>,
    pub light_pos_from_clp_to_wld: Matrix4<f32>,

    pub time: f64,
    pub attenuation_mode: u32,
}

pub const GLOBAL_DATA_DECLARATION: &'static str = r"
#define ATTENUATION_MODE_STEP 1
#define ATTENUATION_MODE_LINEAR 2
#define ATTENUATION_MODE_PHYSICAL 3
#define ATTENUATION_MODE_INTERPOLATED 4
#define ATTENUATION_MODE_REDUCED 5
#define ATTENUATION_MODE_SMOOTH 6

layout(std140, binding = GLOBAL_DATA_BINDING) uniform GlobalData {
    mat4 light_pos_from_wld_to_cam;
    mat4 light_pos_from_cam_to_wld;

    mat4 light_pos_from_cam_to_clp;
    mat4 light_pos_from_clp_to_cam;

    mat4 light_pos_from_wld_to_clp;
    mat4 light_pos_from_clp_to_wld;

    double time;
    uint attenuation_mode;
};
";

#[derive(Debug, Copy, Clone)]
pub struct GlobalResources {
    buffer_name: gl::BufferName,
}

impl GlobalResources {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            GlobalResources {
                buffer_name: gl.create_buffer(),
            }
        }
    }

    #[inline]
    pub fn bind(&self, gl: &gl::Gl) {
        unsafe {
            gl.bind_buffer_base(gl::UNIFORM_BUFFER, GLOBAL_DATA_BINDING, self.buffer_name);
        }
    }

    #[inline]
    pub fn write(&self, gl: &gl::Gl, data: &GlobalData) {
        unsafe {
            gl.named_buffer_data(self.buffer_name, data.value_as_bytes(), gl::DYNAMIC_DRAW);
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, align(256))]
pub struct ViewData {
    pub pos_from_wld_to_cam: Matrix4<f32>,
    pub pos_from_cam_to_wld: Matrix4<f32>,

    pub pos_from_cam_to_clp: Matrix4<f32>,
    pub pos_from_clp_to_cam: Matrix4<f32>,

    pub pos_from_wld_to_clp: Matrix4<f32>,
    pub pos_from_clp_to_wld: Matrix4<f32>,

    pub light_dir_in_cam: Vector3<f32>,
    pub _pad0: f32,
}

pub const VIEW_DATA_DECLARATION: &'static str = r"
layout(std140, binding = VIEW_DATA_BINDING) uniform ViewData {
    mat4 pos_from_wld_to_cam;
    mat4 pos_from_cam_to_wld;

    mat4 pos_from_cam_to_clp;
    mat4 pos_from_clp_to_cam;

    mat4 pos_from_wld_to_clp;
    mat4 pos_from_clp_to_wld;

    vec3 light_dir_in_cam;
    float _view_data_pad0;
};
";

#[derive(Debug, Copy, Clone)]
pub struct ViewResources {
    buffer_name: gl::BufferName,
}

impl ViewResources {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            ViewResources {
                buffer_name: gl.create_buffer(),
            }
        }
    }

    #[inline]
    pub fn bind_index(&self, gl: &gl::Gl, index: usize) {
        unsafe {
            gl.bind_buffer_range(
                gl::UNIFORM_BUFFER,
                VIEW_DATA_BINDING,
                self.buffer_name,
                std::mem::size_of::<ViewData>() * index,
                std::mem::size_of::<ViewData>(),
            );
        }
    }

    #[inline]
    pub fn write_all(&self, gl: &gl::Gl, data: &[ViewData]) {
        unsafe {
            gl.named_buffer_data(self.buffer_name, data.slice_to_bytes(), gl::DYNAMIC_DRAW);
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, align(256))]
pub struct MaterialData {
    pub shininess: f32,
}

pub const MATERIAL_DATA_DECLARATION: &'static str = r"
layout(std140, binding = MATERIAL_DATA_BINDING) uniform MaterialData {
    float shininess;
};
";

#[derive(Debug, Copy, Clone)]
pub struct MaterialResources {
    buffer_name: gl::BufferName,
}

impl MaterialResources {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            MaterialResources {
                buffer_name: gl.create_buffer(),
            }
        }
    }

    #[inline]
    pub fn bind_index(&self, gl: &gl::Gl, index: usize) {
        unsafe {
            gl.bind_buffer_range(
                gl::UNIFORM_BUFFER,
                MATERIAL_DATA_BINDING,
                self.buffer_name,
                std::mem::size_of::<MaterialData>() * index,
                std::mem::size_of::<MaterialData>(),
            );
        }
    }

    #[inline]
    pub fn write_all(&self, gl: &gl::Gl, data: &[MaterialData]) {
        unsafe {
            gl.named_buffer_data(self.buffer_name, data.slice_to_bytes(), gl::DYNAMIC_DRAW);
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct CLSBufferHeader {
    pub dimensions: Vector4<u32>,
    pub pos_from_wld_to_cls: Matrix4<f32>,
    pub pos_from_cls_to_wld: Matrix4<f32>,
}

#[derive(Debug)]
pub struct CLSBuffer {
    pub header: CLSBufferHeader,
    pub body: Vec<[u32; 8]>,
}

pub const CLS_BUFFER_DECLARATION: &'static str = r"
layout(std430, binding = CLS_BUFFER_BINDING) buffer CLSBuffer {
    uvec4 cluster_dims;
    mat4 pos_from_wld_to_cls;
    mat4 pos_from_cls_to_wld;
    uint clusters[];
};
";

#[derive(Debug, Copy, Clone)]
pub struct CLSResources {
    buffer_name: gl::BufferName,
}

impl CLSResources {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            CLSResources {
                buffer_name: gl.create_buffer(),
            }
        }
    }

    #[inline]
    pub fn bind(&self, gl: &gl::Gl) {
        unsafe {
            gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, CLS_BUFFER_BINDING, self.buffer_name);
        }
    }

    #[inline]
    pub fn write(&self, gl: &gl::Gl, cls_buffer: &CLSBuffer) {
        unsafe {
            let header_bytes = cls_buffer.header.value_as_bytes();
            let body_bytes = cls_buffer.body.vec_as_bytes();
            let total_size = header_bytes.len() + body_bytes.len();
            gl.named_buffer_reserve(self.buffer_name, total_size, gl::STREAM_DRAW);
            gl.named_buffer_sub_data(self.buffer_name, 0, header_bytes);
            gl.named_buffer_sub_data(self.buffer_name, header_bytes.len(), body_bytes);
        }
    }
}

pub struct VSFSProgram {
    pub name: gl::ProgramName,
    vertex_shader_name: gl::ShaderName,
    fragment_shader_name: gl::ShaderName,
}

#[derive(Default)]
pub struct VSFSProgramUpdate {
    pub vertex_shader: Option<Vec<u8>>,
    pub fragment_shader: Option<Vec<u8>>,
}

impl VSFSProgram {
    pub fn update(&mut self, gl: &gl::Gl, update: &VSFSProgramUpdate) -> bool {
        let mut should_link = false;

        if let Some(ref bytes) = update.vertex_shader {
            self.vertex_shader_name
                .compile(
                    gl,
                    &[
                        rendering::COMMON_DECLARATION.as_bytes(),
                        rendering::GLOBAL_DATA_DECLARATION.as_bytes(),
                        rendering::VIEW_DATA_DECLARATION.as_bytes(),
                        rendering::CLS_BUFFER_DECLARATION.as_bytes(),
                        "#line 1 1\n".as_bytes(),
                        bytes.as_ref(),
                    ],
                )
                .unwrap_or_else(|e| {
                    eprintln!(
                        "In vertex shader: {}\nCompilation error:\n{}",
                        std::str::from_utf8(bytes.as_ref()).unwrap(),
                        e
                    )
                });
            should_link = true;
        }

        if let Some(ref bytes) = update.fragment_shader {
            self.fragment_shader_name
                .compile(
                    gl,
                    &[
                        rendering::COMMON_DECLARATION.as_bytes(),
                        rendering::GLOBAL_DATA_DECLARATION.as_bytes(),
                        rendering::VIEW_DATA_DECLARATION.as_bytes(),
                        rendering::CLS_BUFFER_DECLARATION.as_bytes(),
                        rendering::MATERIAL_DATA_DECLARATION.as_bytes(),
                        "#line 1 1\n".as_bytes(),
                        bytes.as_ref(),
                    ],
                )
                .unwrap_or_else(|e| {
                    eprintln!(
                        "In fragment shader: {}\nCompilation error:\n{}",
                        std::str::from_utf8(bytes.as_ref()).unwrap(),
                        e
                    )
                });
            should_link = true;
        }

        if should_link {
            self.name.link(gl).map(|_| true).unwrap_or_else(|e| {
                eprintln!("{} (program):\n{}", file!(), e);
                false
            })
        } else {
            false
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            let name = gl.create_program();

            let vertex_shader_name = gl.create_shader(gl::VERTEX_SHADER);
            gl.attach_shader(name, vertex_shader_name);

            let fragment_shader_name = gl.create_shader(gl::FRAGMENT_SHADER);
            gl.attach_shader(name, fragment_shader_name);

            VSFSProgram {
                name,
                vertex_shader_name,
                fragment_shader_name,
            }
        }
    }
}
