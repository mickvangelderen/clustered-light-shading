use gl_typed as gl;
use cgmath::*;

#[derive(Debug, Copy, Clone)]
pub struct ViewIndependentParameters {
    pub light_pos_from_wld_to_cam: Matrix4<f32>,
    pub light_pos_from_cam_to_wld: Matrix4<f32>,

    pub light_pos_from_cam_to_clp: Matrix4<f32>,
    pub light_pos_from_clp_to_cam: Matrix4<f32>,

    pub light_pos_from_wld_to_clp: Matrix4<f32>,
    pub light_pos_from_clp_to_wld: Matrix4<f32>,

    pub pos_from_wld_to_cls: Matrix4<f32>,
    pub pos_from_cls_to_wld: Matrix4<f32>,
}

#[derive(Debug, Default)]
pub struct ViewIndependentUniforms {
    pub light_pos_from_wld_to_cam_loc: gl::OptionUniformLocation,
    pub light_pos_from_cam_to_wld_loc: gl::OptionUniformLocation,

    pub light_pos_from_cam_to_clp_loc: gl::OptionUniformLocation,
    pub light_pos_from_clp_to_cam_loc: gl::OptionUniformLocation,

    pub light_pos_from_wld_to_clp_loc: gl::OptionUniformLocation,
    pub light_pos_from_clp_to_wld_loc: gl::OptionUniformLocation,

    pub pos_from_wld_to_cls_loc: gl::OptionUniformLocation,
    pub pos_from_cls_to_wld_loc: gl::OptionUniformLocation,
}

impl ViewIndependentUniforms {
    /// # Safety
    /// Assumes the correct program is bound.
    pub unsafe fn set(&self, gl: &gl::Gl, params: ViewIndependentParameters) {
        if let Some(loc) = self.light_pos_from_wld_to_cam_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.light_pos_from_wld_to_cam.as_ref());
        }

        if let Some(loc) = self.light_pos_from_cam_to_wld_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.light_pos_from_cam_to_wld.as_ref());
        }

        if let Some(loc) = self.light_pos_from_cam_to_clp_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.light_pos_from_cam_to_clp.as_ref());
        }

        if let Some(loc) = self.light_pos_from_clp_to_cam_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.light_pos_from_clp_to_cam.as_ref());
        }

        if let Some(loc) = self.light_pos_from_wld_to_clp_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.light_pos_from_wld_to_clp.as_ref());
        }

        if let Some(loc) = self.light_pos_from_clp_to_wld_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.light_pos_from_clp_to_wld.as_ref());
        }

        if let Some(loc) = self.pos_from_wld_to_cls_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.pos_from_wld_to_cls.as_ref());
        }

        if let Some(loc) = self.pos_from_cls_to_wld_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.pos_from_cls_to_wld.as_ref());
        }
    }

    pub unsafe fn update(&mut self, gl: &gl::Gl, program_name: gl::ProgramName) {
        self.light_pos_from_wld_to_cam_loc = get_uniform_location!(gl, program_name, "light_pos_from_wld_to_cam");
        self.light_pos_from_cam_to_wld_loc = get_uniform_location!(gl, program_name, "light_pos_from_cam_to_wld");

        self.light_pos_from_cam_to_clp_loc = get_uniform_location!(gl, program_name, "light_pos_from_cam_to_clp");
        self.light_pos_from_clp_to_cam_loc = get_uniform_location!(gl, program_name, "light_pos_from_clp_to_cam");

        self.light_pos_from_wld_to_clp_loc = get_uniform_location!(gl, program_name, "light_pos_from_wld_to_clp");
        self.light_pos_from_clp_to_wld_loc = get_uniform_location!(gl, program_name, "light_pos_from_clp_to_wld");
        self.pos_from_wld_to_cls_loc = get_uniform_location!(gl, program_name, "pos_from_wld_to_cls");
        self.pos_from_cls_to_wld_loc = get_uniform_location!(gl, program_name, "pos_from_cls_to_wld");
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ViewDependentParameters {
    pub pos_from_wld_to_cam: Matrix4<f32>,
    pub pos_from_cam_to_wld: Matrix4<f32>,

    pub pos_from_cam_to_clp: Matrix4<f32>,
    pub pos_from_clp_to_cam: Matrix4<f32>,

    pub pos_from_wld_to_clp: Matrix4<f32>,
    pub pos_from_clp_to_wld: Matrix4<f32>,
}

#[derive(Debug, Default)]
pub struct ViewDependentUniforms {
    pub pos_from_wld_to_cam_loc: gl::OptionUniformLocation,
    pub pos_from_cam_to_wld_loc: gl::OptionUniformLocation,

    pub pos_from_cam_to_clp_loc: gl::OptionUniformLocation,
    pub pos_from_clp_to_cam_loc: gl::OptionUniformLocation,

    pub pos_from_wld_to_clp_loc: gl::OptionUniformLocation,
    pub pos_from_clp_to_wld_loc: gl::OptionUniformLocation,
}

impl ViewDependentUniforms {
    /// # Safety
    /// Assumes the correct program is bound.
    pub unsafe fn set(&self, gl: &gl::Gl, params: ViewDependentParameters) {
        if let Some(loc) = self.pos_from_wld_to_cam_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.pos_from_wld_to_cam.as_ref());
        }

        if let Some(loc) = self.pos_from_cam_to_wld_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.pos_from_cam_to_wld.as_ref());
        }

        if let Some(loc) = self.pos_from_cam_to_clp_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.pos_from_cam_to_clp.as_ref());
        }

        if let Some(loc) = self.pos_from_clp_to_cam_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.pos_from_clp_to_cam.as_ref());
        }

        if let Some(loc) = self.pos_from_wld_to_clp_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.pos_from_wld_to_clp.as_ref());
        }

        if let Some(loc) = self.pos_from_clp_to_wld_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.pos_from_clp_to_wld.as_ref());
        }
    }

    pub unsafe fn update(&mut self, gl: &gl::Gl, program_name: gl::ProgramName) {
        self.pos_from_wld_to_cam_loc = get_uniform_location!(gl, program_name, "pos_from_wld_to_cam");
        self.pos_from_cam_to_wld_loc = get_uniform_location!(gl, program_name, "pos_from_cam_to_wld");

        self.pos_from_cam_to_clp_loc = get_uniform_location!(gl, program_name, "pos_from_cam_to_clp");
        self.pos_from_clp_to_cam_loc = get_uniform_location!(gl, program_name, "pos_from_clp_to_cam");

        self.pos_from_wld_to_clp_loc = get_uniform_location!(gl, program_name, "pos_from_wld_to_clp");
        self.pos_from_clp_to_wld_loc = get_uniform_location!(gl, program_name, "pos_from_clp_to_wld");
    }
}
