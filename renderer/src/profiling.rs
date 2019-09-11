mod indices;

use gl_typed as gl;
use ProfilingConfiguration as Configuration;
use ProfilingContext as Context;

use indices::ProfilerIndex;
pub use indices::{FrameIndex, RunIndex, SampleIndex};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct ProfilingConfiguration {
    pub display: bool,
    pub path: Option<std::path::PathBuf>,
}

enum FrameEvent {
    BeginTimeSpan(SampleIndex),
    EndTimeSpan,
    // QueryValue(SampleIndex),
}

#[derive(Default)]
struct FrameContext {
    frame_index: FrameIndex,
    events: Vec<FrameEvent>,
    profilers_used: usize,
    profilers: Vec<TimeSpanProfiler>,
}

const FRAME_CAPACITY: usize = 3;

#[derive(Default)]
struct FrameContextRing([FrameContext; 3]);

impl std::ops::Index<FrameIndex> for FrameContextRing {
    type Output = FrameContext;

    fn index(&self, index: FrameIndex) -> &Self::Output {
        &self.0[index.to_usize() % 3]
    }
}

impl std::ops::IndexMut<FrameIndex> for FrameContextRing {
    fn index_mut(&mut self, index: FrameIndex) -> &mut Self::Output {
        &mut self.0[index.to_usize() % 3]
    }
}

pub struct ProfilingContext {
    epoch: std::time::Instant,
    frame_context_ring: FrameContextRing,
    frame_index: Option<FrameIndex>,
    run_index: Option<RunIndex>,
    sample_data: Vec<Option<GpuCpuTimeSpan>>,
    thread: ProfilingThread,
}

struct ProfilingThreadInner {
    handle: std::thread::JoinHandle<()>,
    tx: std::sync::mpsc::Sender<Option<MeasurementEvent>>,
}

pub struct ProfilingThread(Option<ProfilingThreadInner>);

impl ProfilingThread {
    fn emit(&mut self, event: MeasurementEvent) {
        if let Some(thread) = self.0.as_mut() {
            thread.tx.send(Some(event)).unwrap();
        }
    }
}

impl Drop for ProfilingThread {
    fn drop(&mut self) {
        if let Some(thread) = self.0.take() {
            thread.tx.send(None).unwrap();
            thread.handle.join().unwrap();
        }
    }
}

impl Context {
    pub fn new(configuration: &Configuration) -> Self {
        let thread = ProfilingThread(configuration.path.as_ref().map(|path| {
            let mut file = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
            let (tx, rx) = std::sync::mpsc::channel();
            let handle = std::thread::Builder::new()
                .name("profiling".to_string())
                .spawn(move || {
                    while let Some(event) = rx.recv().unwrap() {
                        bincode::serialize_into(&mut file, &event).unwrap();
                    }
                })
                .unwrap();
            ProfilingThreadInner { handle, tx }
        }));

        Self {
            epoch: std::time::Instant::now(),
            frame_context_ring: Default::default(),
            run_index: None,
            frame_index: None,
            sample_data: Vec::new(),
            thread,
        }
    }

    #[inline]
    pub fn add_sample(&mut self, sample: &'static str) -> SampleIndex {
        let sample_index = SampleIndex::from_usize(self.sample_data.len());
        self.thread
            .emit(MeasurementEvent::SampleName(sample_index, sample.to_string()));
        self.sample_data.push(None);
        sample_index
    }

    #[inline]
    pub fn begin_run(&mut self, run_index: RunIndex) {
        assert!(self.run_index.replace(run_index).is_none());
        self.thread.emit(MeasurementEvent::BeginRun(run_index));
    }

    #[inline]
    pub fn end_run(&mut self, run_index: RunIndex) {
        assert_eq!(self.run_index.take(), Some(run_index));
        self.thread.emit(MeasurementEvent::EndRun);
    }

