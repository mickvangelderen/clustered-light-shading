pub const FILE_PATH: &'static str = "configuration.toml";

#[derive(serde::Deserialize, Debug, Copy, Clone, Default)]
pub struct Root {
    pub clustered_light_shading: ClusteredLightShading,
    pub main_camera: MainCamera,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct ClusteredLightShading {
    pub cluster_side: f32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct MainCamera {
    pub maximum_smoothness: f32,
    pub positional_velocity: f32,
    pub angular_velocity: f32,
    pub zoom_velocity: f32,
}

impl Default for ClusteredLightShading {
    fn default() -> Self {
        ClusteredLightShading { cluster_side: 5.0 }
    }
}

impl Default for MainCamera {
    fn default() -> Self {
        MainCamera {
            maximum_smoothness: 0.7,
            positional_velocity: 2.0,
            angular_velocity: 0.4,
            zoom_velocity: 1.0,
        }
    }
}
