use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
    //
    pub obj_to_wld_loc: gl::OptionUniformLocation,
    pub cam_to_cls_loc: gl::OptionUniformLocation,
    pub cluster_dims_loc: gl::OptionUniformLocation,

    pub diffuse_sampler_loc: gl::OptionUniformLocation,
    pub normal_sampler_loc: gl::OptionUniformLocation,
    pub specular_sampler_loc: gl::OptionUniformLocation,

    pub diffuse_dimensions_loc: gl::OptionUniformLocation,
    pub normal_dimensions_loc: gl::OptionUniformLocation,
    pub specular_dimensions_loc: gl::OptionUniformLocation,

    pub display_mode_loc: gl::OptionUniformLocation,
}

pub struct Parameters {
    pub mode: u32,
    pub cluster_resources_index: Option<ClusterResourcesIndex>,
}

pub const MAYBE_ACTIVE_CLUSTER_INDICES_BINDING: u32 = 10;
pub const ACTIVE_CLUSTER_LIGHT_COUNTS_BINDING: u32 = 11;
pub const ACTIVE_CLUSTER_LIGHT_OFFSETS_BINDING: u32 = 12;
pub const LIGHT_INDICES_BUFFER: u32 = 13;

impl Context {
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

                    if let Some(loc) = basic_renderer.cam_to_cls_loc.into() {
                        gl.uniform_matrix4f(
                            loc,
                            gl::MajorAxis::Column,
                            cluster_resources.computed.cam_to_cls.cast().unwrap().as_ref(),
                        );
                    }

                    if let Some(loc) = basic_renderer.cluster_dims_loc.into() {
                        gl.uniform_3ui(loc, cluster_resources.computed.dimensions.cast().unwrap().into());
                    }

                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        MAYBE_ACTIVE_CLUSTER_INDICES_BINDING,
                        cluster_resources.cluster_fragment_counts_buffer.name(),
                    );

                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        ACTIVE_CLUSTER_LIGHT_COUNTS_BINDING,
                        cluster_resources.active_cluster_light_counts_buffer.name(),
                    );

                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        ACTIVE_CLUSTER_LIGHT_OFFSETS_BINDING,
                        cluster_resources.active_cluster_light_offsets_buffer.name(),
                    );

                    gl.bind_buffer_base(
                        gl::SHADER_STORAGE_BUFFER,
                        LIGHT_INDICES_BUFFER,
                        cluster_resources.light_indices_buffer.name(),
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

                if let Some(loc) = basic_renderer.display_mode_loc.into() {
                    gl.uniform_1ui(loc, params.mode);
                }

                // Cache texture binding.
                let mut bound_material = None;

                gl.bind_vertex_array(self.resources.scene_vao);

                for (i, mesh_meta) in self.resources.mesh_metas.iter().enumerate() {
                    let maybe_material_index = self.resources.meshes[i].material_index;
                    if bound_material != maybe_material_index {
                        bound_material = maybe_material_index;
                        if let Some(material_index) = maybe_material_index {
                            let material = &self.resources.materials[material_index as usize];

                            let diffuse_texture = &self.resources.textures[material.diffuse as usize];
                            let normal_texture = &self.resources.textures[material.normal as usize];
                            let specular_texture = &self.resources.textures[material.specular as usize];

                            gl.bind_texture_unit(1, diffuse_texture.name);
                            gl.bind_texture_unit(2, normal_texture.name);
                            gl.bind_texture_unit(3, specular_texture.name);

                            if let Some(loc) = basic_renderer.diffuse_dimensions_loc.into() {
                                gl.uniform_2f(loc, diffuse_texture.dimensions);
                            }

                            if let Some(loc) = basic_renderer.normal_dimensions_loc.into() {
                                gl.uniform_2f(loc, normal_texture.dimensions);
                            }

                            if let Some(loc) = basic_renderer.specular_dimensions_loc.into() {
                                gl.uniform_2f(loc, specular_texture.dimensions);
                            }

                        // params.material_resources.bind_index(gl, material_index as usize);
                        } else {
                            // TODO SET DEFAULTS
                        }
                    }

                    if let Some(loc) = basic_renderer.obj_to_wld_loc.into() {
                        let obj_to_wld = Matrix4::from_translation(self.resources.meshes[i].translate);

                        gl.uniform_matrix4f(loc, gl::MajorAxis::Column, obj_to_wld.as_ref());
                    }

                    gl.draw_elements_base_vertex(
                        gl::TRIANGLES,
                        mesh_meta.element_count,
                        gl::UNSIGNED_INT,
                        mesh_meta.element_offset,
                        mesh_meta.vertex_base,
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
                    self.obj_to_wld_loc = get_uniform_location!(gl, name, "obj_to_wld");
                    self.cam_to_cls_loc = get_uniform_location!(gl, name, "cam_to_cls");
                    self.cluster_dims_loc = get_uniform_location!(gl, name, "cluster_dims");

                    self.diffuse_sampler_loc = get_uniform_location!(gl, name, "diffuse_sampler");
                    self.normal_sampler_loc = get_uniform_location!(gl, name, "normal_sampler");
                    self.specular_sampler_loc = get_uniform_location!(gl, name, "specular_sampler");

                    self.diffuse_dimensions_loc = get_uniform_location!(gl, name, "diffuse_dimensions");
                    self.normal_dimensions_loc = get_uniform_location!(gl, name, "normal_dimensions");
                    self.specular_dimensions_loc = get_uniform_location!(gl, name, "specular_dimensions");

                    self.display_mode_loc = get_uniform_location!(gl, name, "display_mode");
                }
            }
        }
    }

    pub fn new(context: &mut RenderingContext) -> Self {
        Renderer {
            program: vs_fs_program(context, "basic_renderer.vert", "basic_renderer.frag"),

            obj_to_wld_loc: gl::OptionUniformLocation::NONE,
            cam_to_cls_loc: gl::OptionUniformLocation::NONE,
            cluster_dims_loc: gl::OptionUniformLocation::NONE,

            diffuse_sampler_loc: gl::OptionUniformLocation::NONE,
            normal_sampler_loc: gl::OptionUniformLocation::NONE,
            specular_sampler_loc: gl::OptionUniformLocation::NONE,

            diffuse_dimensions_loc: gl::OptionUniformLocation::NONE,
            normal_dimensions_loc: gl::OptionUniformLocation::NONE,
            specular_dimensions_loc: gl::OptionUniformLocation::NONE,

            display_mode_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
