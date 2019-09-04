use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

use renderer::configuration;
use renderer::profiling::*;

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let resource_dir: PathBuf = [current_dir.as_ref(), Path::new("resources")].into_iter().collect();
    let configuration_path = resource_dir.join(configuration::FILE_PATH);
    let configuration = configuration::read(&configuration_path);

    let mut file = BufReader::new(File::open(configuration.global.profiling_path.as_ref().unwrap()).unwrap());
    while let Ok(entry) = bincode::deserialize_from::<_, FileEntry>(&mut file) {
        println!("{:?}", entry);
    }
}
