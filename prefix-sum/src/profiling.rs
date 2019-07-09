use crate::*;

use std::time::Instant;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ns(pub u64);

impl std::fmt::Debug for Ns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: Take width/fill char character?
        write!(f, "{:6}.{:03}μs", self.0 / 1000, self.0 % 1000)
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
    begin_query: gl::QueryName,
    end_query: gl::QueryName,
    cpu_begin: Option<u64>,
    cpu_end: Option<u64>,
}

pub struct ProfilerRecording<'a> {
    profiler: &'a mut Profiler,
    gl: &'a gl::Gl,
    epoch: &'a Instant,
}

impl<'a> Drop for ProfilerRecording<'a> {
    fn drop(&mut self) {
        self.profiler.record_end(self.gl, self.epoch);
    }
}

impl Profiler {
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            Self {
                begin_query: gl.create_query(gl::TIMESTAMP),
                end_query: gl.create_query(gl::TIMESTAMP),
                cpu_begin: None,
                cpu_end: None,
            }
        }
    }

    fn record_begin(&mut self, gl: &gl::Gl, epoch: &Instant) {
        unsafe {
            gl.query_counter(&self.begin_query);
        }
        assert!(self.cpu_begin.replace(epoch.elapsed().as_nanos() as u64).is_none());
    }

    fn record_end(&mut self, gl: &gl::Gl, epoch: &Instant) {
        unsafe {
            gl.query_counter(&self.end_query);
        }
        assert!(self.cpu_end.replace(epoch.elapsed().as_nanos() as u64).is_none());
    }

    pub fn record_closure(&mut self, gl: &gl::Gl, epoch: &Instant, f: impl FnOnce()) {
        self.record_begin(gl, epoch);
        f();
        self.record_end(gl, epoch);
    }

    pub fn record<'a>(&'a mut self, gl: &'a gl::Gl, epoch: &'a Instant) -> ProfilerRecording {
        self.record_begin(gl, epoch);
        ProfilerRecording {
            profiler: self,
            gl,
            epoch,
        }
    }

    pub fn query(&mut self, gl: &gl::Gl) -> GpuCpuTimeSpan {
        let gpu_begin = unsafe { gl.query_result_u64(&self.begin_query) };
        let gpu_end = unsafe { gl.query_result_u64(&self.end_query) };

        GpuCpuTimeSpan {
            gpu: TimeSpan {
                begin: gpu_begin,
                end: gpu_end,
            },
            cpu: TimeSpan {
                begin: self.cpu_begin.take().unwrap(),
                end: self.cpu_end.take().unwrap(),
            },
        }
    }

    pub fn try_query(&mut self, gl: &gl::Gl) -> Option<GpuCpuTimeSpan> {
        let gpu_begin = unsafe { gl.try_query_result_u64(&self.begin_query) };
        let gpu_end = unsafe { gl.try_query_result_u64(&self.end_query) };

        let cpu_begin = self.cpu_begin.take();
        let cpu_end = self.cpu_end.take();

        if let (Some(gpu_begin), Some(gpu_end), Some(cpu_begin), Some(cpu_end)) =
            (gpu_begin, gpu_end, cpu_begin, cpu_end)
        {
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
        } else {
            None
        }
    }
}
