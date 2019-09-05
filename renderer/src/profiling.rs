mod frame;

pub use frame::Frame;
use gl_typed as gl;
use ProfilingConfiguration as Configuration;
use ProfilingContext as Context;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct ProfilingConfiguration {
    pub display: bool,
    pub path: Option<std::path::PathBuf>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProfilerIndex(usize);

impl ProfilerIndex {
    #[inline]
    pub fn to_usize(&self) -> usize {
        self.0
    }
}

pub type SampleIndex = usize;

enum Event {
    Start { sample_index: SampleIndex },
    Stop,
}

#[derive(Default)]
struct FrameContext {
    frame: Frame,
    events: Vec<Event>,
    profilers_used: usize,
    profilers: Vec<ProfilerTimer>,
}

const FRAME_CAPACITY: usize = 3;

// struct FrameContextBuffer([FrameContext; 3]);

// impl std::ops::Index<Frame> for FrameContextBuffer {
//     type Output = FrameContext;

//     fn index(&self, index: Frame) -> &Self::Output {
//         &self.0[(index.0 % 3) as usize]
//     }
// }

// impl std::ops::IndexMut<Frame> for FrameContextBuffer {
//     fn index_mut(&mut self, index: Frame) -> &mut  Self::Output {
//         &mut self.0[(index.0 % 3) as usize]
//     }
// }

pub struct ProfilingContext {
    epoch: std::time::Instant,
    frames: [FrameContext; FRAME_CAPACITY],
    frame: Option<Frame>,
    sample_names: Vec<String>,
    sample_data: Vec<Option<GpuCpuTimeSpan>>,
    // sample_stack: Vec<String>,
    pub file: Option<std::io::BufWriter<std::fs::File>>,
}

impl Context {
    pub fn new(configuration: &Configuration) -> Self {
        Self {
            epoch: std::time::Instant::now(),
            frames: Default::default(),
            frame: None,
            sample_names: Vec::new(),
            sample_data: Vec::new(),
            // sample_stack: Vec::new(),
            file: configuration
                .path
                .as_ref()
                .map(|path| std::io::BufWriter::new(std::fs::File::create(path).unwrap())),
        }
    }

    #[inline]
    pub fn add_sample(&mut self, sample: &'static str) -> SampleIndex {
        let index = self.sample_names.len();
        self.sample_names.push(sample.to_string());
        //     match self.sample_stack.iter().last() {
        //     Some(sample_index) => format!("{}.{}", &self.sample_names[sample_index], sample),
        //     None => sample.to_string(),
        // });
        self.sample_data.push(None);
        index
    }

    #[inline]
    pub fn begin_frame(&mut self, gl: &gl::Gl, frame: Frame) {
        assert!(self.frame.is_none(), "Tried to start frame without stopping the frame!");
        let context = &mut self.frames[(frame.0 % FRAME_CAPACITY as u64) as usize];

        if context.frame + FRAME_CAPACITY as u64 == frame {
            // Clear values from all samples.
            for sample in self.sample_data.iter_mut() {
                *sample = None;
            }

            // Read back data from the GPU.
            let mut profilers_used = 0;
            for event in context.events.drain(..) {
                match event {
                    Event::Start { sample_index } => {
                        let profiler_index = profilers_used;
                        profilers_used += 1;
                        debug_assert!(self.sample_data[sample_index].is_none());
                        self.sample_data[sample_index] = context.profilers[profiler_index].read(gl);
                    }
                    Event::Stop => {
                    }
                }
            }

            debug_assert_eq!(profilers_used, context.profilers_used);
            context.profilers_used = 0;

            // Write out all samples.
            if let Some(file) = self.file.as_mut() {
                let entry = FileEntry::Frame(std::mem::replace(&mut self.sample_data, Vec::new()));
                bincode::serialize_into(file, &entry).unwrap();
                match entry {
                    FileEntry::Frame(sample_data) => {
                        std::mem::replace(&mut self.sample_data, sample_data);
                    }
                    _ => unreachable!(),
                }
            }

        }

        context.frame = frame;

        self.frame = Some(frame);
    }

    #[inline]
    pub fn end_frame(&mut self) {
        let _frame = self.frame.take().expect("Tried to stop a stopped frame!");
    }

    #[inline]
    pub fn start(&mut self, gl: &gl::Gl, sample_index: SampleIndex) -> ProfilerIndex {
        let frame = self.frame.expect("Tried to start a timer before starting the frame!");
        let context = &mut self.frames[(frame.0 % FRAME_CAPACITY as u64) as usize];
        context.events.push(Event::Start { sample_index });
        let profiler_index = ProfilerIndex(context.profilers_used);
        context.profilers_used += 1;
        while context.profilers.len() < profiler_index.0 + 1 {
            context.profilers.push(ProfilerTimer::new(gl));
        }
        context.profilers[profiler_index.0].start(gl, self.epoch);
        // self.sample_stack.push(sample_index);
        profiler_index
    }

    #[inline]
    pub fn stop(&mut self, gl: &gl::Gl, profiler_index: ProfilerIndex) {
        let frame = self.frame.expect("Tried to start a timer before starting the frame!");
        let context = &mut self.frames[(frame.0 % FRAME_CAPACITY as u64) as usize];
        context.events.push(Event::Stop);
        context.profilers[profiler_index.0].stop(gl, self.epoch);
        // self.sample_stack.pop();
    }

    #[inline]
    pub fn sample(&mut self, sample_index: SampleIndex) -> Option<GpuCpuTimeSpan> {
        let _frame = self.frame.expect("Tried to start a timer before starting the frame!");
        self.sample_data[sample_index]
    }
}

impl Drop for ProfilingContext {
    fn drop(&mut self) {
        if let Some(file) = self.file.as_mut() {
            let entry = FileEntry::Samples(std::mem::replace(&mut self.sample_names, Vec::new()));
            bincode::serialize_into(file, &entry).unwrap();
            match entry {
                FileEntry::Samples(sample_names) => {
                    std::mem::replace(&mut self.sample_names, sample_names);
                }
                _ => unreachable!(),
            }
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
pub struct ProfilerTimer {
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

impl ProfilerTimer {
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

// pub struct ProfilerTimerPool {
//     pool: [ProfilerTimer; Self::CAPACITY],
// }

// impl ProfilerTimerPool {
//     pub const CAPACITY: usize = 3;

//     pub fn new(gl: &gl::Gl) -> Self {
//         Self {
//             pool: [ProfilerTimer::new(gl), ProfilerTimer::new(gl), ProfilerTimer::new(gl)],
//         }
//     }
// }

// impl std::ops::Index<Frame> for ProfilerTimerPool {
//     type Output = ProfilerTimer;

//     #[inline]
//     fn index(&self, frame: Frame) -> &Self::Output {
//         &self.pool[(frame.0 % Self::CAPACITY as u64) as usize]
//     }
// }

// impl std::ops::IndexMut<Frame> for ProfilerTimerPool {
//     #[inline]
//     fn index_mut(&mut self, frame: Frame) -> &mut Self::Output {
//         &mut self.pool[(frame.0 % Self::CAPACITY as u64) as usize]
//     }
// }

// /// Stores profiling samples in a circular buffer. Will clear the buffer when
// /// samples aren't inserted consecutively.
// pub struct ProfilerSampleBuffer {
//     samples: [GpuCpuTimeSpan; Self::CAPACITY],
//     /// First frame of the consecutive samples.
//     origin_frame: Frame,
//     /// Total number of consecutive samples stored, can be larger than `Self::CAPACITY`.
//     count: usize,
// }

// impl ProfilerSampleBuffer {
//     pub const CAPACITY: usize = 8;

//     pub fn new() -> Self {
//         Self {
//             samples: [GpuCpuTimeSpan::default(); Self::CAPACITY],
//             origin_frame: Frame(0),
//             count: 0,
//         }
//     }

//     pub fn update(&mut self, sample: GpuCpuTimeSpan) {
//         if self.origin_frame + self.count as u64 == sample.frame {
//             self.count += 1;
//         } else {
//             self.origin_frame = sample.frame;
//             self.count = 1;
//         }
//         self.samples[(self.count - 1) % Self::CAPACITY] = sample;
//     }

//     fn len(&self) -> usize {
//         std::cmp::min(self.count, Self::CAPACITY)
//     }

//     fn first_frame(&self) -> Option<Frame> {
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

//     fn last_frame(&self) -> Option<Frame> {
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
//     pub first_frame: Frame,
//     pub last_frame: Frame,
//     pub gpu_elapsed_avg: u64,
//     pub gpu_elapsed_min: u64,
//     pub gpu_elapsed_max: u64,
//     pub cpu_elapsed_avg: u64,
//     pub cpu_elapsed_min: u64,
//     pub cpu_elapsed_max: u64,
// }

// pub struct Profiler {
//     pub scope: &'static str,
//     timers: ProfilerTimerPool,
//     samples: ProfilerSampleBuffer,
// }

// impl Profiler {
//     pub fn new(gl: &gl::Gl, scope: &'static str) -> Self {
//         Self {
//             scope,
//             timers: ProfilerTimerPool::new(gl),
//             samples: ProfilerSampleBuffer::new(),
//         }
//     }

//     pub fn start(&mut self, gl: &gl::Gl, frame: Frame, epoch: Epoch) {
//         let timer = &mut self.timers[frame];
//         if let Some(sample) = timer.read(gl) {
//             self.samples.update(sample);
//         }
//         timer.start(&gl, epoch, frame);
//     }

//     pub fn stop(&mut self, gl: &gl::Gl, frame: Frame, epoch: Epoch) {
//         let timer = &mut self.timers[frame];
//         timer.stop(&gl, epoch);
//     }

//     pub fn current_sample(&self, frame: Frame) -> Option<GpuCpuTimeSpan> {
//         if self.samples.origin_frame + self.samples.count as u64 + ProfilerTimerPool::CAPACITY as u64 == frame + 1 {
//             self.samples.latest_sample()
//         } else {
//             None
//         }
//     }

//     /// Returns Self::stats if the stats are available the latest possible frame, None otherwise.
//     pub fn current_stats(&self, frame: Frame) -> Option<GpuCpuStats> {
//         if self.samples.origin_frame + self.samples.count as u64 + ProfilerTimerPool::CAPACITY as u64 == frame + 1 {
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
pub enum FileEntry {
    Samples(Vec<String>),
    Frame(Vec<Option<GpuCpuTimeSpan>>),
}