    #[inline]
    pub fn begin_frame(&mut self, gl: &gl::Gl, frame_index: FrameIndex) {
        assert!(self.run_index.is_some());
        assert!(
            self.frame_index.is_none(),
            "Tried to start frame_index without stopping the frame_index!"
        );
        self.thread.emit(MeasurementEvent::BeginFrame(frame_index));
        let context = &mut self.frame_context_ring[frame_index];

        // Clear values from all samples.
        for sample in self.sample_data.iter_mut() {
            *sample = None;
        }

        if frame_index.to_usize() >= FRAME_CAPACITY {
            debug_assert_eq!(context.frame_index.to_usize() + FRAME_CAPACITY, frame_index.to_usize());

            // Read back data from the GPU.
            let mut profilers_used = 0;
            for event in context.events.iter() {
                match *event {
                    FrameEvent::BeginTimeSpan(sample_index) => {
                        let profiler_index = profilers_used;
                        profilers_used += 1;
                        debug_assert!(
                            self.sample_data[sample_index.to_usize()].is_none(),
                            "{:?} is writen to more than once",
                            sample_index
                        );
                        let sample = context.profilers[profiler_index].read(gl).unwrap();
                        self.thread.emit(MeasurementEvent::BeginTimeSpan(sample_index, sample));
                        self.sample_data[sample_index.to_usize()] = Some(sample);
                    }
                    FrameEvent::EndTimeSpan => self.thread.emit(MeasurementEvent::EndTimeSpan), // FrameEvent::QueryValue(sample_index) => {
                                                                                                //     unimplemented!();
                                                                                                // }
                }
            }

            debug_assert_eq!(profilers_used, context.profilers_used);
        }

        context.events.clear();
        context.profilers_used = 0;
        context.frame_index = frame_index;

        self.frame_index = Some(frame_index);
    }

    #[inline]
    pub fn end_frame(&mut self, frame_index: FrameIndex) {
        assert!(self.run_index.is_some());
        assert_eq!(self.frame_index.take(), Some(frame_index));
    }

    #[inline]
    pub fn start(&mut self, gl: &gl::Gl, sample_index: SampleIndex) -> ProfilerIndex {
        let frame_index = self
            .frame_index
            .expect("Tried to start a timer before starting the frame_index!");
        let context = &mut self.frame_context_ring[frame_index];
        context.events.push(FrameEvent::BeginTimeSpan(sample_index));
        let profiler_index = ProfilerIndex(context.profilers_used);
        context.profilers_used += 1;
        while context.profilers.len() < profiler_index.0 + 1 {
            context.profilers.push(TimeSpanProfiler::new(gl));
        }
        context.profilers[profiler_index.0].start(gl, self.epoch);
        profiler_index
    }

    #[inline]
    pub fn stop(&mut self, gl: &gl::Gl, profiler_index: ProfilerIndex) {
        let frame_index = self
            .frame_index
            .expect("Tried to start a timer before starting the frame_index!");
        let context = &mut self.frame_context_ring[frame_index];
        context.events.push(FrameEvent::EndTimeSpan);
        context.profilers[profiler_index.0].stop(gl, self.epoch);
    }

    #[inline]
    pub fn sample(&mut self, sample_index: SampleIndex) -> Option<GpuCpuTimeSpan> {
        assert!(self.frame_index.is_some());
        self.sample_data[sample_index.to_usize()]
    }

    pub fn reset(&mut self) {
        // Clear values from all samples.
        for sample in self.sample_data.iter_mut() {
            *sample = None;
        }
    }
}

pub type Epoch = std::time::Instant;

#[derive(serde::Deserialize, serde::Serialize, Debug, Copy, Clone, Default)]
pub struct TimeSpan {
    pub begin: u64,
    pub end: u64,
}

impl TimeSpan {
    pub fn delta(&self) -> u64 {
        // I'd rather see a 0 somewhere than crash when profiling timers overflow.
        self.end.saturating_sub(self.begin)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Copy, Clone, Default)]
pub struct GpuCpuTimeSpan {
    pub gpu: TimeSpan,
    pub cpu: TimeSpan,
}

#[derive(Debug)]
pub struct TimeSpanProfiler {
    begin_query_name: gl::QueryName,
    end_query_name: gl::QueryName,
    state: State,
}

#[derive(Debug)]
enum State {
    Empty,
    Started { cpu_begin: u64 },
    Stopped { cpu_begin: u64, cpu_end: u64 },
}

