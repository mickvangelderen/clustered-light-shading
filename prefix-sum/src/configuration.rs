#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct Root {
    pub input: Input,
    pub prefix_sum: PrefixSum,
    pub iterations: u32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct PrefixSum {
    pub pass_0_threads: u32,
    pub pass_1_threads: u32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct Input {
    pub count: u32,
    pub min: u32,
    pub max: u32,
}

pub fn read(configuration_path: impl AsRef<std::path::Path>) -> Root {
    let configuration_path = configuration_path.as_ref();
    match std::fs::read_to_string(configuration_path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(configuration) => configuration,
            Err(err) => {
                panic!("Failed to parse configuration file {:?}: {}.", configuration_path, err)
            }
        },
        Err(err) => {
            panic!("Failed to read configuration file {:?}: {}.", configuration_path, err)
        }
    }
}
