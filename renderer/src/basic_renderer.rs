use crate::*;

pub struct Renderer {
    pub program: rendering::Program,

    pub diffuse_sampler_loc: gl::OptionUniformLocation,
    pub normal_sampler_loc: gl::OptionUniformLocation,
    pub specular_sampler_loc: gl::OptionUniformLocation,

    pub diffuse_dimensions_loc: gl::OptionUniformLocation,
    pub normal_dimensions_loc: gl::OptionUniformLocation,
    pub specular_dimensions_loc: gl::OptionUniformLocation,
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
        BASIC_ATOMIC_BINDING = 0;
    },
    uniforms: {
        OBJ_TO_WLD_LOC = 0;
        AMBIENT_COLOR_LOC = 2;
        DIFFUSE_COLOR_LOC = 3;
        SPECULAR_COLOR_LOC = 4;
        SHININESS_LOC = 5;
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

                if let Some(loc) = basic_renderer.diffuse_sampler_loc.into() {
                    gl.uniform_1i(loc, 1);
                }

                if let Some(loc) = basic_renderer.normal_sampler_loc.into() {
                    gl.uniform_1i(loc, 2);
                }

                if let Some(loc) = basic_renderer.specular_sampler_loc.into() {
                    gl.uniform_1i(loc, 3);
                }

                // Cache texture binding.
                let mut bound_material = None;

                gl.bind_vertex_array(self.resources.scene_vao);

                let scene_file = &self.resources.scene_file;

                for instance in scene_file.instances.iter() {
                    let transform = &scene_file.transforms[instance.transform_index as usize];
                    let mesh_description = &scene_file.mesh_descriptions[instance.mesh_index as usize];
                    let material_index = instance.material_index.map(|n| n.get()).unwrap_or_default() as usize;
                    let material = &scene_file.materials[material_index];

                    if bound_material != Some(material_index) {
                        bound_material = Some(material_index);

                        gl.uniform_3f(AMBIENT_COLOR_LOC, material.ambient_color);
                        gl.uniform_3f(DIFFUSE_COLOR_LOC, material.diffuse_color);
                        gl.uniform_3f(SPECULAR_COLOR_LOC, material.specular_color);
                        gl.uniform_1f(SHININESS_LOC, material.shininess);

                        // let diffuse_texture = &self.resources.textures[material.diffuse as usize];
                        // let normal_texture = &self.resources.textures[material.normal as usize];
                        // let specular_texture = &self.resources.textures[material.specular as usize];

                        // gl.bind_texture_unit(1, diffuse_texture.name);
                        // gl.bind_texture_unit(2, normal_texture.name);
                        // gl.bind_texture_unit(3, specular_texture.name);

                        // if let Some(loc) = basic_renderer.diffuse_dimensions_loc.into() {
                        //     gl.uniform_2f(loc, diffuse_texture.dimensions);
                        // }

                        // if let Some(loc) = basic_renderer.normal_dimensions_loc.into() {
                        //     gl.uniform_2f(loc, normal_texture.dimensions);
                        // }

                        // if let Some(loc) = basic_renderer.specular_dimensions_loc.into() {
                        //     gl.uniform_2f(loc, specular_texture.dimensions);
                        // }

                        // params.material_resources.bind_index(gl, material_index as usize);
                    }

                    {
                        let obj_to_wld = Matrix4::from_translation(transform.translation.into())
                            * Matrix4::from(Euler {
                                x: Deg(transform.rotation[0]),
                                y: Deg(transform.rotation[1]),
                                z: Deg(transform.rotation[2]),
                            })
                            * Matrix4::from_nonuniform_scale(
                                transform.scaling[0],
                                transform.scaling[1],
                                transform.scaling[2],
                            );
                        gl.uniform_matrix4f(
                            OBJ_TO_WLD_LOC,
                            gl::MajorAxis::Column,
                            obj_to_wld.cast().unwrap().as_ref(),
                        );
                    }

                    gl.draw_elements_base_vertex(
                        gl::TRIANGLES,
                        mesh_description.element_count,
                        gl::UNSIGNED_INT,
                        mesh_description.index_byte_offset as usize,
                        mesh_description.vertex_offset,
                    );
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
            let gl = &context.gl;

            if let ProgramName::Linked(name) = self.program.name {
                unsafe {
                    self.diffuse_sampler_loc = get_uniform_location!(gl, name, "diffuse_sampler");
                    self.normal_sampler_loc = get_uniform_location!(gl, name, "normal_sampler");
                    self.specular_sampler_loc = get_uniform_location!(gl, name, "specular_sampler");

                    self.diffuse_dimensions_loc = get_uniform_location!(gl, name, "diffuse_dimensions");
                    self.normal_dimensions_loc = get_uniform_location!(gl, name, "normal_dimensions");
                    self.specular_dimensions_loc = get_uniform_location!(gl, name, "specular_dimensions");
                }
            }
        }
    }

    pub fn new(context: &mut RenderingContext) -> Self {
        Renderer {
            program: vs_fs_program(context, "basic_renderer.vert", "basic_renderer.frag", fixed_header()),

            diffuse_sampler_loc: gl::OptionUniformLocation::NONE,
            normal_sampler_loc: gl::OptionUniformLocation::NONE,
            specular_sampler_loc: gl::OptionUniformLocation::NONE,

            diffuse_dimensions_loc: gl::OptionUniformLocation::NONE,
            normal_dimensions_loc: gl::OptionUniformLocation::NONE,
            specular_dimensions_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
