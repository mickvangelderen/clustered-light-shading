use crate::convert::*;
use crate::gl_ext::*;
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
#[repr(C)]
pub struct GlobalData {
    pub light_pos_from_wld_to_cam: Matrix4<f32>,
    pub light_pos_from_cam_to_wld: Matrix4<f32>,

    pub light_pos_from_cam_to_clp: Matrix4<f32>,
    pub light_pos_from_clp_to_cam: Matrix4<f32>,

    pub light_pos_from_wld_to_clp: Matrix4<f32>,
    pub light_pos_from_clp_to_wld: Matrix4<f32>,

    pub pos_from_wld_to_cls: Matrix4<f32>,
    pub pos_from_cls_to_wld: Matrix4<f32>,

    pub time: f32,
    pub _pad0: [f32; 3],
}

pub const GLOBAL_DATA_DECLARATION: &'static str = r"
layout(std140, binding = GLOBAL_DATA_BINDING) uniform GlobalData {
    mat4 light_pos_from_wld_to_cam;
    mat4 light_pos_from_cam_to_wld;

    mat4 light_pos_from_cam_to_clp;
    mat4 light_pos_from_clp_to_cam;

    mat4 light_pos_from_wld_to_clp;
    mat4 light_pos_from_clp_to_wld;

    mat4 pos_from_wld_to_cls;
    mat4 pos_from_cls_to_wld;

    float time;
    float _global_data_pad0[3];
};
";

#[derive(Debug, Copy, Clone)]
pub struct GlobalResources {
    buffer_name: gl::BufferName,
}

impl GlobalResources {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        let buffer_name = gl::BufferName::new_unwrap(gl);
        GlobalResources { buffer_name }
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
#[repr(C)]
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
        let buffer_name = gl::BufferName::new_unwrap(gl);
        ViewResources { buffer_name }
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
#[repr(C)]
pub struct MaterialData {
    pub shininess: f32,
    pub _pad0: [f32; 3],
}

pub const MATERIAL_DATA_DECLARATION: &'static str = r"
layout(std140, binding = MATERIAL_DATA_BINDING) uniform MaterialData {
    float shininess;
    float _material_data_pad0[3];
};
";

#[derive(Debug, Copy, Clone)]
pub struct MaterialResources {
    buffer_name: gl::BufferName,
}

impl MaterialResources {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        let buffer_name = gl::BufferName::new_unwrap(gl);
        MaterialResources { buffer_name }
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
