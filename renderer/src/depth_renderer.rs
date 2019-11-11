use crate::*;

pub struct Renderer {
    pub opaque_program: rendering::Program,
    pub masked_program: rendering::Program,
}

glsl_defines!(fixed_header {
    bindings: {
        // CLUSTER_MAYBE_ACTIVE_CLUSTER_INDICES_BUFFER_BINDING = 1;
        // ACTIVE_CLUSTER_LIGHT_COUNTS_BUFFER_BINDING = 2;
        // ACTIVE_CLUSTER_LIGHT_OFFSETS_BUFFER_BINDING = 3;
        // LIGHT_BUFFER_BINDING = 4;
        // LIGHT_INDICES_BUFFER_BINDING = 8;
        // CLUSTER_SPACE_BUFFER_BINDING = 9;
        INSTANCE_MATRICES_BUFFER_BINDING = 10;

        // BASIC_ATOMIC_BINDING = 0;

        // NORMAL_SAMPLER_BINDING = 1;
        // EMISSIVE_SAMPLER_BINDING = 2;
        // AMBIENT_SAMPLER_BINDING = 3;
        DIFFUSE_SAMPLER_BINDING = 4;
        // SPECULAR_SAMPLER_BINDING = 5;
    },
    uniforms: {
        // OBJ_TO_WLD_LOC = 0;
        // SHININESS_LOC = 6;
    },
});

impl Context<'_> {
    pub fn render_depth(&mut self) {
        let Context {
            ref gl,
            ref resources,
            ref mut depth_renderer,
            ..
        } = *self;
        unsafe {
            depth_renderer.opaque_program.update(&mut rendering_context!(self));
            depth_renderer.masked_program.update(&mut rendering_context!(self));
            if let (&ProgramName::Linked(opaque_program), &ProgramName::Linked(masked_program)) =
                (&depth_renderer.opaque_program.name, &depth_renderer.masked_program.name)
            {
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    INSTANCE_MATRICES_BUFFER_BINDING,
                    resources.draw_resources.instance_matrices_buffer,
                );

                gl.bind_buffer(
                    gl::DRAW_INDIRECT_BUFFER,
                    resources.draw_resources.draw_command_buffer,
                );

                gl.bind_vertex_array(resources.scene_vao);

                let draw_counts = &resources.draw_resources.draw_counts;
                let draw_offsets = &resources.draw_resources.draw_offsets;

                for &(program, has_alpha) in [(opaque_program, false), (masked_program, true)].iter() {
                    gl.use_program(program);
                    for (material_index, material) in resources.materials.iter().enumerate() {
                        if resources.textures[material.diffuse_texture_index as usize].has_alpha != has_alpha
                            || draw_counts[material_index] == 0
                        {
                            continue;
                        }

                        if has_alpha {
                            // Update material.
                            let textures = &resources.textures;
                            gl.bind_texture_unit(
                                DIFFUSE_SAMPLER_BINDING,
                                textures[material.diffuse_texture_index].name,
                            );
                        }

                        // Execute draw.
                        gl.multi_draw_elements_indirect(
                            gl::TRIANGLES,
                            gl::UNSIGNED_INT,
                            draw_offsets[material_index] as usize * std::mem::size_of::<DrawCommand>(),
                            draw_counts[material_index] as i32,
                            std::mem::size_of::<DrawCommand>() as i32,
                        );
                    }
                }
                gl.unuse_program();

                gl.unbind_vertex_array();
            }
        }
    }
}

impl Renderer {
    pub fn new(context: &mut RenderingContext) -> Self {
        Renderer {
            opaque_program: vs_fs_program(
                context,
                "depth_renderer.vert",
                "depth_renderer.frag",
                format!(
                    "{}\
                     #define BASIC_PASS_OPAQUE 1\n\
                     #define BASIC_PASS_MASKED 2\n\
                     #define BASIC_PASS BASIC_PASS_OPAQUE\n\
                     ",
                    fixed_header()
                ),
            ),
            masked_program: vs_fs_program(
                context,
                "depth_renderer.vert",
                "depth_renderer.frag",
                format!(
                    "{}\
                     #define BASIC_PASS_OPAQUE 1\n\
                     #define BASIC_PASS_MASKED 2\n\
                     #define BASIC_PASS BASIC_PASS_MASKED\n\
                     ",
                    fixed_header()
                ),
            ),
        }
    }
}
