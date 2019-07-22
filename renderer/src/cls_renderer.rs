use crate::*;

pub struct Renderer {
    pub fragments_per_cluster_program: rendering::Program,
    pub compact_clusters_0_program: rendering::Program,
    pub compact_clusters_1_program: rendering::Program,
    pub compact_clusters_2_program: rendering::Program,
    // pub fb_dims_loc: gl::OptionUniformLocation,
}

pub struct Buffer {
    name: gl::BufferName,
    byte_capacity: usize,
}

impl Buffer {
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            Self {
                name: gl.create_buffer(),
                byte_capacity: 0,
            }
        }
    }

    pub fn ensure_capacity<T>(&mut self, gl: &gl::Gl, capacity: usize) {
        unsafe {
            let byte_capacity = std::mem::size_of::<T>() * capacity;

            if self.byte_capacity < capacity {
                gl.named_buffer_reserve(self.name, byte_capacity, gl::DYNAMIC_DRAW);
                self.byte_capacity = byte_capacity;
            }
        }
    }
}

pub struct Resources {
    pub fragments_per_cluster_buffer: Buffer,
    pub offset_buffer: Buffer,
    pub active_cluster_buffer: Buffer,
}

impl Resources {
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            fragments_per_cluster_buffer: Buffer::new(gl),
            offset_buffer: Buffer::new(gl),
            active_cluster_buffer: Buffer::new(gl),
        }
    }
}

pub struct RenderParams<'a> {
    pub gl: &'a gl::Gl,
    pub world: &'a mut World,
    pub cfg: &'a configuration::Root,
    pub resources: &'a mut Resources,
    pub depth_texture: gl::TextureName,
    pub depth_dims: Vector2<u32>,
    pub cluster_dims: Vector3<u32>,
    pub clp_to_cls: Matrix4<f32>,
}

// fragments per cluster program
const DEPTH_SAMPLER_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(0) };
const DEPTH_DIMS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(1) };
const CLP_TO_CLS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(2) };
const CLUSTER_DIMS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(3) };

const FRAGMENTS_PER_CLUSTER_BINDING: u32 = 0;

// fn fragments_per_cluster_header() -> String {
//     format!(
//         r##"
// #define DEPTH_SAMPLER_LOC {}
// #define DEPTH_DIMS_LOC {}
// #define CLP_TO_CLS_LOC {}
// #define CLUSTER_DIMS_LOC {}
// #define FRAGMENTS_PER_CLUSTER_BINDING {}
// "##,
//         DEPTH_SAMPLER_LOC.into_i32(),
//         DEPTH_DIMS_LOC.into_i32(),
//         CLP_TO_CLS_LOC.into_i32(),
//         CLUSTER_DIMS_LOC.into_i32(),
//         FRAGMENTS_PER_CLUSTER_BINDING,
//     )
// }

// // compact clusters program
// const ITEM_COUNT_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(0) };
// const ACTIVE_CLUSTER_CAPACITY: u32 = 1024 * 1024;

// fn compact_cluster_header(pass: u32) -> String {
//     format!(
//         r##"
// #define PASS {}
// #define ACTIVE_CLUSTER_CAPACITY {}
// #define ITEM_COUNT_LOC {}
// "##,
//         pass,
//         ACTIVE_CLUSTER_CAPACITY,
//         ITEM_COUNT_LOC.into_i32(),
//     )
// }