impl TimeSpanProfiler {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            begin_query_name: unsafe { gl.create_query(gl::TIMESTAMP) },
            end_query_name: unsafe { gl.create_query(gl::TIMESTAMP) },
            state: State::Empty,
        }
    }

    #[inline]
    pub fn start(&mut self, gl: &gl::Gl, epoch: Epoch) {
        self.state = match self.state {
            State::Empty | State::Stopped { .. } => {
                unsafe {
                    gl.query_counter(self.begin_query_name);
                }
                State::Started {
                    cpu_begin: epoch.elapsed().as_nanos() as u64,
                }
            }
            State::Started { .. } => {
                panic!("Tried to start a profiler that had already been started!");
            }
        };
    }

    #[inline]
    pub fn stop(&mut self, gl: &gl::Gl, epoch: Epoch) {
        self.state = match self.state {
            State::Empty => {
                panic!("Tried to stop a profiler that was never started!");
            }
            State::Started { cpu_begin } => {
                unsafe {
                    gl.query_counter(self.end_query_name);
                }
                State::Stopped {
                    cpu_begin,
                    cpu_end: epoch.elapsed().as_nanos() as u64,
                }
            }
            State::Stopped { .. } => {
                panic!("Tried to stop a profiler that had already been stopped!");
            }
        }
    }

    #[inline]
    pub fn read(&mut self, gl: &gl::Gl) -> Option<GpuCpuTimeSpan> {
        match self.state {
            State::Empty => None,
            State::Started { .. } => {
                panic!("Tried to read a profiler that was started but never stopped!");
            }
            State::Stopped { cpu_begin, cpu_end } => {
                // Not really necessary but I wan't to catch double reads.
                self.state = State::Empty;

                let (gpu_begin, gpu_end) = unsafe {
                    (
                        gl.try_query_result_u64(self.begin_query_name)
                            .expect("Query result was not ready!"),
                        gl.try_query_result_u64(self.end_query_name)
                            .expect("Query result was not ready!"),
                    )
                };

                Some(GpuCpuTimeSpan {
                    gpu: TimeSpan {
                        begin: gpu_begin.get(),
                        end: gpu_end.get(),
                    },
                    cpu: TimeSpan {
                        begin: cpu_begin,
                        end: cpu_end,
                    },
                })
            }
        }
    }
}

// pub struct TimeSpanProfilerPool {
//     pool: [TimeSpanProfiler; Self::CAPACITY],
// }

// impl TimeSpanProfilerPool {
//     pub const CAPACITY: usize = 3;

//     pub fn new(gl: &gl::Gl) -> Self {
//         Self {
//             pool: [TimeSpanProfiler::new(gl), TimeSpanProfiler::new(gl), TimeSpanProfiler::new(gl)],
//         }
//     }
// }

// impl std::ops::Index<FrameIndex> for TimeSpanProfilerPool {
//     type Output = TimeSpanProfiler;

//     #[inline]
//     fn index(&self, frame_index: FrameIndex) -> &Self::Output {
//         &self.pool[(frame_index.0 % Self::CAPACITY as u64) as usize]
//     }
// }

// impl std::ops::IndexMut<FrameIndex> for TimeSpanProfilerPool {
//     #[inline]
//     fn index_mut(&mut self, frame_index: FrameIndex) -> &mut Self::Output {
//         &mut self.pool[(frame_index.0 % Self::CAPACITY as u64) as usize]
//     }
// }

// /// Stores profiling samples in a circular buffer. Will clear the buffer when
// /// samples aren't inserted consecutively.
// pub struct ProfilerSampleBuffer {
//     samples: [GpuCpuTimeSpan; Self::CAPACITY],
//     /// First frame_index of the consecutive samples.
//     origin_frame: FrameIndex,
//     /// Total number of consecutive samples stored, can be larger than `Self::CAPACITY`.
//     count: usize,
// }

// impl ProfilerSampleBuffer {
//     pub const CAPACITY: usize = 8;

//     pub fn new() -> Self {
//         Self {
//             samples: [GpuCpuTimeSpan::default(); Self::CAPACITY],
//             origin_frame: FrameIndex(0),
//             count: 0,
//         }
//     }

//     pub fn update(&mut self, sample: GpuCpuTimeSpan) {
//         if self.origin_frame + self.count as u64 == sample.frame_index {
//             self.count += 1;
//         } else {
//             self.origin_frame = sample.frame_index;
//             self.count = 1;
//         }
//         self.samples[(self.count - 1) % Self::CAPACITY] = sample;
//     }

//     fn len(&self) -> usize {
//         std::cmp::min(self.count, Self::CAPACITY)
//     }

//     fn first_frame(&self) -> Option<FrameIndex> {
//         if self.count > 0 {
//             Some(if self.count <= Self::CAPACITY {
//                 self.origin_frame
//             } else {
//                 self.origin_frame + (self.count - Self::CAPACITY) as u64
//             })
//         } else {
//             None
//         }
//     }

//     fn last_frame(&self) -> Option<FrameIndex> {
//         if self.count > 0 {
//             Some(self.origin_frame + self.count as u64 - 1)
//         } else {
//             None
//         }
//     }

