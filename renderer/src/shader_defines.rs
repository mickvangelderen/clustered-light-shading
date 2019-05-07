use gl_typed as gl;

pub const VS_POS_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(0) };
pub const VS_POS_IN_TEX_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(1) };
pub const VS_NOR_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(2) };
pub const VS_TAN_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(3) };

pub const VERSION: &'static [u8] = br"
#version 420 core
";

pub const DEFINES: &'static [u8] = br"
#define VS_POS_IN_OBJ_LOC 0
#define VS_POS_IN_TEX_LOC 1
#define VS_NOR_IN_OBJ_LOC 2
#define VS_TAN_IN_OBJ_LOC 3
";
