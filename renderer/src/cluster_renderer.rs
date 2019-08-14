use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
}

pub const CLS_TO_CLP_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(0) };
pub const CLUSTER_DIMS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(1) };
pub const PASS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(2) };

pub struct Parameters {
    pub cluster_resources_index: ClusterResourcesIndex,
    pub cls_to_clp: Matrix4<f32>,
}

impl Context {
    pub fn render_debug_clusters(&mut self, params: &Parameters) {
        unsafe {
            let Self {
                ref gl,
                ref resources,
                ref cluster_resources_pool,
                ref cluster_data_vec,
                ref mut cluster_renderer,
                ..
            } = *self;

            cluster_renderer.program.update(&mut rendering_context!(self));

            let cluster_resources = &cluster_resources_pool[params.cluster_resources_index];
            let cluster_data = &cluster_data_vec[params.cluster_resources_index.to_usize()];

            if let ProgramName::Linked(program_name) = cluster_renderer.program.name {
                gl.use_program(program_name);
                gl.bind_vertex_array(resources.cluster_vao);

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::CLUSTER_FRAGMENT_COUNTS_BINDING,
                    cluster_resources.cluster_fragment_counts_buffer.name(),
                );

                // gl.bind_buffer_base(
                //     gl::SHADER_STORAGE_BUFFER,
                //     cls_renderer::CLUSTER_METAS_BINDING,
                //     cluster_resources.cluster_metas_buffer.name(),
                // );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::ACTIVE_CLUSTER_INDICES_BINDING,
                    cluster_resources.active_cluster_indices_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::ACTIVE_CLUSTER_LIGHT_COUNTS_BINDING,
                    cluster_resources.active_cluster_light_counts_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::ACTIVE_CLUSTER_LIGHT_OFFSETS_BINDING,
                    cluster_resources.active_cluster_light_offsets_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::LIGHT_INDICES_BINDING,
                    cluster_resources.light_indices_buffer.name(),
                );

                gl.bind_buffer(gl::DRAW_INDIRECT_BUFFER, cluster_resources.draw_command_buffer.name());

                gl.uniform_matrix4f(CLS_TO_CLP_LOC, gl::MajorAxis::Column, params.cls_to_clp.as_ref());
                gl.uniform_3ui(CLUSTER_DIMS_LOC, cluster_data.dimensions.into());
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
}

impl Renderer {
    pub fn new(context: &mut RenderingContext) -> Self {
        Renderer {
            program: vs_fs_program(context, "cluster_renderer.vert", "cluster_renderer.frag"),
        }
    }
}
