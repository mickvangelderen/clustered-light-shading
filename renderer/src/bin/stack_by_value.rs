mod profiling {
    #[derive(Debug)]
    pub enum ProfilingEvent {
        BeginRun(usize),
        EndRun(usize),
        BeginFrame(usize),
        EndFrame(usize),
    }

    #[derive(Debug, Default)]
    pub struct MainProfiler {
        events: Vec<ProfilingEvent>,
    }

    impl MainProfiler {
        pub fn begin_run(mut self, run_index: usize) -> RunProfiler {
            self.emit(ProfilingEvent::BeginRun(run_index));
            RunProfiler {
                parent: self,
                run_index,
            }
        }

        pub fn emit(&mut self, event: ProfilingEvent) {
            self.events.push(event);
        }
    }

    #[derive(Debug)]
    pub struct RunProfiler {
        parent: MainProfiler,
        run_index: usize,
    }

    impl RunProfiler {
        pub fn end_run(self) -> MainProfiler {
            let Self { mut parent, run_index } = self;
            parent.emit(ProfilingEvent::EndRun(run_index));
            parent
        }

        pub fn begin_frame(mut self, frame_index: usize) -> FrameProfiler {
            self.emit(ProfilingEvent::BeginFrame(frame_index));
            FrameProfiler {
                parent: self,
                frame_index,
            }
        }

        pub fn emit(&mut self, event: ProfilingEvent) {
            self.parent.emit(event);
        }
    }

    #[derive(Debug)]
    pub struct FrameProfiler {
        parent: RunProfiler,
        frame_index: usize,
    }

    impl FrameProfiler {
        pub fn end_frame(self) -> RunProfiler {
            let Self {
                mut parent,
                frame_index,
            } = self;
            parent.emit(ProfilingEvent::EndFrame(frame_index));
            parent
        }

        pub fn emit(&mut self, event: ProfilingEvent) {
            self.parent.emit(event);
        }
    }
}

use profiling::{FrameProfiler, MainProfiler, RunProfiler};

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
