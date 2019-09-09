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

    pub fn run_index(&self) -> usize {
        self.run_index
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

    pub fn run_index(&self) -> usize {
        self.parent.run_index
    }

    pub fn frame_index(&self) -> usize {
        self.frame_index
    }
}
