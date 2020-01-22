use crate::*;

pub struct Renderer {
    pub opaque_program: rendering::Program,
    pub masked_program: rendering::Program,
}

glsl_defines!(fixed_header {
    bindings: {
        INSTANCE_MATRICES_BUFFER_BINDING = 10;
        LIGHT_BUFFER_BINDING = 4;

        DIFFUSE_SAMPLER_BINDING = 4;
    },
    uniforms: {
        WLD_TO_CLP_ARRAY_LOC = 0;
    },
});

pub struct Parameters {
    pub draw_resources_index: usize,
}

impl Context<'_> {
    pub fn render_light_depth(&mut self, params: Parameters) {
        let Context {
            ref gl,
            ref resources,
            light_depth_renderer: ref mut renderer,
            ..
        } = *self;

        let draw_resources = &self.resources.draw_resources_pool[params.draw_resources_index];

        unsafe {
            renderer.opaque_program.update(&mut rendering_context!(self));
            renderer.masked_program.update(&mut rendering_context!(self));
            if let (&ProgramName::Linked(opaque_program), &ProgramName::Linked(masked_program)) =
                (&renderer.opaque_program.name, &renderer.masked_program.name)
            {
                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    INSTANCE_MATRICES_BUFFER_BINDING,
                    draw_resources.instance_matrices_buffer,
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    basic_renderer::LIGHT_BUFFER_BINDING,
                    self.light_resources.buffer_ring[self.frame_index.to_usize()].name(),
                );

                gl.bind_buffer(gl::DRAW_INDIRECT_BUFFER, draw_resources.draw_command_buffer);

                gl.bind_vertex_array(resources.scene_vao);

                let draw_counts = &draw_resources.draw_counts;
                let draw_offsets = &draw_resources.draw_offsets;

                let light = self.point_lights[0];

                let cam_to_clp = Frustum {
                    x0: -1.0,
                    x1: 1.0,
                    y0: -1.0,
                    y1: 1.0,
                    z0: -light.attenuation.r1 as f64,
                    z1: -light.attenuation.r0 as f64 * 0.5_f64.sqrt(),
                }
                .perspective(&super::RENDER_RANGE);

                let wld_to_cam = Matrix4::from_translation(-light.position.to_vec().cast::<f64>().unwrap());

                let wld_to_clp_array: [[[f32; 4]; 4]; 6] = [
                    // +X: s = -Z, t = -Y
                    (cam_to_clp
                        * Matrix4::from_angle_y(Rad::turn_div_4())
                        * Matrix4::from_angle_x(-Rad::turn_div_2())
                        * wld_to_cam)
                        .cast()
                        .unwrap()
                        .into(),
                    // -X: s = Z, t = -Y
                    (cam_to_clp
                        * Matrix4::from_angle_y(-Rad::turn_div_4())
                        * Matrix4::from_angle_x(-Rad::turn_div_2())
                        * wld_to_cam)
                        .cast()
                        .unwrap()
                        .into(),
                    (cam_to_clp * Matrix4::from_angle_x(-Rad::turn_div_4()) * wld_to_cam)
                        .cast()
                        .unwrap()
                        .into(),
                    (cam_to_clp * Matrix4::from_angle_x(Rad::turn_div_4()) * wld_to_cam)
                        .cast()
                        .unwrap()
                        .into(),
                    (cam_to_clp * Matrix4::from_angle_x(-Rad::turn_div_2()) * wld_to_cam)
                        .cast()
                        .unwrap()
                        .into(),
                    (cam_to_clp
                        * Matrix4::from_angle_z(-Rad::turn_div_2())
                        * wld_to_cam)
                        .cast()
                        .unwrap()
                        .into(),
                ];

                for &(program, material_kind) in [
                    (opaque_program, resources::MaterialKind::Opaque),
                    (masked_program, resources::MaterialKind::Masked),
                ]
                .iter()
                {
                    gl.use_program(program);

                    gl.uniform_matrix4fv(WLD_TO_CLP_ARRAY_LOC, gl::MajorAxis::Column, &wld_to_clp_array);

                    for (material_index, material) in resources
                        .materials
                        .iter()
                        .enumerate()
                        .filter(|(_, material)| material.kind == material_kind)
                    {
                        match material_kind {
                            resources::MaterialKind::Opaque => {
                                // Do nothing.
                            }
                            resources::MaterialKind::Masked => {
                                // Update material.
                                let textures = &resources.textures;
                                gl.bind_texture_unit(
                                    DIFFUSE_SAMPLER_BINDING,
                                    textures[material.diffuse_texture_index].name,
                                );
                            }
                            resources::MaterialKind::Transparent => {
                                panic!("Don't use transparent material for depth passes?")
                            }
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
        fn create_header(kind: resources::MaterialKind) -> String {
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
        }

        let mut create_program = |kind: resources::MaterialKind| -> rendering::Program {
            let header = format!("{}{}", fixed_header(), create_header(kind),);

            rendering::Program::new(
                context.gl,
                vec![
                    Shader::new(
                        context.gl,
                        gl::VERTEX_SHADER,
                        EntryPoint::new(
                            &mut shader_compilation_context!(context),
                            "light_depth_renderer.vert",
                            header.clone(),
                        ),
                    ),
                    Shader::new(
                        context.gl,
                        gl::GEOMETRY_SHADER,
                        EntryPoint::new(
                            &mut shader_compilation_context!(context),
                            "light_depth_renderer.geom",
                            header.clone(),
                        ),
                    ),
                    Shader::new(
                        context.gl,
                        gl::FRAGMENT_SHADER,
                        EntryPoint::new(
                            &mut shader_compilation_context!(context),
                            "light_depth_renderer.frag",
                            header,
                        ),
                    ),
                ],
            )
        };

        Renderer {
            opaque_program: create_program(resources::MaterialKind::Opaque),
            masked_program: create_program(resources::MaterialKind::Masked),
        }
    }
}
