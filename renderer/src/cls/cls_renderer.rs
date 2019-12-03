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
        DEPTH_DIMENSIONS_LOC = 1;
        REN_CLP_TO_CLU_CAM = 2;
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

impl Context<'_> {
    pub fn compute_clustering(&mut self, cluster_resources_index: ClusterResourcesIndex) {
        // Reborrow
        let gl = &self.gl;
        let cluster_resources = &mut self.cluster_resources_pool[cluster_resources_index];
        cluster_resources.recompute();

        let cluster_count = cluster_resources.computed.cluster_count();

        unsafe {
            let data = ClusterSpaceBuffer::from(cluster_resources);
            let buffer = &mut cluster_resources.cluster_space_buffer;

            buffer.invalidate(gl);
            buffer.write(gl, data.value_as_bytes());
            gl.bind_buffer_base(
                gl::UNIFORM_BUFFER,
                cls_renderer::CLUSTER_SPACE_BUFFER_BINDING,
                buffer.name(),
            );
        }

        unsafe {
            let buffer = &mut cluster_resources.cluster_fragment_counts_buffer;
            let byte_count = std::mem::size_of::<u32>() * cluster_resources.computed.cluster_count() as usize;
            buffer.invalidate(gl);
            // buffer.ensure_capacity(gl, byte_count);
            buffer.clear_0u32(gl, byte_count);
        }

        unsafe {
            let buffer = &mut cluster_resources.cluster_maybe_active_cluster_indices_buffer;
            let byte_count = std::mem::size_of::<u32>() * cluster_resources.computed.cluster_count() as usize;
            assert!(byte_count <= buffer.byte_capacity());
            buffer.invalidate(gl);
            // buffer.ensure_capacity(gl, byte_count);
            // buffer.clear_0u32(gl, byte_count);
            gl.bind_buffer_base(
                gl::SHADER_STORAGE_BUFFER,
                cls_renderer::CLUSTER_MAYBE_ACTIVE_CLUSTER_INDICES_BUFFER_BINDING,
                buffer.name(),
            );
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

            let profiler_index = self
                .profiling_context
                .start(gl, camera_resources.profilers.render_depth);

            let main_resources_index = self.main_resources_pool.next_unused(
                gl,
                &mut self.profiling_context,
                camera_parameters.frame_dims,
                self.configuration.global.sample_count,
            );

            self.clear_and_render_depth(main_resources_index, draw_resources_index);

            self.profiling_context.stop(gl, profiler_index);

            // Reborrow.
            let gl = &self.gl;
            let cluster_resources = &mut self.cluster_resources_pool[cluster_resources_index];
            let camera_resources = &mut cluster_resources.camera_resources_pool[camera_resources_index];

            let camera_parameters = &camera_resources.parameters;
            let main_resources = &mut self.main_resources_pool[main_resources_index];

            {
                let profiler_index = self.profiling_context.start(gl, camera_resources.profilers.count_frags);

                unsafe {
                    // gl.bind_framebuffer(gl::FRAMEBUFFER, gl::FramebufferName::Default);
                    let program = &mut self.cls_renderer.count_fragments_program;
                    program.update(&mut rendering_context!(self));
                    if let ProgramName::Linked(name) = program.name {
                        gl.use_program(name);

                        gl.bind_buffer_base(
                            gl::SHADER_STORAGE_BUFFER,
                            cls_renderer::CLUSTER_FRAGMENT_COUNTS_BUFFER_BINDING,
                            cluster_resources.cluster_fragment_counts_buffer.name(),
                        );

                        gl.bind_texture_unit(0, main_resources.cluster_depth_texture.unwrap_or(main_resources.depth_texture));

                        gl.uniform_2i(
                            cls_renderer::DEPTH_DIMENSIONS_LOC,
                            main_resources.dimensions.cast::<i32>().unwrap().into(),
                        );

                        let ren_clp_to_clu_cam =
                            cluster_resources.computed.wld_to_clu_cam * camera_parameters.camera.clp_to_wld;

                        gl.uniform_matrix4f(
                            cls_renderer::REN_CLP_TO_CLU_CAM,
                            gl::MajorAxis::Column,
                            ren_clp_to_clu_cam.cast::<f32>().unwrap().as_ref(),
                        );

                        gl.memory_barrier(gl::MemoryBarrierFlag::TEXTURE_FETCH | gl::MemoryBarrierFlag::FRAMEBUFFER);

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
                }

                self.profiling_context.stop(gl, profiler_index);
            }
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
                let buffer = &mut cluster_resources.active_cluster_cluster_indices_buffer;
                buffer.invalidate(gl);
                // buffer.ensure_capacity(gl, byte_count);
                buffer.clear_0u32(gl, buffer.byte_capacity());
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
            }

            unsafe {
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::COMPUTE_COMMANDS_BUFFER_BINDING,
                    cluster_resources.compute_commands_buffer.name(),
                );
            }

            let renderer::PrefixSumConfiguration {
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

        {
            let profiler_index = self
                .profiling_context
                .start(gl, cluster_resources.profilers.upload_lights);

            unsafe {
                let data: Vec<[f32; 4]> = self
                    .point_lights
                    .iter()
                    .map(|&light| {
                        let pos_in_clu_cam = cluster_resources
                            .computed
                            .wld_to_clu_cam
                            .transform_point(light.pos_in_wld.cast().unwrap());
                        let [x, y, z]: [f32; 3] = pos_in_clu_cam.cast::<f32>().unwrap().into();
                        [x, y, z, light.attenuation.clip_far]
                    })
                    .collect();
                let bytes = data.vec_as_bytes();
                let padded_byte_count = bytes.len().ceiled_div(64) * 64;

                let buffer = &mut cluster_resources.light_xyzr_buffer;
                buffer.invalidate(gl);
                buffer.ensure_capacity(gl, padded_byte_count);
                buffer.write_at(gl, bytes, 0);
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    cls_renderer::LIGHT_XYZR_BUFFER_BINDING,
                    buffer.name(),
                );
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
                    gl.uniform_1ui(cls_renderer::LIGHT_COUNT_LOC, self.point_lights.len() as u32);
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
                    gl.uniform_1ui(cls_renderer::LIGHT_COUNT_LOC, self.point_lights.len() as u32);
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
    }
}
