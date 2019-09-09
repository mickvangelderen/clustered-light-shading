use renderer::profiling_by_value::{FrameProfiler, MainProfiler, RunProfiler};

fn main() {
    let run_count = 2;
    let frame_count = 3;
    let mut profiler = MainProfiler::default();

    for run_index in 0..run_count {
        {
            let context = run(RunContext {
                profiler: profiler.begin_run(run_index),
                frame_count,
            });
            // Restore non-copy values.
            profiler = context.profiler.end_run();
        }
    }
}

struct RunContext {
    profiler: RunProfiler,
    frame_count: usize,
}

fn run(context: RunContext) -> RunContext {
    let RunContext {
        mut profiler,
        frame_count,
    } = context;

    for frame_index in 0..frame_count {
        {
            let context = frame(FrameContext {
                profiler: profiler.begin_frame(frame_index),
            });
            profiler = context.profiler.end_frame();
        }
    }

    RunContext { profiler, frame_count }
}

struct FrameContext {
    profiler: FrameProfiler,
}

fn frame(context: FrameContext) -> FrameContext {
    let FrameContext { profiler } = context;

    dbg!(&profiler);

    FrameContext { profiler }
}
