mod configuration;
mod convert;
mod gl_ext;
mod profiling;

pub(crate) use convert::*;
pub(crate) use gl_ext::*;
pub(crate) use gl_typed as gl;
pub(crate) use profiling::*;
pub(crate) use rand::prelude::*;

fn main() {
    let cfg = configuration::read("prefix-sum/configuration.toml");
    println!("{:#?}", cfg);
    dbg!(cfg.block_size());
    dbg!(cfg.block_count());

    assert_eq!(0, cfg.item_count % cfg.block_size());

    let event_loop = glutin::event_loop::EventLoop::new();
    let context = glutin::ContextBuilder::new()
        .build_headless(&event_loop, glutin::dpi::PhysicalSize::new(1920.0, 1080.0))
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

    let (shader_down, program_down) = prefix_sum_program(&gl, &cfg, Pass::Down);
    let (shader_up, program_up) = prefix_sum_program(&gl, &cfg, Pass::Up);

    let values: Vec<u32> = {
        let rng = rand::thread_rng();
        let dist = rand::distributions::Uniform::new_inclusive(cfg.input.min, cfg.input.max);
        rng.sample_iter(dist).take(cfg.item_count as usize).collect()
    };

    let cpu_result: Vec<u32> = values
        .chunks_exact(cfg.block_size() as usize)
        .flat_map(|chunk| {
            chunk.iter().scan(0, |state, &item| {
                *state += item;
                Some(*state)
            })
        })
        .collect();

    unsafe {
        let input_buffer = gl.create_buffer();
        let output_buffer = gl.create_buffer();

        gl.named_buffer_storage(input_buffer, values.vec_as_bytes(), gl::BufferStorageFlag::empty());
        gl.named_buffer_storage_reserve(
            output_buffer,
            values.vec_as_bytes().len(),
            gl::BufferStorageFlag::empty(),
        );
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
        gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, 1, output_buffer);

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

                let pass_0_count = cfg.item_count / cfg.block_size();
                gl.use_program(*program_up.as_ref());
                gl.dispatch_compute(pass_0_count, 1, 1);
                gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE);

                // let pass_1_count = pass_0_count / cfg.block_size();
                // gl.use_program(*program_down.as_ref());
                // gl.dispatch_compute(pass_1_count, 1, 1);
                // gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE);

                drop(rec);
                gl.finish();
            }
            drop(rec);
        }

        println!("querying gpu data...");

        // Query computation result.
        let mut gpu_result: Vec<u32> = std::iter::repeat(0).take(values.len()).collect();

        gl.memory_barrier(gl::MemoryBarrierFlag::BUFFER_UPDATE);
        gl.get_named_buffer_sub_data(
            &output_buffer,
            0,
            std::mem::size_of_val(&gpu_result[..]),
            gpu_result.vec_as_bytes_mut(),
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

        let b = cfg.block_size() as usize;
        for i in 0..cfg.item_count as usize / b {
            let expected = &cpu_result[i * b..(i + 1) * b];
            let actual = &gpu_result[i * b..(i + 1) * b];
            if expected != actual {
                panic!("block {} expected {:?}, got {:?}", i, expected, actual);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Pass {
    Down,
    Up,
}

impl Pass {
    pub fn header(self) -> &'static str {
        match self {
            Pass::Down => "#define PASS_DOWN\n",
            Pass::Up => "#define PASS_UP\n",
        }
    }
}

fn prefix_sum_program(gl: &gl::Gl, cfg: &configuration::Root, pass: Pass) -> (ShaderName, ProgramName) {
    let mut shader = ShaderName::new(gl, gl::COMPUTE_SHADER);
    let header = format!(
        r"
#version 430

#define LOCAL_X {}
#define LOCAL_Y {}
#define LOCAL_Z {}

#define ITEM_COUNT {}

{}
",
        cfg.local_x,
        cfg.local_y,
        cfg.local_z,
        cfg.item_count,
        pass.header()
    );
    let source = std::fs::read_to_string("resources/prefix_sum.comp").unwrap();
    shader.compile(gl, &[header.as_str(), source.as_str()]);
    if !shader.is_compiled() {
        panic!(shader.log(gl));
    }
    let mut program = ProgramName::new(gl);
    program.attach(gl, &[&shader]);
    program.link(gl);
    if !program.is_linked() {
        panic!(program.log(gl));
    }
    (shader, program)
}
