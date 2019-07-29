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
        self.end - self.begin
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GpuCpuTimeSpan {
    pub gpu: TimeSpan,
    pub cpu: TimeSpan,
}

#[derive(Debug)]
pub struct Profiler {
    begin_query_name: gl::QueryName,
    end_query_name: gl::QueryName,
    cpu_begin: Option<NonZeroU64>,
    cpu_end: Option<NonZeroU64>,
}

impl Profiler {
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

pub struct ProfilerPool {
    pool: [Profiler; Self::CAPACITY],
}

impl ProfilerPool {
    pub const CAPACITY: usize = 3;

    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            pool: [
                Profiler::new(gl),
                Profiler::new(gl),
                Profiler::new(gl),
            ]
        }
    }
}

impl std::ops::Index<u64> for ProfilerPool {
    type Output = Profiler;

    #[inline]
    fn index(&self, tick: u64) -> &Self::Output {
        &self.pool[(tick % Self::CAPACITY as u64) as usize]
    }
}

impl std::ops::IndexMut<u64> for ProfilerPool {
    #[inline]
    fn index_mut(&mut self, tick: u64) -> &mut Self::Output {
        &mut self.pool[(tick % Self::CAPACITY as u64) as usize]
    }
}
