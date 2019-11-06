use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
    pub draw_commands_buffer: DynamicBuffer,
    pub instance_matrices_buffer: DynamicBuffer,
}

pub struct Parameters {
    pub mode: u32,
    pub cluster_resources_index: Option<ClusterResourcesIndex>,
}

glsl_defines!(fixed_header {
    bindings: {
        CLUSTER_MAYBE_ACTIVE_CLUSTER_INDICES_BUFFER_BINDING = 1;
        ACTIVE_CLUSTER_LIGHT_COUNTS_BUFFER_BINDING = 2;
        ACTIVE_CLUSTER_LIGHT_OFFSETS_BUFFER_BINDING = 3;
        LIGHT_BUFFER_BINDING = 4;
        LIGHT_INDICES_BUFFER_BINDING = 8;
        CLUSTER_SPACE_BUFFER_BINDING = 9;
        INSTANCE_MATRICES_BUFFER_BINDING = 10;
        BASIC_ATOMIC_BINDING = 0;
    },
    uniforms: {
        OBJ_TO_WLD_LOC = 0;
        NORMAL_SAMPLER_LOC = 1;
        EMISSIVE_SAMPLER_LOC = 2;
        AMBIENT_SAMPLER_LOC = 3;
        DIFFUSE_SAMPLER_LOC = 4;
        SPECULAR_SAMPLER_LOC = 5;
        SHININESS_LOC = 6;
    },
});

impl Context<'_> {
    pub fn render_main(&mut self, params: &Parameters) {
        let Context {
            ref gl,
            ref mut basic_renderer,
            ..
        } = *self;
        unsafe {
            basic_renderer.update(&mut rendering_context!(self));
            if let ProgramName::Linked(ref program_name) = basic_renderer.program.name {
                let scene_file = &self.resources.scene_file;

                #[derive(Debug)]
                #[repr(C)]
                pub struct InstanceMatrices {
                    pub pos_from_obj_to_wld: Matrix4<f32>,
                    pub pos_from_obj_to_lgt: Matrix4<f32>,
                    pub nor_from_obj_to_lgt: Matrix4<f32>,
                }

                let instance_matrices_buffer: Vec<InstanceMatrices> = scene_file
                    .instances
                    .iter()
                    .map(|instance| {
                        let transform = &scene_file.transforms[instance.transform_index as usize];

                        let pos_from_obj_to_wld = {
                            Matrix4::from_translation(transform.translation.into())
                                * Matrix4::from(Euler {
                                    x: Deg(transform.rotation[0]),
                                    y: Deg(transform.rotation[1]),
                                    z: Deg(transform.rotation[2]),
                                })
                                * Matrix4::from_nonuniform_scale(
                                    transform.scaling[0],
                                    transform.scaling[1],
                                    transform.scaling[2],
                                )
                        };

                        let pos_from_obj_to_lgt = pos_from_obj_to_wld;

                        let nor_from_obj_to_lgt = pos_from_obj_to_lgt.invert().unwrap().transpose();

                        InstanceMatrices {
                            pos_from_obj_to_wld,
                            pos_from_obj_to_lgt,
                            nor_from_obj_to_lgt,
                        }
                    })
                    .collect();

                let mut material_index_to_draw_commands: Vec<Vec<DrawCommand>> =
                    scene_file.materials.iter().map(|_| Vec::new()).collect();

                for (instance_index, instance) in scene_file.instances.iter().enumerate() {
                    let material_index = instance.material_index.map(|n| n.get()).unwrap_or_default() as usize;
                    let draw_commands: &mut Vec<DrawCommand> = &mut material_index_to_draw_commands[material_index];
                    let mesh_description = &scene_file.mesh_descriptions[instance.mesh_index as usize];

                    draw_commands.push(DrawCommand {
                        count: mesh_description.element_count(),
                        prim_count: 1,
                        first_index: mesh_description.element_offset(),
                        base_vertex: mesh_description.vertex_offset,
                        base_instance: instance_index as u32,
                    });
                }

                gl.use_program(*program_name);

                if let Some(cluster_resources_index) = params.cluster_resources_index {
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

                gl.uniform_1i(NORMAL_SAMPLER_LOC, 1);
                gl.uniform_1i(EMISSIVE_SAMPLER_LOC, 2);
                gl.uniform_1i(AMBIENT_SAMPLER_LOC, 3);
                gl.uniform_1i(DIFFUSE_SAMPLER_LOC, 4);
                gl.uniform_1i(SPECULAR_SAMPLER_LOC, 5);

                gl.bind_vertex_array(self.resources.scene_vao);

                {
                    let data = instance_matrices_buffer;
                    let buffer = &mut basic_renderer.instance_matrices_buffer;
                    buffer.ensure_capacity(gl, data.vec_as_bytes().len());
                    buffer.write(gl, data.vec_as_bytes());

                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        INSTANCE_MATRICES_BUFFER_BINDING,
                        buffer.name(),
                    );
                }

                gl.bind_buffer(gl::DRAW_INDIRECT_BUFFER, basic_renderer.draw_commands_buffer.name());

                for (material_index, draw_commands) in material_index_to_draw_commands.iter().enumerate() {
                    let material = &self.resources.materials[material_index];
                    // TODO(mickvangelderen): Have two programs, one without discard and one with discard.
                    // if scene_file.textures[material.diffuse_texture_index as usize].has_alpha {
                    //     continue
                    // }

                    // Update material.
                    gl.uniform_1f(SHININESS_LOC, material.shininess);
                    let textures = &self.resources.textures;
                    gl.bind_texture_unit(1, textures[material.normal_texture_index].name);
                    gl.bind_texture_unit(2, textures[material.emissive_texture_index].name);
                    gl.bind_texture_unit(3, textures[material.ambient_texture_index].name);
                    gl.bind_texture_unit(4, textures[material.diffuse_texture_index].name);
                    gl.bind_texture_unit(5, textures[material.specular_texture_index].name);

                    // Update draw commands buffer.
                    {
                        let data = draw_commands;
                        let buffer = &mut basic_renderer.draw_commands_buffer;
                        buffer.ensure_capacity(gl, data.vec_as_bytes().len());
                        buffer.write(gl, data.vec_as_bytes());
                    }

                    // Execute draw.
                    gl.multi_draw_elements_indirect(gl::TRIANGLES, gl::UNSIGNED_INT, 0, draw_commands.len() as i32, 0);
                }

                gl.unbind_vertex_array();
                gl.unuse_program();
            }
        }
    }
}

impl Renderer {
    pub fn update(&mut self, context: &mut RenderingContext) {
        if self.program.update(context) {
            if let ProgramName::Linked(_name) = self.program.name {
                // Nothing to do anymore.
            }
        }
    }

    pub fn new(context: &mut RenderingContext) -> Self {
        Renderer {
            program: vs_fs_program(context, "basic_renderer.vert", "basic_renderer.frag", fixed_header()),
            draw_commands_buffer: unsafe {
                let buffer = Buffer::new(&context.gl);
                context.gl.buffer_label(&buffer, "basic_renderer.draw_comands");
                buffer
            },
            instance_matrices_buffer: unsafe {
                let buffer = Buffer::new(&context.gl);
                context.gl.buffer_label(&buffer, "basic_renderer.instance_matrices");
                buffer
            },
        }
    }
}