impl Renderer {
    pub fn render(&mut self, params: RenderParams) {
        let RenderParams {
            gl,
            world,
            resources,
            cfg,
            depth_texture,
            depth_dims,
            clp_to_cls,
            cluster_dims,
            ..
        } = params;

        unsafe {
            self.update(gl, world);

            let cluster_count = cluster_dims.product();

            resources
                .fragments_per_cluster_buffer
                .ensure_capacity::<u32>(gl, cluster_count as usize);

            gl.clear_named_buffer_sub_data(
                resources.fragments_per_cluster_buffer.name,
                gl::R32UI,
                0,
                resources.fragments_per_cluster_buffer.byte_capacity,
                gl::RED,
                gl::UNSIGNED_INT,
                None,
            );

            gl.bind_buffer_base(
                gl::SHADER_STORAGE_BUFFER,
                FRAGMENTS_PER_CLUSTER_BINDING,
                resources.fragments_per_cluster_buffer.name,
            );

            if let ProgramName::Linked(name) = self.fragments_per_cluster_program.name {
                gl.use_program(name);

                gl.uniform_1i(DEPTH_SAMPLER_LOC, 0);
                gl.bind_texture_unit(0, depth_texture);

                gl.uniform_2f(DEPTH_DIMS_LOC, depth_dims.cast::<f32>().unwrap().into());

                gl.uniform_matrix4f(CLP_TO_CLS_LOC, gl::MajorAxis::Column, clp_to_cls.as_ref());

                gl.uniform_3f(CLUSTER_DIMS_LOC, cluster_dims.cast::<f32>().unwrap().into());

                assert_eq!(0, depth_dims.x % 16);
                assert_eq!(0, depth_dims.y % 16);
                gl.dispatch_compute(depth_dims.x / 16, depth_dims.y / 16, 1);
            }
        }

        // unsafe {
        //     gl.named_buffer_data(context.vb, text_box.vertices.vec_as_bytes(), gl::STREAM_DRAW);
        //     gl.named_buffer_data(context.eb, text_box.indices.vec_as_bytes(), gl::STREAM_DRAW);

        //     self.update(gl, world);
        //     if let ProgramName::Linked(name) = self.program.name {
        //         gl.disable(gl::DEPTH_TEST);
        //         gl.depth_mask(gl::FALSE);
        //         gl.enable(gl::BLEND);
        //         gl.blend_func(gl::SRC_ALPHA, gl::ONE);

        //         gl.use_program(name);
        //         gl.bind_vertex_array(context.vao);

        //         if let Some(loc) = self.dimensions_loc.into() {
        //             gl.uniform_2f(loc, [world.win_size.width as f32, world.win_size.height as f32]);
        //         }

        //         if let Some(loc) = self.text_sampler_loc.into() {
        //             gl.uniform_1i(loc, 0);
        //         }

        //         if let Some(loc) = self.text_dimensions_loc.into() {
        //             gl.uniform_2f(loc, [context.meta.scale_x as f32, context.meta.scale_y as f32]);
        //         }

        //         // TODO: Handle more than 1 page.
        //         gl.bind_texture_unit(0, context.pages[0].texture_name);

        //         gl.draw_elements(gl::TRIANGLES, text_box.indices.len() as u32, gl::UNSIGNED_INT, 0);

        //         gl.unbind_vertex_array();
        //         gl.unuse_program();

        //         gl.enable(gl::DEPTH_TEST);
        //         gl.depth_mask(gl::TRUE);
        //         gl.disable(gl::BLEND);
        //     }
        // }
    }

    pub fn update(&mut self, gl: &gl::Gl, world: &mut World) {
        self.fragments_per_cluster_program.update(gl, world);
        self.compact_clusters_0_program.update(gl, world);
        self.compact_clusters_1_program.update(gl, world);
        self.compact_clusters_2_program.update(gl, world);
    }

    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        Renderer {
            fragments_per_cluster_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/fragments_per_cluster.comp"),
                )],
            ),
            compact_clusters_0_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/compact_clusters_0.comp"),
                )],
            ),
            compact_clusters_1_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/compact_clusters_1.comp"),
                )],
            ),
            compact_clusters_2_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/compact_clusters_2.comp"),
                )],
            ),
            // dimensions_loc: gl::OptionUniformLocation::NONE,
            // text_sampler_loc: gl::OptionUniformLocation::NONE,
            // text_dimensions_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
