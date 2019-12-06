use std::process::Command;

pub fn run_with_configuration(configuration_path: &str) {
    let _output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "renderer",
            "--",
            "--configuration-path",
            configuration_path,
        ])
        .output()
        .expect("failed to execute process");

    let _output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "profiling_reader",
        ])
        .output()
        .expect("failed to execute process");
}

pub fn main() {
    run_with_configuration("resources/sun_temple_ortho_0200.toml");
    run_with_configuration("resources/sun_temple_ortho_0400.toml");
}
