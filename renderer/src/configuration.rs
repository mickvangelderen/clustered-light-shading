pub const FILE_PATH: &'static str = "configuration.toml";

#[derive(serde::Deserialize, Debug, Copy, Clone, Default)]
pub struct Root {
    pub clustered_light_shading: ClusteredLightShading,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct ClusteredLightShading {
    pub cluster_side: f32,
}

impl Default for ClusteredLightShading {
    fn default() -> Self {
        ClusteredLightShading { cluster_side: 5.0 }
    }
}
