use crate::*;

pub struct Renderer {
    pub opaque_program: rendering::Program,
    pub masked_program: rendering::Program,
    pub transparent_program: rendering::Program,
}

pub struct Parameters {
    pub transparent_only: bool,
    pub draw_resources_index: usize,
    pub cluster_resources_index: usize,
    pub camera_resources_index: usize,
}

glsl_defines!(fixed_header {
    bindings: {
        CLUSTER_FRAGMENT_COUNTS_BUFFER_BINDING = 0;
        LIGHT_BUFFER_BINDING = 4;
        CLUSTER_SPACE_BUFFER_BINDING = 9;
        INSTANCE_MATRICES_BUFFER_BINDING = 10;

        DIFFUSE_SAMPLER_BINDING = 4;
    },
    uniforms: {},
});

impl Context<'_> {
    pub fn render_count_fragments(&mut self, params: &Parameters) {
        let Context {
            ref gl,
            ref mut count_fragments_renderer: renderer,
            ..
        } = *self;

        let cluster_resources = &self.cluster_resources_pool[cluster_resources_index];

        let profiler_index = self.profiling_context.start(gl, cluster_resources.count_fragments_profiler);

        unsafe {
            renderer.opaque_program.update(&mut rendering_context!(self));
            renderer.masked_program.update(&mut rendering_context!(self));
            renderer.transparent_program.update(&mut rendering_context!(self));
            if let (&ProgramName::Linked(opaque_program), &ProgramName::Linked(masked_program), &ProgramName::Linked(transparent_program)) =
                (&renderer.opaque_program.name, &renderer.transparent_program.name, &renderer.transparent_program.name)
            {

                debug_assert_eq!(
                    RenderTechnique::Clustered,
                    self.shader_compiler.variables.render_technique
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    CLUSTER_FRAGMENT_COUNTS_BUFFER,
                    cluster_resources.cluster_fragment_counts_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::UNIFORM_BUFFER,
                    CLUSTER_SPACE_BUFFER_BINDING,
                    cluster_resources.cluster_space_buffer.name(),
                );

                let draw_resources = &self.resources.draw_resources_pool[draw_resources_index];

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
                        cluster_resources..basic_opaque_profiler,
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
