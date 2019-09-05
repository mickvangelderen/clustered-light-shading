use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use renderer::configuration;
use renderer::profiling::*;

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let resource_dir: PathBuf = [current_dir.as_ref(), Path::new("resources")].into_iter().collect();
    let configuration_path = resource_dir.join(configuration::FILE_PATH);
    let configuration = configuration::read(&configuration_path);

    let mut file = BufReader::new(File::open(configuration.profiling.path.as_ref().unwrap()).unwrap());

    let (frames, sample_names) = {
        let mut frames: Vec<Vec<Option<GpuCpuTimeSpan>>> = Vec::new();
        let mut samples = None;
        while let Ok(entry) = bincode::deserialize_from::<_, FileEntry>(&mut file) {
            match entry {
                FileEntry::Frame(frame) => frames.push(frame),
                FileEntry::Samples(s) => {
                    assert!(samples.replace(s).is_none())
                }
            }
        }
        (frames, samples.unwrap())
    };

    for (frame_index, samples) in frames.iter().enumerate() {
        println!("Frame {}", frame_index);
        for (sample_index, sample) in samples.iter().enumerate() {
            println!("({}) {}: {:?}", sample_index, sample_names[sample_index], sample);
        }
    }
}
