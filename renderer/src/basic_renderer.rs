use crate::*;

pub struct Renderer {
    pub opaque_program: rendering::Program,
    pub masked_program: rendering::Program,
    pub transparent_program: rendering::Program,
}

pub struct Parameters {
    pub mode: u32,
    pub main_resources_index: MainResourcesIndex,
    pub main_parameters_index: usize,
}

glsl_defines!(fixed_header {
    bindings: {
        // CLUSTER_FRAGMENT_COUNTS_BUFFER_BINDING = 0;
        CLUSTER_MAYBE_ACTIVE_CLUSTER_INDICES_BUFFER_BINDING = 1;
        // ACTIVE_CLUSTER_CLUSTER_INDICES_BUFFER_BINDING = 2;
        ACTIVE_CLUSTER_LIGHT_COUNTS_BUFFER_BINDING = 3;
        ACTIVE_CLUSTER_LIGHT_OFFSETS_BUFFER_BINDING = 4;
        LIGHT_BUFFER_BINDING = 5;
        // LIGHT_XYZR_BUFFER_BINDING = 6;
        // OFFSETS_BUFFER_BINDING = 7;
        // DRAW_COMMANDS_BUFFER_BINDING = 8;
        // COMPUTE_COMMANDS_BUFFER_BINDING = 9;
        INSTANCE_MATRICES_BUFFER_BINDING = 10;
        LIGHT_INDICES_BUFFER_BINDING = 11;
        CLUSTER_SPACE_BUFFER_BINDING = 12;
        // PROFILING_CLUSTER_BUFFER_BINDING = 13;

        BASIC_ATOMIC_BINDING = 0;

        NORMAL_SAMPLER_BINDING = 1;
        EMISSIVE_SAMPLER_BINDING = 2;
        AMBIENT_SAMPLER_BINDING = 3;
        DIFFUSE_SAMPLER_BINDING = 4;
        SPECULAR_SAMPLER_BINDING = 5;
    },
    uniforms: {
        CAM_POS_IN_LGT_LOC = 0;
    },
});

