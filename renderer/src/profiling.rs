use crate::*;

pub struct QueryPool {
    names: [gl::QueryName; 2],
}

impl QueryPool {
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            Self {
                names: [
                    gl.create_query(gl::TIMESTAMP),
                    gl.create_query(gl::TIMESTAMP),
                    // gl.create_query(gl::TIMESTAMP),
                    // gl.create_query(gl::TIMESTAMP),
                ],
            }
        }
    }

    pub fn now(&self, gl: &gl::Gl, tick: u64) -> Option<NonZeroU64> {
        unsafe {
            let name = &self.names[tick as usize % self.names.len()];
            let result = gl.try_query_result_u64(name);
            gl.query_counter(name);
            result
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TimeSpan {
    pub begin: u64,
    pub end: u64,
}

impl TimeSpan {
    pub fn delta(&self) -> u64 {
        self.end - self.begin
    }
}

#[derive(Debug, Copy, Clone)]
pub struct GpuCpuTimeSpan {
    pub gpu: TimeSpan,
    pub cpu: TimeSpan,
}

pub struct Profiler {
    begin_query_pool: QueryPool,
    end_query_pool: QueryPool,
}

impl Profiler {
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            begin_query_pool: QueryPool::new(gl),
            end_query_pool: QueryPool::new(gl),
        }
    }

    pub fn measure(&self, gl: &gl::Gl, epoch: Instant, tick: u64, f: impl FnOnce()) -> GpuCpuTimeSpan {
        let gpu_begin = self.begin_query_pool.now(&gl, tick).map(NonZeroU64::get).unwrap_or(0);
        let cpu_begin = epoch.elapsed().as_nanos() as u64;

        f();

        let gpu_end = self.end_query_pool.now(&gl, tick).map(NonZeroU64::get).unwrap_or(0);
        let cpu_end = epoch.elapsed().as_nanos() as u64;

        GpuCpuTimeSpan {
            gpu: TimeSpan {
                begin: gpu_begin,
                end: gpu_end,
            },
            cpu: TimeSpan {
                begin: cpu_begin,
                end: cpu_end,
            },
        }
    }
}

// pub struct CpuGpuTimer {
//     names: [gl::QueryName; 2],
// }

// impl QueryTimer {
//     pub fn new(gl: &gl::Gl) -> Self {
//         unsafe {
//             Self {
//                 names: [
//                     gl.create_query(gl::TIME_ELAPSED),
//                     gl.create_query(gl::TIME_ELAPSED),
//                     // gl.create_query(gl::TIME_ELAPSED),
//                     // gl.create_query(gl::TIME_ELAPSED),
//                 ],
//             }
//         }
//     }

//     pub fn time(&self, gl: &gl::Gl, tick: u64, f: impl FnOnce()) -> Option<NonZeroU64> {
//         let result = self.begin(gl, tick);
//         f();
//         self.end(gl);
//         result
//     }

//     fn begin(&self, gl: &gl::Gl, tick: u64) -> Option<NonZeroU64> {
//         unsafe {
//             let name = self.names[usize::try_from(tick).unwrap() % self.names.len()];
//             let result = gl.try_query_result_u64(&name);
//             gl.begin_query(gl::TIME_ELAPSED, &name);
//             result
//         }
//     }

//     fn end(&self, gl: &gl::Gl) {
//         unsafe {
//             gl.end_query(gl::TIME_ELAPSED);
//         }
//     }
// }
