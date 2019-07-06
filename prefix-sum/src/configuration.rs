#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct Root {
    pub local_x: u32,
    pub local_y: u32,
    pub local_z: u32,
    pub item_count: usize,
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
