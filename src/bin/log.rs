use log;
use std::fs;
use std::io::{self, prelude::*};

fn main() -> io::Result<()> {
    let mut file = io::BufReader::new(fs::File::open("log.bin")?);

    let mut buf = [0u8; std::mem::size_of::<log::Entry>()];

    if let Err(error) = file.read_exact(&mut buf) {
        match error.kind() {
            io::ErrorKind::UnexpectedEof => panic!("Log requires at least 1 entry."),
            _ => return Err(error),
        }
    }

    let mut frame: u64 = 1;
    let mut entry = log::Entry::from_ne_bytes(buf);
    let first_simulation_start_nanos: u64 = entry.simulation_start_nanos;
    let mut sim: u64 = 0;
    let mut pos: u64 = 0;
    let mut ren: u64 = 0;

    loop {
        let mut buf = [0u8; std::mem::size_of::<log::Entry>()];

        if let Err(error) = file.read_exact(&mut buf) {
            match error.kind() {
                io::ErrorKind::UnexpectedEof => break,
                _ => return Err(error),
            }
        }

        frame += 1;
        entry = log::Entry::from_ne_bytes(buf);
        sim = sim.checked_add(entry.simulation_duration_nanos()).unwrap();
        pos = pos.checked_add(entry.pose_duration_nanos()).unwrap();
        ren = ren.checked_add(entry.render_duration_nanos()).unwrap();
    }

    println!(
        "frame count: {}, fps: {}",
        frame,
        1_000_000_000.0
            / ((entry.simulation_start_nanos - first_simulation_start_nanos) / frame) as f64
    );

    fn ms(a: u64, b: u64) -> f64 {
        ((a / b) as f64 + (a % b) as f64) / 1_000_000.0
    }

    println!("simulation avg: {} ms", ms(sim, frame));
    println!("pose avg: {} ms", ms(pos, frame));
    println!("render avg: {} ms", ms(ren, frame));

    Ok(())
}
