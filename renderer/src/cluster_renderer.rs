use crate::resources::Resources;
use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
}

pub const CLS_TO_CLP_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(0) };
pub const CLUSTER_DIMS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(1) };
pub const PASS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(2) };

pub struct Parameters<'a> {
    pub cluster_resources: &'a cluster_shading::ClusterResources,
    pub cluster_data: &'a cluster_shading::ClusterData,
    pub configuration: &'a configuration::ClusteredLightShading,
    pub cls_to_clp: Matrix4<f32>,
}

impl Renderer {
    pub fn render(&mut self, gl: &gl::Gl, params: &Parameters, world: &mut World, resources: &Resources) {
        unsafe {
            self.program.update(gl, world);
            if let ProgramName::Linked(program_name) = self.program.name {
                gl.use_program(program_name);
                gl.bind_vertex_array(resources.cluster_vao);

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::FRAGMENTS_PER_CLUSTER_BINDING,
                    params.cluster_resources.fragments_per_cluster_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::ACTIVE_CLUSTER_BINDING,
                    params.cluster_resources.active_cluster_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::LIGHT_COUNT_BINDING,
                    params.cluster_resources.light_count_buffer.name(),
                );

                gl.bind_buffer(
                    gl::DRAW_INDIRECT_BUFFER,
                    params.cluster_resources.draw_command_buffer.name(),
                );

                gl.uniform_matrix4f(CLS_TO_CLP_LOC, gl::MajorAxis::Column, params.cls_to_clp.as_ref());
                gl.uniform_3ui(CLUSTER_DIMS_LOC, params.cluster_data.dimensions.into());
                gl.uniform_1ui(PASS_LOC, 0);

                gl.draw_elements_indirect(gl::TRIANGLES, gl::UNSIGNED_INT, 0);

                gl.depth_mask(gl::FALSE);
                gl.enable(gl::BLEND);
                gl.blend_func(gl::SRC_ALPHA, gl::ONE);

                gl.uniform_1ui(PASS_LOC, 1);

                gl.draw_elements_indirect(gl::TRIANGLES, gl::UNSIGNED_INT, 0);

                gl.depth_mask(gl::TRUE);
                gl.disable(gl::BLEND);

                gl.unbind_vertex_array();
                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        Renderer {
            program: vs_fs_program(gl, world, "cluster_renderer.vert", "cluster_renderer.frag"),
        }
    }
}