impl Context<'_> {
    pub fn render_main(&mut self, params: &Parameters) {
        let Context {
            ref gl,
            ref mut basic_renderer,
            ..
        } = *self;

        let main_resources = &self.main_resources_pool[params.main_resources_index];

        let profiler_index = self.profiling_context.start(gl, main_resources.basic_profiler);

        let main_parameters = &self.main_parameters_vec[params.main_parameters_index];
        let cam_pos_in_lgt = main_parameters.cam_pos_in_lgt;
        let cluster_resources_index = main_parameters.cluster_resources_index;

        unsafe {
            basic_renderer.opaque_program.update(&mut rendering_context!(self));
            basic_renderer.masked_program.update(&mut rendering_context!(self));
            basic_renderer.transparent_program.update(&mut rendering_context!(self));
            if let (&ProgramName::Linked(opaque_program), &ProgramName::Linked(masked_program), &ProgramName::Linked(transparent_program)) =
                (&basic_renderer.opaque_program.name, &basic_renderer.masked_program.name, &basic_renderer.transparent_program.name)
            {
                if let Some(cluster_resources_index) = cluster_resources_index {
                    let cluster_resources = &self.cluster_resources_pool[cluster_resources_index];

                    debug_assert_eq!(
                        RenderTechnique::Clustered,
                        self.shader_compiler.variables.render_technique
                    );

                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        CLUSTER_MAYBE_ACTIVE_CLUSTER_INDICES_BUFFER_BINDING,
                        cluster_resources.cluster_maybe_active_cluster_indices_buffer.name(),
                    );

                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        ACTIVE_CLUSTER_LIGHT_COUNTS_BUFFER_BINDING,
                        cluster_resources.active_cluster_light_counts_buffer.name(),
                    );

                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        ACTIVE_CLUSTER_LIGHT_OFFSETS_BUFFER_BINDING,
                        cluster_resources.active_cluster_light_offsets_buffer.name(),
                    );

                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        LIGHT_INDICES_BUFFER_BINDING,
                        cluster_resources.light_indices_buffer.name(),
                    );

                    gl.bind_buffer_base(
                        gl::UNIFORM_BUFFER,
                        CLUSTER_SPACE_BUFFER_BINDING,
                        cluster_resources.cluster_space_buffer.name(),
                    );
                }

                let draw_resources = &self.resources.draw_resources_pool[main_parameters.draw_resources_index];

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    INSTANCE_MATRICES_BUFFER_BINDING,
                    draw_resources.instance_matrices_buffer,
                );

                gl.bind_buffer(gl::DRAW_INDIRECT_BUFFER, draw_resources.draw_command_buffer);

                gl.bind_vertex_array(self.resources.scene_vao);

                let draw_counts = &draw_resources.draw_counts;
                let draw_offsets = &draw_resources.draw_offsets;

                for &(program, material_kind, profiler) in [
                    (
                        opaque_program,
                        resources::MaterialKind::Opaque,
                        main_resources.basic_opaque_profiler,
                    ),
                    (
                        masked_program,
                        resources::MaterialKind::Masked,
                        main_resources.basic_masked_profiler,
                    ),
                    (
                        transparent_program,
                        resources::MaterialKind::Transparent,
                        main_resources.basic_transparent_profiler,
                    ),
                ]
                .iter()
                {
                    let profiler_index = self.profiling_context.start(gl, profiler);

                    gl.use_program(program);

                    gl.uniform_3f(CAM_POS_IN_LGT_LOC, cam_pos_in_lgt.cast().unwrap().into());

                    if material_kind == resources::MaterialKind::Transparent {
                        gl.depth_mask(gl::WriteMask::Disabled);
                        gl.enable(gl::BLEND);
                        gl.blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                    }

                    for (material_index, material) in self
                        .resources
                        .materials
                        .iter()
                        .enumerate()
                        .filter(|(_, material)| material.kind == material_kind)
                    {
                        // Update material.
                        let textures = &self.resources.textures;
                        gl.bind_texture_unit(NORMAL_SAMPLER_BINDING, textures[material.normal_texture_index].name);
                        gl.bind_texture_unit(EMISSIVE_SAMPLER_BINDING, textures[material.emissive_texture_index].name);
                        gl.bind_texture_unit(AMBIENT_SAMPLER_BINDING, textures[material.ambient_texture_index].name);
                        gl.bind_texture_unit(DIFFUSE_SAMPLER_BINDING, textures[material.diffuse_texture_index].name);
                        gl.bind_texture_unit(SPECULAR_SAMPLER_BINDING, textures[material.specular_texture_index].name);

                        // Execute draw.
                        gl.multi_draw_elements_indirect(
                            gl::TRIANGLES,
                            gl::UNSIGNED_INT,
                            draw_offsets[material_index] as usize * std::mem::size_of::<DrawCommand>(),
                            draw_counts[material_index] as i32,
                            std::mem::size_of::<DrawCommand>() as i32,
                        );
                    }

                    if material_kind == resources::MaterialKind::Transparent {
                        gl.depth_mask(gl::WriteMask::Enabled);
                        gl.disable(gl::BLEND);
                    }

                    self.profiling_context.stop(gl, profiler_index);
                }

                gl.unuse_program();

                gl.unbind_vertex_array();
            }
        }

        self.profiling_context.stop(gl, profiler_index);
    }
}

impl Renderer {
    pub fn new(context: &mut RenderingContext) -> Self {
        fn basic_pass_header(kind: resources::MaterialKind) -> String {
            format!(
                "\
                #define BASIC_PASS_OPAQUE 1\n\
                #define BASIC_PASS_MASKED 2\n\
                #define BASIC_PASS_TRANSPARENT 3\n\
                #define BASIC_PASS {}\n\
                ",
                match kind {
                    resources::MaterialKind::Opaque => "BASIC_PASS_OPAQUE",
                    resources::MaterialKind::Masked => "BASIC_PASS_MASKED",
                    resources::MaterialKind::Transparent => "BASIC_PASS_TRANSPARENT",
                }
            )
        };

        let mut create_program = |kind: resources::MaterialKind| -> rendering::Program {
            vs_fs_program(
                context,
                "basic_renderer.vert",
                "basic_renderer.frag",
                format!("{}{}", fixed_header(), basic_pass_header(kind))
            )
        };

        Renderer {
            opaque_program: create_program(resources::MaterialKind::Opaque),
            masked_program: create_program(resources::MaterialKind::Masked),
            transparent_program: create_program(resources::MaterialKind::Transparent),
        }
    }
}
