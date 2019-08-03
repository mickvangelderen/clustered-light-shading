use crate::*;

// pub struct QueryPool {
//     names: [gl::QueryName; 2],
// }

// impl QueryPool {
//     pub fn new(gl: &gl::Gl) -> Self {
//         unsafe {
//             Self {
//                 names: [
//                     gl.create_query(gl::TIMESTAMP),
//                     gl.create_query(gl::TIMESTAMP),
//                     // gl.create_query(gl::TIMESTAMP),
//                     // gl.create_query(gl::TIMESTAMP),
//                 ],
//             }
//         }
//     }

//     pub fn now(&self, gl: &gl::Gl, tick: u64) -> Option<NonZeroU64> {
//         unsafe {
//             let name = &self.names[tick as usize % self.names.len()];
//             let result = gl.try_query_result_u64(name);
//             gl.query_counter(name);
//             result
//         }
//     }
// }

#[derive(Debug, Copy, Clone, Default)]
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

#[derive(Debug, Copy, Clone, Default)]
pub struct GpuCpuTimeSpan {
    pub gpu: TimeSpan,
    pub cpu: TimeSpan,
}

#[derive(Debug)]
pub struct ProfilerTimer {
    begin_query_name: gl::QueryName,
    end_query_name: gl::QueryName,
    cpu_begin: Option<NonZeroU64>,
    cpu_end: Option<NonZeroU64>,
}

impl ProfilerTimer {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            begin_query_name: unsafe { gl.create_query(gl::TIMESTAMP) },
            end_query_name: unsafe { gl.create_query(gl::TIMESTAMP) },
            cpu_begin: None,
            cpu_end: None,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.cpu_begin = None;
        self.cpu_end = None;
    }

    #[inline]
    pub fn start(&mut self, gl: &gl::Gl, epoch: Instant) {
        unsafe {
            gl.query_counter(self.begin_query_name);
        }
        debug_assert!(self.cpu_begin.is_none(), "Profiler was started more than once!");
        self.cpu_begin = Some(NonZeroU64::new(epoch.elapsed().as_nanos() as u64).unwrap());
    }

    #[inline]
    pub fn stop(&mut self, gl: &gl::Gl, epoch: Instant) {
        unsafe {
            gl.query_counter(self.end_query_name);
        }
        debug_assert!(self.cpu_begin.is_some(), "Profiler was stopped before it was started!");
        debug_assert!(self.cpu_end.is_none(), "Profiler was stopped more than once!");
        self.cpu_end = Some(NonZeroU64::new(epoch.elapsed().as_nanos() as u64).unwrap());
    }

    #[inline]
    pub fn read(&mut self, gl: &gl::Gl) -> Option<GpuCpuTimeSpan> {
        unsafe {
            self.cpu_begin.take().map(|cpu_begin| {
                let gpu_begin = gl.try_query_result_u64(self.begin_query_name).expect("Query result was not ready!");
                let gpu_end = gl.try_query_result_u64(self.end_query_name).expect("Query result was not ready!");
                GpuCpuTimeSpan {
                    gpu: TimeSpan {
                        begin: gpu_begin.get(),
                        end: gpu_end.get(),
                    },
                    cpu: TimeSpan {
                        begin: cpu_begin.get(),
                        end: self.cpu_end.take().expect("Profiler was started but never stopped!").get(),
                    }
                }
            })
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
            pool: [
                ProfilerTimer::new(gl),
                ProfilerTimer::new(gl),
                ProfilerTimer::new(gl),
            ]
        }
    }
}

impl std::ops::Index<u64> for ProfilerTimerPool {
    type Output = ProfilerTimer;

    #[inline]
    fn index(&self, frame: u64) -> &Self::Output {
        &self.pool[(frame % Self::CAPACITY as u64) as usize]
    }
}

impl std::ops::IndexMut<u64> for ProfilerTimerPool {
    #[inline]
    fn index_mut(&mut self, frame: u64) -> &mut Self::Output {
        &mut self.pool[(frame % Self::CAPACITY as u64) as usize]
    }
}

/// Stores profiling samples in a circular buffer. Will clear the buffer when
/// samples aren't inserted consecutively.
pub struct ProfilerSampleBuffer {
    samples: [GpuCpuTimeSpan; Self::CAPACITY],
    frame: u64,
    count: usize,
}

impl ProfilerSampleBuffer {
    pub const CAPACITY: usize = 8;

    pub fn new() -> Self {
        Self {
            samples: [GpuCpuTimeSpan::default(); Self::CAPACITY],
            frame: 0,
            count: 0,
        }
    }

    pub fn update(&mut self, frame: u64, sample: GpuCpuTimeSpan) {
        self.samples[(frame % Self::CAPACITY as u64) as usize] = sample;
        if self.frame + 1 == frame {
            if self.count < Self::CAPACITY {
                self.count += 1;
            }
        } else {
            self.count = 1;
        }
        self.frame = frame;
    }

    pub fn stats(&self, frame: u64) -> Option<GpuCpuStats> {
        if self.frame == frame {
            let mut iter = self.samples.iter();
            let first = iter.next();
            first.map(move |first| {
                let dg = first.gpu.delta();
                let dc = first.cpu.delta();
                let mut stats = iter.fold(GpuCpuStats {
                    gpu_elapsed_avg: dg,
                    gpu_elapsed_min: dg,
                    gpu_elapsed_max: dg,
                    cpu_elapsed_avg: dc,
                    cpu_elapsed_min: dc,
                    cpu_elapsed_max: dc,
                }, |mut stats, item| {
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
                });

                stats.gpu_elapsed_avg /= Self::CAPACITY as u64;
                stats.cpu_elapsed_avg /= Self::CAPACITY as u64;

                stats
            })
        } else {
            None
        }
    }
}

pub struct GpuCpuStats {
    pub gpu_elapsed_avg: u64,
    pub gpu_elapsed_min: u64,
    pub gpu_elapsed_max: u64,
    pub cpu_elapsed_avg: u64,
    pub cpu_elapsed_min: u64,
    pub cpu_elapsed_max: u64,
}

pub struct Profiler {
    timers: ProfilerTimerPool,
    samples: ProfilerSampleBuffer,
}

impl Profiler {
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            timers: ProfilerTimerPool::new(gl),
            samples: ProfilerSampleBuffer::new(),
        }
    }

    pub fn start(&mut self, gl: &gl::Gl, frame: u64, epoch: Instant) {
        let timer = &mut self.timers[frame];
        let sample = timer.read(gl);
        if let Some(sample) = sample {
            self.samples.update(frame, sample);
        }
        timer.start(&gl, epoch);
    }

    pub fn stop(&mut self, gl: &gl::Gl, frame: u64, epoch: Instant) {
        let timer = &mut self.timers[frame];
        timer.stop(&gl, epoch);
    }

    pub fn stats(&self, frame: u64) -> Option<GpuCpuStats> {
        self.samples.stats(frame)
    }
}
