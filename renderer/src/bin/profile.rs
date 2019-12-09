use std::process::Command;
use renderer::*;

pub fn run_with_configuration(configuration_path: &str) {
    let _ = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "renderer",
            "--",
            "--configuration-path",
            configuration_path,
        ])
        .status()
        .expect("failed to execute process");

    let _ = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "profiling_reader",
        ])
        .status()
        .expect("failed to execute process");
}

pub fn main() {
    let sun_temple_cfg = Configuration::read("resources/profile_sun_temple_base.toml");

    let light_counts: &[u32; 2] = &[1000, 10000];

    for &light_count in light_counts {
        let name = format!("sun_temple_{:07}", light_count);
        let mut cfg = sun_temple_cfg.clone();
        cfg.profiling.name = Some(std::path::PathBuf::from(name.clone()));
        cfg.rain.max_count = light_count;
        let profiling_dir = std::path::PathBuf::from("profiling").join(name);
        let configuration_path = profiling_dir.join("configuration.toml");
        cfg.write(&configuration_path).unwrap();
        run_with_configuration(configuration_path.to_str().unwrap());
    }
}
