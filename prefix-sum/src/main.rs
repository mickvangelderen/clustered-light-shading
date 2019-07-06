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

    let event_loop = glutin::event_loop::EventLoop::new();
    let context = glutin::ContextBuilder::new()
        .build_headless(&event_loop, glutin::dpi::PhysicalSize::new(1920.0, 1080.0))
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };

    let gl = &unsafe { gl::Gl::load_with(|s| context.get_proc_address(s) as *const _) };

    let epoch = &std::time::Instant::now();

    let mut total_profiler = Profiler::new(gl);
    let mut profilers: Vec<Profiler> = std::iter::repeat_with(|| Profiler::new(gl)).take(1000).collect();

    let (shader, program) = {
        let mut shader = ShaderName::new(gl, gl::COMPUTE_SHADER);
        let header = format!(
            r"
#version 430

#define LOCAL_X {}
#define LOCAL_Y {}
#define LOCAL_Z {}

#define ITEM_COUNT {}
",
            cfg.local_x, cfg.local_y, cfg.local_z, cfg.item_count,
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
    };

    let rng = rand::thread_rng();
    let dist = rand::distributions::Uniform::new_inclusive(0, 200);

    // let values: Vec<u32> = rng.sample_iter(dist).take(32 * 4).collect();
    let values: Vec<u32> = std::iter::repeat(1).take(cfg.item_count).collect();
    let cpu_result: Vec<u32> = values
        .iter()
        .scan(0, |state, &item| {
            *state += item;
            Some(*state)
        })
        .collect();

    unsafe {
        let input_buffer = gl.create_buffer();
        let output_buffer = gl.create_buffer();

        gl.named_buffer_data(input_buffer, values.vec_as_bytes(), gl::STATIC_DRAW);

        gl.named_buffer_reserve(output_buffer, values.vec_as_bytes().len(), gl::DYNAMIC_DRAW);

        gl.use_program(*program.as_ref());
        gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, 0, input_buffer);
        gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, 1, output_buffer);

        {
            let rec = total_profiler.record(gl, epoch);
            for profiler in profilers.iter_mut() {
                let rec = profiler.record(gl, epoch);
                gl.dispatch_compute(1, 1, 1);
                drop(rec);
            }
            drop(rec);
        }

        // Query computation result.
        let mut gpu_result: Vec<u32> = std::iter::repeat(0).take(values.len()).collect();
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

        assert_eq!(&cpu_result[..cfg.item_count], &gpu_result[..cfg.item_count]);
    }
}