//     fn latest_sample(&self) -> Option<GpuCpuTimeSpan> {
//         if self.count > 0 {
//             Some(self.samples[(self.count - 1) % Self::CAPACITY])
//         } else {
//             None
//         }
//     }

//     pub fn stats(&self) -> Option<GpuCpuStats> {
//         let mut iter = self.samples[0..self.len()].iter();
//         let first = iter.next();
//         first.map(move |first| {
//             let dg = first.gpu.delta();
//             let dc = first.cpu.delta();
//             let mut stats = iter.fold(
//                 GpuCpuStats {
//                     first_frame: self.first_frame().unwrap(),
//                     last_frame: self.last_frame().unwrap(),
//                     gpu_elapsed_avg: dg,
//                     gpu_elapsed_min: dg,
//                     gpu_elapsed_max: dg,
//                     cpu_elapsed_avg: dc,
//                     cpu_elapsed_min: dc,
//                     cpu_elapsed_max: dc,
//                 },
//                 |mut stats, item| {
//                     {
//                         let dg = item.gpu.delta();
//                         stats.gpu_elapsed_avg += dg;
//                         if dg < stats.gpu_elapsed_min {
//                             stats.gpu_elapsed_min = dg;
//                         }
//                         if dg > stats.gpu_elapsed_max {
//                             stats.gpu_elapsed_max = dg;
//                         }
//                     }
//                     {
//                         let dc = item.cpu.delta();
//                         stats.cpu_elapsed_avg += dc;
//                         if dc < stats.cpu_elapsed_min {
//                             stats.cpu_elapsed_min = dc;
//                         }
//                         if dc > stats.cpu_elapsed_max {
//                             stats.cpu_elapsed_max = dc;
//                         }
//                     }
//                     stats
//                 },
//             );

//             stats.gpu_elapsed_avg /= Self::CAPACITY as u64;
//             stats.cpu_elapsed_avg /= Self::CAPACITY as u64;

//             stats
//         })
//     }
// }

// pub struct GpuCpuStats {
//     pub first_frame: FrameIndex,
//     pub last_frame: FrameIndex,
//     pub gpu_elapsed_avg: u64,
//     pub gpu_elapsed_min: u64,
//     pub gpu_elapsed_max: u64,
//     pub cpu_elapsed_avg: u64,
//     pub cpu_elapsed_min: u64,
//     pub cpu_elapsed_max: u64,
// }

// pub struct Profiler {
//     pub scope: &'static str,
//     timers: TimeSpanProfilerPool,
//     samples: ProfilerSampleBuffer,
// }

// impl Profiler {
//     pub fn new(gl: &gl::Gl, scope: &'static str) -> Self {
//         Self {
//             scope,
//             timers: TimeSpanProfilerPool::new(gl),
//             samples: ProfilerSampleBuffer::new(),
//         }
//     }

//     pub fn start(&mut self, gl: &gl::Gl, frame_index: FrameIndex, epoch: Epoch) {
//         let timer = &mut self.timers[frame_index];
//         if let Some(sample) = timer.read(gl) {
//             self.samples.update(sample);
//         }
//         timer.start(&gl, epoch, frame_index);
//     }

//     pub fn stop(&mut self, gl: &gl::Gl, frame_index: FrameIndex, epoch: Epoch) {
//         let timer = &mut self.timers[frame_index];
//         timer.stop(&gl, epoch);
//     }

//     pub fn current_sample(&self, frame_index: FrameIndex) -> Option<GpuCpuTimeSpan> {
//         if self.samples.origin_frame + self.samples.count as u64 + TimeSpanProfilerPool::CAPACITY as u64 == frame_index + 1 {
//             self.samples.latest_sample()
//         } else {
//             None
//         }
//     }

//     /// Returns Self::stats if the stats are available the latest possible frame_index, None otherwise.
//     pub fn current_stats(&self, frame_index: FrameIndex) -> Option<GpuCpuStats> {
//         if self.samples.origin_frame + self.samples.count as u64 + TimeSpanProfilerPool::CAPACITY as u64 == frame_index + 1 {
//             self.stats()
//         } else {
//             None
//         }
//     }

//     pub fn stats(&self) -> Option<GpuCpuStats> {
//         self.samples.stats()
//     }
// }

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum MeasurementEvent {
    SampleName(SampleIndex, String),
    BeginRun(RunIndex),
    EndRun,
    BeginFrame(FrameIndex),
    EndFrame,
    BeginTimeSpan(SampleIndex, GpuCpuTimeSpan),
    EndTimeSpan,
    QueryValue(SampleIndex, u64),
}
