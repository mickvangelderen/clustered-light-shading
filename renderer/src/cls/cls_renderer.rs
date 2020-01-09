use crate::*;
use renderer::configuration::FragmentCountingStrategy;

pub struct Renderer {
    pub count_fragments_depth_program: rendering::Program,
    pub count_fragments_opaque_program: rendering::Program,
    pub count_fragments_masked_program: rendering::Program,
    pub count_fragments_transparent_program: rendering::Program,
    pub frag_count_hist_program: rendering::Program,
    pub compact_clusters_0_program: rendering::Program,
    pub compact_clusters_1_program: rendering::Program,
    pub compact_clusters_2_program: rendering::Program,
    pub transform_lights_program: rendering::Program,
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
        LIGHT_BUFFER_BINDING = 5;
        LIGHT_XYZR_BUFFER_BINDING = 6;
        OFFSETS_BUFFER_BINDING = 7;
        DRAW_COMMANDS_BUFFER_BINDING = 8;
        COMPUTE_COMMANDS_BUFFER_BINDING = 9;
        INSTANCE_MATRICES_BUFFER_BINDING = 10;
        LIGHT_INDICES_BUFFER_BINDING = 11;
        CLUSTER_SPACE_BUFFER_BINDING = 12;
        PROFILING_CLUSTER_BUFFER_BINDING = 13;

        // BASIC_ATOMIC_BINDING = 0;

        // NORMAL_SAMPLER_BINDING = 1;
        // EMISSIVE_SAMPLER_BINDING = 2;
        // AMBIENT_SAMPLER_BINDING = 3;
        DIFFUSE_SAMPLER_BINDING = 4;
        // SPECULAR_SAMPLER_BINDING = 5;
    },
    uniforms: {
        DEPTH_SAMPLER_LOC = 0;
        DEPTH_DIMENSIONS_LOC = 1;
        REN_CLP_TO_CLU_CAM_LOC = 2;
        ITEM_COUNT_LOC = 3;
        LGT_TO_CLU_CAM_LOC = 4;
    },
});

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

        let count_fragments_program =
            |context: &mut RenderingContext, kind: resources::MaterialKind| -> rendering::Program {
                vs_fs_program(
                    context,
                    "cls/count_fragments.vert",
                    "cls/count_fragments.frag",
                    format!("{}{}", fixed_header(), basic_pass_header(kind)),
                )
            };

        let compute_program = |context: &mut RenderingContext, path: &'static str| -> rendering::Program {
            rendering::Program::new(
                context.gl,
                vec![rendering::Shader::new(
                    context.gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(&mut shader_compilation_context!(context), path, fixed_header()),
                )],
            )
        };

        Renderer {
            count_fragments_depth_program: compute_program(context, "cls/count_fragments_depth.comp"),
            count_fragments_opaque_program: count_fragments_program(context, resources::MaterialKind::Opaque),
            count_fragments_masked_program: count_fragments_program(context, resources::MaterialKind::Masked),
            count_fragments_transparent_program: count_fragments_program(context, resources::MaterialKind::Transparent),
            frag_count_hist_program: compute_program(context, "cls/frag_count_hist.comp"),
            compact_clusters_0_program: compute_program(context, "cls/compact_clusters_0.comp"),
            compact_clusters_1_program: compute_program(context, "cls/compact_clusters_1.comp"),
            compact_clusters_2_program: compute_program(context, "cls/compact_clusters_2.comp"),
            transform_lights_program: compute_program(context, "cls/transform_lights.comp"),
            count_lights_program: compute_program(context, "cls/count_lights.comp"),
            light_count_hist_program: compute_program(context, "cls/light_count_hist.comp"),
            compact_light_counts_0_program: compute_program(context, "cls/compact_light_counts_0.comp"),
            compact_light_counts_1_program: compute_program(context, "cls/compact_light_counts_1.comp"),
            compact_light_counts_2_program: compute_program(context, "cls/compact_light_counts_2.comp"),
            assign_lights_program: compute_program(context, "cls/assign_lights.comp"),
        }
    }
}

