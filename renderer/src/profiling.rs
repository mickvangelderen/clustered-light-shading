use gl_typed as gl;

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Frame(pub u64);

impl Frame {
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

impl std::ops::Add<Self> for Frame {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Frame(self.0 + rhs.0)
    }
}

impl std::ops::Sub<Self> for Frame {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Frame(self.0 - rhs.0)
    }
}

impl std::ops::Add<u64> for Frame {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Frame(self.0 + rhs)
    }
}

impl std::ops::Sub<u64> for Frame {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Frame(self.0 - rhs)
    }
}

#[derive(Debug, Default)]
pub struct ProfilerCounter(usize);

#[derive(serde::Deserialize, serde::Serialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProfilerId(usize);

impl ProfilerId {
    #[inline]
    pub fn new(counter: &mut ProfilerCounter) -> Self {
        let id = Self(counter.0);
        counter.0 += 1;
        id
    }

    #[inline]
    pub fn to_usize(&self) -> usize {
        self.0
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
    pub frame: Frame,
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
    Started { frame: Frame, cpu_begin: u64 },
    Stopped { frame: Frame, cpu_begin: u64, cpu_end: u64 },
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
    pub fn start(&mut self, gl: &gl::Gl, epoch: Epoch, frame: Frame) {
        self.state = match self.state {
            State::Empty | State::Stopped { .. } => {
                unsafe {
                    gl.query_counter(self.begin_query_name);
                }
                State::Started {
                    frame: frame,
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
            State::Started { frame, cpu_begin } => {
                unsafe {
                    gl.query_counter(self.end_query_name);
                }
                State::Stopped {
                    frame,
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
            State::Stopped {
                frame,
                cpu_begin,
                cpu_end,
            } => {
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
                    frame,
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

pub struct ProfilerTimerPool {
    pool: [ProfilerTimer; Self::CAPACITY],
}

impl ProfilerTimerPool {
    pub const CAPACITY: usize = 3;

    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            pool: [ProfilerTimer::new(gl), ProfilerTimer::new(gl), ProfilerTimer::new(gl)],
        }
    }
}

impl std::ops::Index<Frame> for ProfilerTimerPool {
    type Output = ProfilerTimer;

    #[inline]
    fn index(&self, frame: Frame) -> &Self::Output {
        &self.pool[(frame.0 % Self::CAPACITY as u64) as usize]
    }
}

impl std::ops::IndexMut<Frame> for ProfilerTimerPool {
    #[inline]
    fn index_mut(&mut self, frame: Frame) -> &mut Self::Output {
        &mut self.pool[(frame.0 % Self::CAPACITY as u64) as usize]
    }
}

/// Stores profiling samples in a circular buffer. Will clear the buffer when
/// samples aren't inserted consecutively.
pub struct ProfilerSampleBuffer {
    samples: [GpuCpuTimeSpan; Self::CAPACITY],
    /// First frame of the consecutive samples.
    origin_frame: Frame,
    /// Total number of consecutive samples stored, can be larger than `Self::CAPACITY`.
    count: usize,
}

impl ProfilerSampleBuffer {
    pub const CAPACITY: usize = 8;

    pub fn new() -> Self {
        Self {
            samples: [GpuCpuTimeSpan::default(); Self::CAPACITY],
            origin_frame: Frame(0),
            count: 0,
        }
    }

    pub fn update(&mut self, sample: GpuCpuTimeSpan) {
        if self.origin_frame + self.count as u64 == sample.frame {
            self.count += 1;
        } else {
            self.origin_frame = sample.frame;
            self.count = 1;
        }
        self.samples[(self.count - 1) % Self::CAPACITY] = sample;
    }

    fn len(&self) -> usize {
        std::cmp::min(self.count, Self::CAPACITY)
    }

    fn first_frame(&self) -> Option<Frame> {
        if self.count > 0 {
            Some(if self.count <= Self::CAPACITY {
                self.origin_frame
            } else {
                self.origin_frame + (self.count - Self::CAPACITY) as u64
            })
        } else {
            None
        }
    }

    fn last_frame(&self) -> Option<Frame> {
        if self.count > 0 {
            Some(self.origin_frame + self.count as u64 - 1)
        } else {
            None
        }
    }

    fn latest_sample(&self) -> Option<GpuCpuTimeSpan> {
        if self.count > 0 {
            Some(self.samples[(self.count - 1) % Self::CAPACITY])
        } else {
            None
        }
    }

    pub fn stats(&self) -> Option<GpuCpuStats> {
        let mut iter = self.samples[0..self.len()].iter();
        let first = iter.next();
        first.map(move |first| {
            let dg = first.gpu.delta();
            let dc = first.cpu.delta();
            let mut stats = iter.fold(
                GpuCpuStats {
                    first_frame: self.first_frame().unwrap(),
                    last_frame: self.last_frame().unwrap(),
                    gpu_elapsed_avg: dg,
                    gpu_elapsed_min: dg,
                    gpu_elapsed_max: dg,
                    cpu_elapsed_avg: dc,
                    cpu_elapsed_min: dc,
                    cpu_elapsed_max: dc,
                },
                |mut stats, item| {
                    {
                        let dg = item.gpu.delta();
                        stats.gpu_elapsed_avg += dg;
                        if dg < stats.gpu_elapsed_min {
                            stats.gpu_elapsed_min = dg;
                        }
                        if dg > stats.gpu_elapsed_max {
                            stats.gpu_elapsed_max = dg;
                        }
                    }
                    {
                        let dc = item.cpu.delta();
                        stats.cpu_elapsed_avg += dc;
                        if dc < stats.cpu_elapsed_min {
                            stats.cpu_elapsed_min = dc;
                        }
                        if dc > stats.cpu_elapsed_max {
                            stats.cpu_elapsed_max = dc;
                        }
                    }
                    stats
                },
            );

            stats.gpu_elapsed_avg /= Self::CAPACITY as u64;
            stats.cpu_elapsed_avg /= Self::CAPACITY as u64;

            stats
        })
    }
}

pub struct GpuCpuStats {
    pub first_frame: Frame,
    pub last_frame: Frame,
    pub gpu_elapsed_avg: u64,
    pub gpu_elapsed_min: u64,
    pub gpu_elapsed_max: u64,
    pub cpu_elapsed_avg: u64,
    pub cpu_elapsed_min: u64,
    pub cpu_elapsed_max: u64,
}

pub struct Profiler {
    id: ProfilerId,
    timers: ProfilerTimerPool,
    samples: ProfilerSampleBuffer,
}

impl Profiler {
    pub fn new(gl: &gl::Gl, counter: &mut ProfilerCounter) -> Self {
        Self {
            id: ProfilerId::new(counter),
            timers: ProfilerTimerPool::new(gl),
            samples: ProfilerSampleBuffer::new(),
        }
    }

    pub fn id(&self) -> ProfilerId {
        self.id
    }

    pub fn start(&mut self, gl: &gl::Gl, frame: Frame, epoch: Epoch) {
        let timer = &mut self.timers[frame];
        if let Some(sample) = timer.read(gl) {
            self.samples.update(sample);
        }
        timer.start(&gl, epoch, frame);
    }

    pub fn stop(&mut self, gl: &gl::Gl, frame: Frame, epoch: Epoch) {
        let timer = &mut self.timers[frame];
        timer.stop(&gl, epoch);
    }

    pub fn current_sample(&self, frame: Frame) -> Option<FileEntry> {
        if self.samples.origin_frame + self.samples.count as u64 + ProfilerTimerPool::CAPACITY as u64 == frame + 1 {
            self.samples
                .latest_sample()
                .map(|sample| FileEntry::Sample { id: self.id, sample })
        } else {
            None
        }
    }

    /// Returns Self::stats if the stats are available the latest possible frame, None otherwise.
    pub fn current_stats(&self, frame: Frame) -> Option<GpuCpuStats> {
        if self.samples.origin_frame + self.samples.count as u64 + ProfilerTimerPool::CAPACITY as u64 == frame + 1 {
            self.stats()
        } else {
            None
        }
    }

    pub fn stats(&self) -> Option<GpuCpuStats> {
        self.samples.stats()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum FileEntry {
    Profiler { id: ProfilerId, name: String },
    Sample { id: ProfilerId, sample: GpuCpuTimeSpan },
}
