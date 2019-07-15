use crate::resources::Resources;
use crate::*;
use cgmath::*;
use gl_typed as gl;

pub struct Renderer {
    pub program: rendering::Program,
    pub light_count_loc: gl::OptionUniformLocation,
    pub debug_pass_loc: gl::OptionUniformLocation,
    pub obj_to_wld_loc: gl::OptionUniformLocation,
}

pub struct Parameters<'a> {
    pub cluster_resources: &'a cluster_shading::ClusterResources,
    pub cluster_data: &'a cluster_shading::ClusterData,
    pub configuration: &'a configuration::ClusteredLightShading,
}

impl Renderer {
    pub fn render(&mut self, gl: &gl::Gl, params: &Parameters, world: &mut World, resources: &Resources) {
        unsafe {
            self.update(gl, world);
            if let ProgramName::Linked(program_name) = self.program.name(&world.global) {
                gl.use_program(*program_name);
                gl.bind_vertex_array(resources.cluster_vao);

                let configuration = params.configuration;

                let pass = || {
                    let [xn, yn, zn]: [u32; 3] = params.cluster_data.dimensions.cast::<u32>().unwrap().into();
                    for zi in 0..zn {
                        if let Some(animate_z) = configuration.animate_z {
                            if zi != (((world.tick as f64 / DESIRED_UPS) * animate_z as f64) as u64 % zn as u64) as u32
                            {
                                continue;
                            }
                        }

                        let min_light_count = configuration.min_light_count;

                        // if let Some(animate_light_count) = configuration.animate_light_count {
                        //     let time = world.tick as f64 / DESIRED_UPS;
                        //     let delta = cluster_shading::CLUSTER_CAPACITY as u32 - min_light_count;
                        //     min_light_count = ((time * animate_light_count as f64) as u64 % delta as u64) as u32;
                        // }

                        for yi in 0..yn {
                            for xi in 0..xn {
                                let i = ((zi * yn) + yi) * xn + xi;
                                let cluster_shading::ClusterMeta { length, .. } =
                                    params.cluster_resources.cluster_meta[i as usize];
                                if length >= min_light_count {
                                    if let Some(loc) = self.light_count_loc.into() {
                                        gl.uniform_1ui(loc, length);
                                    }

                                    if let Some(loc) = self.obj_to_wld_loc.into() {
                                        let obj_to_cls =
                                            Matrix4::from_translation(Vector3::new(xi as f32, yi as f32, zi as f32));

                                        let obj_to_wld = params.cluster_data.cls_to_wld.cast().unwrap() * obj_to_cls;
                                        gl.uniform_matrix4f(loc, gl::MajorAxis::Column, obj_to_wld.as_ref());
                                    }

                                    gl.draw_elements(
                                        gl::TRIANGLES,
                                        resources.cluster_element_count,
                                        gl::UNSIGNED_INT,
                                        0,
                                    );
                                }
                            }
                        }
                    }
                };

                if let Some(loc) = self.debug_pass_loc.into() {
                    gl.uniform_1ui(loc, 0);
                }

                pass();

                gl.depth_mask(gl::FALSE);
                gl.enable(gl::BLEND);
                gl.blend_func(gl::SRC_ALPHA, gl::ONE);

                if let Some(loc) = self.debug_pass_loc.into() {
                    gl.uniform_1ui(loc, 1);
                }

                pass();

                gl.depth_mask(gl::TRUE);
                gl.disable(gl::BLEND);

                gl.unbind_vertex_array();
                gl.unuse_program();
            }
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, world: &mut World) {
        let modified = self.program.modified();
        if modified < self.program.update(gl, world) {
            if let ProgramName::Linked(name) = self.program.name(&world.global) {
                unsafe {
                    self.light_count_loc = get_uniform_location!(gl, *name, "current_light_count");
                    self.debug_pass_loc = get_uniform_location!(gl, *name, "debug_pass");
                    self.obj_to_wld_loc = get_uniform_location!(gl, *name, "obj_to_wld");
                }
            }
        }
    }

    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        Renderer {
            program: vs_fs_program(gl, world, "cluster_renderer.vert", "cluster_renderer.frag"),
            light_count_loc: gl::OptionUniformLocation::NONE,
            debug_pass_loc: gl::OptionUniformLocation::NONE,
            obj_to_wld_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
