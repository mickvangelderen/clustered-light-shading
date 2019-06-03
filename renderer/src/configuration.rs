use crate::camera;

pub const FILE_PATH: &'static str = "configuration.toml";

#[derive(serde::Deserialize, Debug, Copy, Clone, Default)]
pub struct Root {
    pub global: Global,
    pub window: Window,
    pub clustered_light_shading: ClusteredLightShading,
    pub camera: GenericCamera,
    pub main_camera: Camera,
    pub debug_camera: Camera,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct Window {
    pub vsync: bool,
    pub rgb_bits: u8,
    pub alpha_bits: u8,
    pub srgb: bool,
}

impl Default for Window {
    fn default() -> Self {
        Window {
            vsync: true,
            rgb_bits: 24,
            alpha_bits: 8,
            srgb: true,
        }
    }
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct Global {
    pub diffuse_srgb: bool,
    pub framebuffer_srgb: bool,
}

impl Default for Global {
    fn default() -> Self {
        Global {
            diffuse_srgb: true,
            framebuffer_srgb: true,
        }
    }
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct ClusteredLightShading {
    pub cluster_side: f32,
    pub light_index: Option<u32>,
    pub min_light_count: u32,
    pub animate_z: Option<f32>,
}

impl Default for ClusteredLightShading {
    fn default() -> Self {
        ClusteredLightShading {
            cluster_side: 5.0,
            light_index: None,
            min_light_count: 1,
            animate_z: None,
        }
    }
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct GenericCamera {
    pub maximum_smoothness: f32,
}

impl Default for GenericCamera {
    fn default() -> Self {
        GenericCamera {
            maximum_smoothness: 0.8,
        }
    }
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct Camera {
    pub z0: f32,
    pub z1: f32,
    pub positional_velocity: f32,
    pub angular_velocity: f32,
    pub zoom_velocity: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            z0: -0.1,
            z1: -50.0,
            positional_velocity: 2.0,
            angular_velocity: 0.4,
            zoom_velocity: 1.0,
        }
    }
}

impl Into<camera::CameraProperties> for Camera {
    fn into(self) -> camera::CameraProperties {
        let Camera {
            z0,
            z1,
            positional_velocity,
            angular_velocity,
            zoom_velocity,
        } = self;
        camera::CameraProperties {
            z0,
            z1,
            positional_velocity,
            angular_velocity,
            zoom_velocity,
        }
    }
}