impl Context<'_> {
    pub fn compute_clustering(&mut self, cluster_resources_index: ClusterResourcesIndex) {
        // Reborrow
        let gl = &self.gl;
        let cluster_resources = &mut self.cluster_resources_pool[cluster_resources_index];
        cluster_resources.recompute();

        let cluster_profiler_index = self.profiling_context.start(gl, cluster_resources.profilers.cluster);

        let cluster_count = cluster_resources.computed.cluster_count();

        unsafe {
            let data = ClusterSpaceBuffer::from(cluster_resources);
            let buffer = &mut cluster_resources.cluster_space_buffer;

            buffer.invalidate(gl);
            buffer.write(gl, data.value_as_bytes());
        }

        unsafe {
            let buffer = &mut cluster_resources.cluster_fragment_counts_buffer;
            let byte_count = std::mem::size_of::<u32>() * cluster_resources.computed.cluster_count() as usize;
            buffer.invalidate(gl);
            // buffer.ensure_capacity(gl, byte_count);
            buffer.clear_0u32(gl, byte_count);
        }

        // NOTE: Work around borrow checker.
        for camera_resources_index in cluster_resources.camera_resources_pool.used_index_iter() {
            let draw_resources_index = self.resources.draw_resources_pool.next({
                let gl = &self.gl;
                let profiling_context = &mut self.profiling_context;
                move || resources::DrawResources::new(gl, profiling_context)
            });

            // Reborrow.
            let gl = &self.gl;
            let cluster_resources = &mut self.cluster_resources_pool[cluster_resources_index];
            let camera_resources = &mut cluster_resources.camera_resources_pool[camera_resources_index];
            let camera_parameters = &camera_resources.parameters;
            let draw_resources = &mut self.resources.draw_resources_pool[draw_resources_index];

            let camera_profiler_index = self.profiling_context.start(gl, camera_resources.profilers.camera);

            draw_resources.recompute(
                &self.gl,
                &mut self.profiling_context,
                camera_parameters.camera.wld_to_clp,
                cluster_resources.computed.wld_to_clu_cam,
                &self.resources.scene_file.instances,
                &self.resources.materials,
                &self.resources.scene_file.transforms,
                &self.resources.scene_file.mesh_descriptions,
            );

            let main_resources_index = self.main_resources_pool.next_unused(
                gl,
                &mut self.profiling_context,
                camera_parameters.frame_dims,
                self.configuration.global.sample_count,
            );

            self.clear_and_render_depth(main_resources_index, draw_resources_index);

            // Reborrow.
            let gl = &self.gl;
            let cluster_resources = &mut self.cluster_resources_pool[cluster_resources_index];
            let camera_resources = &mut cluster_resources.camera_resources_pool[camera_resources_index];

            let camera_parameters = &camera_resources.parameters;
            let main_resources = &mut self.main_resources_pool[main_resources_index];

            // Re-bind buffers.
            unsafe {
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::CLUSTER_FRAGMENT_COUNTS_BUFFER_BINDING,
                    cluster_resources.cluster_fragment_counts_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::UNIFORM_BUFFER,
                    cls_renderer::CLUSTER_SPACE_BUFFER_BINDING,
                    cluster_resources.cluster_space_buffer.name(),
                );
            }

            {
                let profiler_index = self.profiling_context.start(gl, camera_resources.profilers.count_frags);
                {
                    let profiler_index = self
                        .profiling_context
                        .start(gl, camera_resources.profilers.count_opaque_masked_frags);

                    match self.configuration.clustered_light_shading.fragment_counting_strategy {
                        FragmentCountingStrategy::Depth => unsafe {
                            let program = &mut self.cls_renderer.count_fragments_depth_program;
                            program.update(&mut rendering_context!(self));
                            if let ProgramName::Linked(name) = program.name {
                                gl.use_program(name);

                                gl.bind_texture_unit(
                                    0,
                                    main_resources
                                        .cluster_depth_texture
                                        .unwrap_or(main_resources.depth_texture),
                                );

                                gl.uniform_2i(
                                    cls_renderer::DEPTH_DIMENSIONS_LOC,
                                    main_resources.dimensions.cast::<i32>().unwrap().into(),
                                );

                                let ren_clp_to_clu_cam =
                                    cluster_resources.computed.wld_to_clu_cam * camera_parameters.camera.clp_to_wld;

                                gl.uniform_matrix4f(
                                    cls_renderer::REN_CLP_TO_CLU_CAM_LOC,
                                    gl::MajorAxis::Column,
                                    ren_clp_to_clu_cam.cast::<f32>().unwrap().as_ref(),
                                );

                                gl.memory_barrier(
                                    gl::MemoryBarrierFlag::TEXTURE_FETCH | gl::MemoryBarrierFlag::FRAMEBUFFER,
                                );

                                let (lx, ly) = match self.configuration.global.sample_count {
                                    0 | 1 => (16, 16),
                                    2 => (8, 16),
                                    4 => (8, 8),
                                    8 => (4, 8),
                                    16 => (4, 4),
                                    other => panic!("Unsupported multisampling sample count {}.", other),
                                };

                                gl.dispatch_compute(
                                    main_resources.dimensions.x.ceiled_div(lx) as u32,
                                    main_resources.dimensions.y.ceiled_div(ly) as u32,
                                    1,
                                );

                                gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                            }
                        },
                        FragmentCountingStrategy::Geometry => unsafe {
                            let renderer = &mut self.cls_renderer;

                            renderer
                                .count_fragments_opaque_program
                                .update(&mut rendering_context!(self));
                            renderer
                                .count_fragments_masked_program
                                .update(&mut rendering_context!(self));
                            if let (&ProgramName::Linked(opaque_program), &ProgramName::Linked(masked_program)) = (
                                &renderer.count_fragments_opaque_program.name,
                                &renderer.count_fragments_masked_program.name,
                            ) {
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
                                        camera_resources.profilers.count_opaque_frags,
                                    ),
                                    (
                                        masked_program,
                                        resources::MaterialKind::Masked,
                                        camera_resources.profilers.count_masked_frags,
                                    ),
                                ]
                                .iter()
                                {
                                    let profiler_index = self.profiling_context.start(gl, profiler);

                                    gl.use_program(program);

                                    gl.depth_func(gl::GEQUAL);
                                    gl.depth_mask(gl::WriteMask::Disabled);
                                    gl.color_mask(
                                        gl::WriteMask::Disabled,
                                        gl::WriteMask::Disabled,
                                        gl::WriteMask::Disabled,
                                        gl::WriteMask::Disabled,
                                    );

                                    for (material_index, material) in self
                                        .resources
                                        .materials
                                        .iter()
                                        .enumerate()
                                        .filter(|(_, material)| material.kind == material_kind)
                                    {
                                        if material_kind == resources::MaterialKind::Masked {
                                            // Update material.
                                            gl.bind_texture_unit(
                                                DIFFUSE_SAMPLER_BINDING,
                                                self.resources.textures[material.diffuse_texture_index].name,
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

                                    gl.depth_func(gl::GREATER);
                                    gl.depth_mask(gl::WriteMask::Enabled);
                                    gl.color_mask(
                                        gl::WriteMask::Enabled,
                                        gl::WriteMask::Enabled,
                                        gl::WriteMask::Enabled,
                                        gl::WriteMask::Enabled,
                                    );

                                    self.profiling_context.stop(gl, profiler_index);
                                }

                                gl.unuse_program();

                                gl.unbind_vertex_array();
                            }
                        },
                    }

                    self.profiling_context.stop(gl, profiler_index);
                }

                // Transparent
                unsafe {
                    let renderer = &mut self.cls_renderer;

                    renderer
                        .count_fragments_transparent_program
                        .update(&mut rendering_context!(self));
                    if let &ProgramName::Linked(transparent_program) = &renderer.count_fragments_opaque_program.name {
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

                        for &(program, material_kind, profiler) in [(
                            transparent_program,
                            resources::MaterialKind::Transparent,
                            camera_resources.profilers.count_transparent_frags,
                        )]
                        .iter()
                        {
                            let profiler_index = self.profiling_context.start(gl, profiler);

                            gl.use_program(program);

                            gl.depth_func(gl::GEQUAL);
                            gl.depth_mask(gl::WriteMask::Disabled);
                            gl.color_mask(
                                gl::WriteMask::Disabled,
                                gl::WriteMask::Disabled,
                                gl::WriteMask::Disabled,
                                gl::WriteMask::Disabled,
                            );

                            for (material_index, material) in self
                                .resources
                                .materials
                                .iter()
                                .enumerate()
                                .filter(|(_, material)| material.kind == material_kind)
                            {
                                // Update material.
                                gl.bind_texture_unit(
                                    DIFFUSE_SAMPLER_BINDING,
                                    self.resources.textures[material.diffuse_texture_index].name,
                                );

                                // Execute draw.
                                gl.multi_draw_elements_indirect(
                                    gl::TRIANGLES,
                                    gl::UNSIGNED_INT,
                                    draw_offsets[material_index] as usize * std::mem::size_of::<DrawCommand>(),
                                    draw_counts[material_index] as i32,
                                    std::mem::size_of::<DrawCommand>() as i32,
                                );
                            }

                            gl.depth_func(gl::GREATER);
                            gl.depth_mask(gl::WriteMask::Enabled);
                            gl.color_mask(
                                gl::WriteMask::Enabled,
                                gl::WriteMask::Enabled,
                                gl::WriteMask::Enabled,
                                gl::WriteMask::Enabled,
                            );

                            self.profiling_context.stop(gl, profiler_index);
                        }

                        gl.unuse_program();

                        gl.unbind_vertex_array();
                    }
                }

                self.profiling_context.stop(gl, profiler_index);
            }

            self.profiling_context.stop(gl, camera_profiler_index);
        }

        // Reborrow.
        let gl = &self.gl;
        let cluster_resources = &mut self.cluster_resources_pool[cluster_resources_index];

        // We have our fragments per cluster buffer here.
        // TODO: Don't do this when running the application??
        if !self.profiling_context.time_sensitive() {
            unsafe {
                let buffer = &mut cluster_resources.profiling_cluster_buffer;
                let byte_count = std::mem::size_of::<profiling::ClusterBuffer>();
                buffer.invalidate(gl);
                buffer.ensure_capacity(gl, byte_count);
                buffer.clear_0u32(gl, byte_count);
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::PROFILING_CLUSTER_BUFFER_BINDING,
                    buffer.name(),
                );
            }

            unsafe {
                let program = &mut self.cls_renderer.frag_count_hist_program;
                program.update(&mut rendering_context!(self));
                if let ProgramName::Linked(name) = program.name {
                    gl.use_program(name);
                    // NOTE(mickvangelderen): 32*8 is defined in the shader
                    gl.dispatch_compute(cluster_count.ceiled_div(32 * 8), 1, 1);
                    gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                }
            }
        }

        {
            let profiler_index = self
                .profiling_context
                .start(gl, cluster_resources.profilers.compact_clusters);

            unsafe {
                let buffer = &mut cluster_resources.offset_buffer;
                let byte_count = std::mem::size_of::<u32>() * self.configuration.prefix_sum.pass_1_threads as usize;
                buffer.invalidate(gl);
                buffer.ensure_capacity(gl, byte_count);
                buffer.clear_0u32(gl, byte_count);
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::OFFSETS_BUFFER_BINDING,
                    buffer.name(),
                );
            }

            unsafe {
                let buffer = &mut cluster_resources.cluster_maybe_active_cluster_indices_buffer;
                let byte_count = std::mem::size_of::<u32>() * cluster_resources.computed.cluster_count() as usize;
                buffer.invalidate(gl);
                assert!(byte_count <= buffer.byte_capacity());
                // buffer.ensure_capacity(gl, byte_count);
                // NOTE(mickvangelderen): No need to clear, a value is written for each cluster.
                // buffer.clear_0u32(gl, byte_count);
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::CLUSTER_MAYBE_ACTIVE_CLUSTER_INDICES_BUFFER_BINDING,
                    buffer.name(),
                );
            }

            unsafe {
                let buffer = &mut cluster_resources.active_cluster_cluster_indices_buffer;
                buffer.invalidate(gl);
                // Can't check capacity because the number of active clusters is only known on the GPU.
                // NOTE(mickvangelderen): No need to clear, a value is written for each cluster.
                // buffer.clear_0u32(gl, buffer.byte_capacity());
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::ACTIVE_CLUSTER_CLUSTER_INDICES_BUFFER_BINDING,
                    buffer.name(),
                );
            }

            unsafe {
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::DRAW_COMMANDS_BUFFER_BINDING,
                    cluster_resources.draw_commands_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::COMPUTE_COMMANDS_BUFFER_BINDING,
                    cluster_resources.compute_commands_buffer.name(),
                );
            }

            let renderer::configuration::PrefixSumConfiguration {
                pass_0_threads,
                pass_1_threads,
                ..
            } = self.configuration.prefix_sum;
            let items_per_workgroup = cluster_count.ceiled_div(pass_0_threads * pass_1_threads) * pass_0_threads;
            let workgroup_count = cluster_count.ceiled_div(items_per_workgroup);

            unsafe {
                let program = &mut self.cls_renderer.compact_clusters_0_program;
                program.update(&mut rendering_context!(self));
                if let ProgramName::Linked(name) = program.name {
                    gl.use_program(name);
                    gl.dispatch_compute(workgroup_count, 1, 1);
                    gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                }
            }

            unsafe {
                let program = &mut self.cls_renderer.compact_clusters_1_program;
                program.update(&mut rendering_context!(self));
                if let ProgramName::Linked(name) = program.name {
                    gl.use_program(name);
                    gl.dispatch_compute(1, 1, 1);
                    gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                }
            }

            unsafe {
                let program = &mut self.cls_renderer.compact_clusters_2_program;
                program.update(&mut rendering_context!(self));
                if let ProgramName::Linked(name) = program.name {
                    gl.use_program(name);
                    gl.dispatch_compute(workgroup_count, 1, 1);
                    gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                }
            }

            self.profiling_context.stop(gl, profiler_index);
        }

        // We have our active clusters.

        unsafe {
            let profiler_index = self
                .profiling_context
                .start(gl, cluster_resources.profilers.transform_lights);

            gl.bind_buffer_base(
                gl::SHADER_STORAGE_BUFFER,
                cls_renderer::LIGHT_BUFFER_BINDING,
                self.light_resources.buffer_ring[self.frame_index.to_usize()].name(),
            );

            gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, cls_renderer::LIGHT_XYZR_BUFFER_BINDING, {
                let buffer = &mut cluster_resources.light_xyzr_buffer_ring[self.frame_index.to_usize()];
                buffer.reconcile(
                    gl,
                    std::mem::size_of::<[f32; 4]>() * self.light_resources.header.light_count as usize,
                );

                buffer.name()
            });

            let program = &mut self.cls_renderer.transform_lights_program;
            program.update(&mut rendering_context!(self));
            if let ProgramName::Linked(name) = program.name {
                gl.use_program(name);

                gl.uniform_matrix4f(
                    cls_renderer::LGT_TO_CLU_CAM_LOC,
                    gl::MajorAxis::Column,
                    cluster_resources
                        .computed
                        .wld_to_clu_cam
                        .cast::<f32>()
                        .unwrap()
                        .as_ref(),
                );

                gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE | gl::MemoryBarrierFlag::COMMAND);
                gl.dispatch_compute(self.light_resources.header.light_count.ceiled_div(480), 1, 1);
                gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
            }

            self.profiling_context.stop(gl, profiler_index);
        }

        {
            let profiler_index = self
                .profiling_context
                .start(gl, cluster_resources.profilers.count_lights);

            unsafe {
                let buffer = &mut cluster_resources.active_cluster_light_counts_buffer;
                buffer.invalidate(gl);
                // buffer.ensure_capacity(gl, byte_count);
                buffer.clear_0u32(gl, buffer.byte_capacity());
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::ACTIVE_CLUSTER_LIGHT_COUNTS_BUFFER_BINDING,
                    buffer.name(),
                );
            }

            unsafe {
                let program = &mut self.cls_renderer.count_lights_program;
                program.update(&mut rendering_context!(self));
                if let ProgramName::Linked(name) = program.name {
                    gl.use_program(name);
                    gl.bind_buffer(
                        gl::DISPATCH_INDIRECT_BUFFER,
                        cluster_resources.compute_commands_buffer.name(),
                    );
                    gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE | gl::MemoryBarrierFlag::COMMAND);
                    gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 0);
                    gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                }
            }
            self.profiling_context.stop(gl, profiler_index);
        }

        // We have our light counts.

        if !self.profiling_context.time_sensitive() {
            unsafe {
                let program = &mut self.cls_renderer.light_count_hist_program;
                program.update(&mut rendering_context!(self));
                if let ProgramName::Linked(name) = program.name {
                    gl.use_program(name);
                    gl.bind_buffer(
                        gl::DISPATCH_INDIRECT_BUFFER,
                        cluster_resources.compute_commands_buffer.name(),
                    );
                    gl.memory_barrier(gl::MemoryBarrierFlag::COMMAND);
                    // NOTE: the compute command at offset 2 should be (x = active_cluster_count/(32*8), y = 1, z = 0).
                    gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 2);
                    gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                }
            }
        }

        {
            let profiler_index = self
                .profiling_context
                .start(gl, cluster_resources.profilers.light_offsets);

            unsafe {
                let buffer = &mut cluster_resources.offset_buffer;
                let byte_count = std::mem::size_of::<u32>() * self.configuration.prefix_sum.pass_1_threads as usize;
                buffer.invalidate(gl);
                buffer.ensure_capacity(gl, byte_count);
                buffer.clear_0u32(gl, byte_count);
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::OFFSETS_BUFFER_BINDING,
                    buffer.name(),
                );
            }

            unsafe {
                let buffer = &mut cluster_resources.active_cluster_light_offsets_buffer;
                buffer.invalidate(gl);
                // buffer.ensure_capacity(gl, byte_count);
                buffer.clear_0u32(gl, buffer.byte_capacity());
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::ACTIVE_CLUSTER_LIGHT_OFFSETS_BUFFER_BINDING,
                    buffer.name(),
                );
                gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE);
            }

            unsafe {
                let program = &mut self.cls_renderer.compact_light_counts_0_program;
                program.update(&mut rendering_context!(self));
                if let ProgramName::Linked(name) = program.name {
                    gl.use_program(name);
                    gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 1);
                    gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                }
            }

            unsafe {
                let program = &mut self.cls_renderer.compact_light_counts_1_program;
                program.update(&mut rendering_context!(self));
                if let ProgramName::Linked(name) = program.name {
                    gl.use_program(name);
                    gl.dispatch_compute(1, 1, 1);
                    gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                }
            }

            unsafe {
                let program = &mut self.cls_renderer.compact_light_counts_2_program;
                program.update(&mut rendering_context!(self));
                if let ProgramName::Linked(name) = program.name {
                    gl.use_program(name);
                    gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 1);
                    gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                }
            }

            self.profiling_context.stop(gl, profiler_index);
        }

        // We have our light offsets.

        {
            let profiler_index = self
                .profiling_context
                .start(gl, cluster_resources.profilers.assign_lights);

            unsafe {
                let buffer = &mut cluster_resources.light_indices_buffer;
                buffer.invalidate(gl);
                // buffer.ensure_capacity(gl, byte_count);
                buffer.clear_0u32(gl, buffer.byte_capacity());
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::LIGHT_INDICES_BUFFER_BINDING,
                    buffer.name(),
                );
            }

            unsafe {
                let program = &mut self.cls_renderer.assign_lights_program;
                program.update(&mut rendering_context!(self));
                if let ProgramName::Linked(name) = program.name {
                    gl.use_program(name);
                    gl.bind_buffer(
                        gl::DISPATCH_INDIRECT_BUFFER,
                        cluster_resources.compute_commands_buffer.name(),
                    );
                    gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE | gl::MemoryBarrierFlag::COMMAND);
                    gl.dispatch_compute_indirect(std::mem::size_of::<ComputeCommand>() * 0);
                    gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);
                }
            }

            self.profiling_context.stop(gl, profiler_index);
        }

        if !self.profiling_context.time_sensitive() {
            unsafe {
                self.profiling_context
                    .record_cluster_buffer(gl, &cluster_resources.profiling_cluster_buffer.name(), 0);
            }
        }

        self.profiling_context.stop(gl, cluster_profiler_index);
    }
}
