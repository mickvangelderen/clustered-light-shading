pub mod configuration;
pub mod convert;
pub mod gl_ext;
pub mod profiling;

pub(crate) use convert::*;
pub(crate) use gl_ext::*;
pub(crate) use gl_typed as gl;
pub(crate) use profiling::*;
pub(crate) use rand::prelude::*;
pub(crate) use std::path::Path;

fn u32_div_ceil(a: u32, b: u32) -> u32 {
    a / b
        + match a % b {
            0 => 0,
            _ => 1,
        }
}

fn main() {
    let cfg = configuration::read("prefix-sum/configuration.toml");
    println!("{:#?}", cfg);

    let N = cfg.item_count;
    let L = cfg.local_xyz() * cfg.local_xyz();
    let C = u32_div_ceil(N, L);
    let B = C * cfg.local_xyz();
    let D = u32_div_ceil(N, B);

    dbg!(N);
    dbg!(L);
    dbg!(C);
    dbg!(B);
    dbg!(D);

    let event_loop = glutin::event_loop::EventLoop::new();
    let context = glutin::ContextBuilder::new()
        .build_windowed(glutin::window::WindowBuilder::new().with_visible(false), &event_loop)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };

    let gl = &unsafe { gl::Gl::load_with(|s| context.get_proc_address(s) as *const _) };

    fn symbol_u32<S: gl::Symbol<u32>>(_symbol: S) -> u32 {
        S::VALUE
    }

    unsafe {
        let mut warp_size = 0u32;
        gl.get_integer_v(symbol_u32(gl::WARP_SIZE_NV), std::slice::from_mut(&mut warp_size));
        let mut warps_per_sm = 0u32;
        gl.get_integer_v(symbol_u32(gl::WARPS_PER_SM_NV), std::slice::from_mut(&mut warps_per_sm));
        let mut sm_count = 0u32;
        gl.get_integer_v(symbol_u32(gl::SM_COUNT_NV), std::slice::from_mut(&mut sm_count));

        dbg!(warp_size);
        dbg!(warps_per_sm);
        dbg!(sm_count);
    }

    let epoch = &std::time::Instant::now();

    let mut total_profiler = Profiler::new(gl);
    let mut profilers: Vec<Profiler> = std::iter::repeat_with(|| Profiler::new(gl))
        .take(cfg.iterations as usize)
        .collect();

    let (_s0, ps0) = prefix_sum_program(&gl, &cfg, "resources/ps0.comp");
    let (_s1, ps1) = prefix_sum_program(&gl, &cfg, "resources/ps1.comp");
    let (_s2, ps2) = prefix_sum_program(&gl, &cfg, "resources/ps2.comp");

    let values: Vec<u32> = {
        let rng = rand::thread_rng();
        let dist = rand::distributions::Uniform::new_inclusive(cfg.input.min, cfg.input.max);
        rng.sample_iter(dist).take(N as usize).collect()
    };

    unsafe {
        let input_buffer = gl.create_buffer();
        let offset_buffer = gl.create_buffer();
        let output_buffer = gl.create_buffer();

        let buffer_flags =
            gl::BufferStorageFlag::DYNAMIC_STORAGE | gl::BufferStorageFlag::READ | gl::BufferStorageFlag::WRITE;
        gl.named_buffer_storage(input_buffer, values.vec_as_bytes(), buffer_flags);
        gl.named_buffer_storage_reserve(
            offset_buffer,
            std::mem::size_of::<u32>() * cfg.local_xyz() as usize,
            buffer_flags,
        );
        gl.named_buffer_storage_reserve(output_buffer, values.vec_as_bytes().len(), buffer_flags);
        gl.clear_named_buffer_sub_data(
            output_buffer,
            gl::R32UI,
            0,
            values.vec_as_bytes().len(),
            gl::RED,
            gl::UNSIGNED_INT,
            None,
        );

        gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, 0, input_buffer);
        gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, 1, offset_buffer);
        gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, 2, output_buffer);

        {
            let rec = total_profiler.record(gl, epoch);
            // for tick in 0.. {
            //     let profilers_len = profilers.len();
            //     let profiler = &mut profilers[tick % profilers_len];

            //     // Clear the profiler.
            //     if let Some(span) = profiler.try_query(gl) {
            //         println!(
            //             "{}: {:?} CPU | {:?} GPU",
            //             tick - profilers_len,
            //             Ns(span.cpu.delta()),
            //             Ns(span.gpu.delta()),
            //         );
            //     }

            //     let rec = profiler.record(gl, epoch);
            //     gl.dispatch_compute(1, 1, 1);
            //     drop(rec);
            //     gl.finish();
            // }

            for profiler in profilers.iter_mut() {
                let rec = profiler.record(gl, epoch);

                gl.use_program(*ps0.as_ref());
                gl.dispatch_compute(D, 1, 1);
                gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);

                gl.use_program(*ps1.as_ref());
                gl.dispatch_compute(1, 1, 1);
                gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE);

                gl.use_program(*ps2.as_ref());
                gl.dispatch_compute(D, 1, 1);
                gl.memory_barrier(gl::MemoryBarrierFlag::SHADER_STORAGE | gl::MemoryBarrierFlag::BUFFER_UPDATE);

                drop(rec);
                context.swap_buffers().unwrap();
            }
            drop(rec);
        }

        println!("querying gpu data...");

        // Query computation result.
        gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE);

        let mut gpu_offsets: Vec<u32> = std::iter::repeat(0).take(cfg.local_xyz() as usize).collect();

        gl.get_named_buffer_sub_data(
            &offset_buffer,
            0,
            gpu_offsets.vec_as_bytes().len(),
            gpu_offsets.vec_as_bytes_mut(),
        );

        let mut gpu_values: Vec<u32> = std::iter::repeat(0).take(values.len()).collect();

        gl.get_named_buffer_sub_data(
            &output_buffer,
            0,
            gpu_values.vec_as_bytes().len(),
            gpu_values.vec_as_bytes_mut(),
        );

        let total = total_profiler.query(gl);
        let time_spans: Vec<GpuCpuTimeSpan> = profilers.iter_mut().map(|profiler| profiler.query(gl)).collect();

        println!(
            "{} iterations: {:?} CPU | {:?} GPU",
            time_spans.len(),
            Ns(total.cpu.delta()),
            Ns(total.gpu.delta()),
        );

        let cpu_sum: u64 = time_spans.iter().map(|GpuCpuTimeSpan { cpu, .. }| cpu.delta()).sum();
        let gpu_sum: u64 = time_spans.iter().map(|GpuCpuTimeSpan { gpu, .. }| gpu.delta()).sum();
        println!(
            "iteration avg {:?} CPU | {:?} GPU",
            Ns(cpu_sum / time_spans.len() as u64),
            Ns(gpu_sum / time_spans.len() as u64),
        );

        let cpu_min = time_spans
            .iter()
            .map(|GpuCpuTimeSpan { cpu, .. }| cpu.delta())
            .min()
            .unwrap();
        let gpu_min = time_spans
            .iter()
            .map(|GpuCpuTimeSpan { gpu, .. }| gpu.delta())
            .min()
            .unwrap();
        println!("iteration min {:?} CPU | {:?} GPU", Ns(cpu_min), Ns(gpu_min),);

        let cpu_max = time_spans
            .iter()
            .map(|GpuCpuTimeSpan { cpu, .. }| cpu.delta())
            .max()
            .unwrap();
        let gpu_max = time_spans
            .iter()
            .map(|GpuCpuTimeSpan { gpu, .. }| gpu.delta())
            .max()
            .unwrap();
        println!("iteration max {:?} CPU | {:?} GPU", Ns(cpu_max), Ns(gpu_max),);

        let cpu_offsets: Vec<u32> = values
            .chunks(B as usize)
            .map(|chunk| chunk.iter().sum::<u32>())
            .scan(0, |state, item| {
                *state += item;
                Some(*state)
            })
            .collect();

        assert_eq!(D, cpu_offsets.len() as u32);

        assert_eq!(cpu_offsets[0..D as usize], gpu_offsets[0..D as usize]);

        let cpu_values: Vec<u32> = values
            .iter()
            .scan(0, |state, &item| {
                *state += item;
                Some(*state)
            })
            .collect();

        assert_eq!(cpu_values, gpu_values);
    }
}

fn prefix_sum_program(gl: &gl::Gl, cfg: &configuration::Root, path: impl AsRef<Path>) -> (ShaderName, ProgramName) {
    let mut shader = ShaderName::new(gl, gl::COMPUTE_SHADER);
    let header = format!(
        r"
#version 430

#define LOCAL_X {}
#define LOCAL_Y {}
#define LOCAL_Z {}
#define LOCAL_XYZ {}

#define ITEM_COUNT {}
",
        cfg.local_x,
        cfg.local_y,
        cfg.local_z,
        cfg.local_xyz(),
        cfg.item_count,
    );
    let source = std::fs::read_to_string(path.as_ref()).unwrap();
    shader.compile(gl, &[header.as_str(), source.as_str()]);
    if !shader.is_compiled() {
        panic!("{}: {}", path.as_ref().display(), shader.log(gl));
    }
    let mut program = ProgramName::new(gl);
    program.attach(gl, &[&shader]);
    program.link(gl);
    if !program.is_linked() {
        panic!("{}: {}", path.as_ref().display(), program.log(gl));
    }
    (shader, program)
}
