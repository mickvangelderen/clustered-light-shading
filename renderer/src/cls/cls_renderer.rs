use crate::*;

pub struct Renderer {
    pub count_fragments_program: rendering::Program,
    pub frag_count_hist_program: rendering::Program,
    pub compact_clusters_0_program: rendering::Program,
    pub compact_clusters_1_program: rendering::Program,
    pub compact_clusters_2_program: rendering::Program,
    pub count_lights_program: rendering::Program,
    pub light_count_hist_program: rendering::Program,
    pub compact_light_counts_0_program: rendering::Program,
    pub compact_light_counts_1_program: rendering::Program,
    pub compact_light_counts_2_program: rendering::Program,
    pub assign_lights_program: rendering::Program,
}

glsl_defines!(fixed_header {
    bindings: {
        CLUSTER_FRAGMENT_COUNTS_BUFFER_BINDING = 0;
        CLUSTER_MAYBE_ACTIVE_CLUSTER_INDICES_BUFFER_BINDING = 1;
        ACTIVE_CLUSTER_CLUSTER_INDICES_BUFFER_BINDING = 2;
        ACTIVE_CLUSTER_LIGHT_COUNTS_BUFFER_BINDING = 3;
        ACTIVE_CLUSTER_LIGHT_OFFSETS_BUFFER_BINDING = 4;
        LIGHT_XYZR_BUFFER_BINDING = 5;
        OFFSETS_BUFFER_BINDING = 6;
        DRAW_COMMANDS_BUFFER_BINDING = 7;
        COMPUTE_COMMANDS_BUFFER_BINDING = 8;
        LIGHT_INDICES_BUFFER_BINDING = 9;
        CLUSTER_SPACE_BUFFER_BINDING = 10;
        PROFILING_CLUSTER_BUFFER_BINDING = 11;
    },
    uniforms: {
        DEPTH_SAMPLER_LOC = 0;
        FB_DIMS_LOC = 1;
        CLP_TO_WLD_LOC = 2;
        ITEM_COUNT_LOC = 3;
        LIGHT_COUNT_LOC = 4;
    },
});

impl Renderer {
    pub fn new(context: &mut RenderingContext) -> Self {
        let gl = context.gl;
        let mut shader_compilation_context = shader_compilation_context!(context);

        Renderer {
            count_fragments_program: rendering::Program::new(
                context.gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(
                        &mut shader_compilation_context,
                        "cls/count_fragments.comp",
                        fixed_header(),
                    ),
                )],
            ),
            frag_count_hist_program: rendering::Program::new(
                context.gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(
                        &mut shader_compilation_context,
                        "cls/frag_count_hist.comp",
                        fixed_header(),
                    ),
                )],
            ),
            compact_clusters_0_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(
                        &mut shader_compilation_context,
                        "cls/compact_clusters_0.comp",
                        fixed_header(),
                    ),
                )],
            ),
            compact_clusters_1_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(
                        &mut shader_compilation_context,
                        "cls/compact_clusters_1.comp",
                        fixed_header(),
                    ),
                )],
            ),
            compact_clusters_2_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(
                        &mut shader_compilation_context,
                        "cls/compact_clusters_2.comp",
                        fixed_header(),
                    ),
                )],
            ),
            count_lights_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(&mut shader_compilation_context, "cls/count_lights.comp", fixed_header()),
                )],
            ),
            light_count_hist_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(
                        &mut shader_compilation_context,
                        "cls/light_count_hist.comp",
                        fixed_header(),
                    ),
                )],
            ),
            compact_light_counts_0_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(
                        &mut shader_compilation_context,
                        "cls/compact_light_counts_0.comp",
                        fixed_header(),
                    ),
                )],
            ),
            compact_light_counts_1_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(
                        &mut shader_compilation_context,
                        "cls/compact_light_counts_1.comp",
                        fixed_header(),
                    ),
                )],
            ),
            compact_light_counts_2_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(
                        &mut shader_compilation_context,
                        "cls/compact_light_counts_2.comp",
                        fixed_header(),
                    ),
                )],
            ),
            assign_lights_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(
                        &mut shader_compilation_context,
                        "cls/assign_lights.comp",
                        fixed_header(),
                    ),
                )],
            ),
        }
    }
}
