use log;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let mut entries: Vec<u8> = fs::read("log/log.bin")?;

    if entries.len() % std::mem::size_of::<log::Entry>() != 0 {
        panic!("Unexpected number of bytes read.");
    }

    let entries: Vec<log::Entry> = unsafe {
        let out = Vec::from_raw_parts(
            entries.as_mut_ptr() as *mut log::Entry,
            entries.len() / std::mem::size_of::<log::Entry>(),
            entries.capacity(),
        );
        std::mem::forget(entries);
        out
    };

    let frame = entries.len() as u64;
    let sim = entries
        .iter()
        .map(log::Entry::simulation_duration_nanos)
        .sum();
    let pos = entries.iter().map(log::Entry::pose_duration_nanos).sum();
    let ren = entries.iter().map(log::Entry::render_duration_nanos).sum();

    let total = entries.last().unwrap().simulation_start_nanos
        - entries.first().unwrap().simulation_start_nanos;
    println!(
        "frame count: {}, fps: {}",
        frame,
        1_000_000_000.0 / total as f64
    );

    fn ms(a: u64, b: u64) -> f64 {
        ((a / b) as f64 + (a % b) as f64) / 1_000_000.0
    }

    println!("simulation avg: {} ms", ms(sim, frame));
    println!("pose avg: {} ms", ms(pos, frame));
    println!("render avg: {} ms", ms(ren, frame));

    Ok(())
}
