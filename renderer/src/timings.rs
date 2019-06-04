use std::time;

#[derive(Debug, Copy, Clone, Default)]
pub struct Span<T> {
    pub start: T,
    pub end: T,
}

impl<T> Span<T>
where
    T: Copy + std::ops::Sub,
{
    pub fn delta(&self) -> <T as std::ops::Sub>::Output {
        self.end - self.start
    }
}

#[derive(Debug)]
pub struct Timings {
    pub accumulate_file_updates: Span<time::Instant>,
    pub execute_file_updates: Span<time::Instant>,
    pub wait_for_pose: Span<time::Instant>,
    pub accumulate_window_updates: Span<time::Instant>,
    pub accumulate_vr_updates: Span<time::Instant>,
    pub simulate: Span<time::Instant>,
    pub prepare_render_data: Span<time::Instant>,
    pub render: Span<time::Instant>,
    pub swap_buffers: Span<time::Instant>,
}

impl Timings {
    pub fn print_deltas(&self) {
        println!(
            "accumulate_file_updates   {:4>}μs",
            self.accumulate_file_updates.delta().as_micros()
        );
        println!(
            "execute_file_updates      {:4>}μs",
            self.execute_file_updates.delta().as_micros()
        );
        println!(
            "wait_for_pose             {:4>}μs",
            self.wait_for_pose.delta().as_micros()
        );
        println!(
            "accumulate_window_updates {:4>}μs",
            self.accumulate_window_updates.delta().as_micros()
        );
        println!(
            "accumulate_vr_updates     {:4>}μs",
            self.accumulate_vr_updates.delta().as_micros()
        );
        println!("simulate                  {:4>}μs", self.simulate.delta().as_micros());
        println!(
            "prepare_render_data       {:4>}μs",
            self.prepare_render_data.delta().as_micros()
        );
        println!("render                    {:4>}μs", self.render.delta().as_micros());
        println!(
            "swap_buffers              {:4>}μs",
            self.swap_buffers.delta().as_micros()
        );
    }
}
