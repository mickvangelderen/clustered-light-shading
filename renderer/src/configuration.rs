use crate::camera;

pub const FILE_PATH: &'static str = "configuration.toml";

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct Root {
    pub global: Global,
    pub window: Window,
    pub clustered_light_shading: ClusteredLightShading,
    pub virtual_stereo: VirtualStereo,
    pub camera: GenericCamera,
    pub main_camera: Camera,
    pub debug_camera: Camera,
    pub prefix_sum: PrefixSum,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct Window {
    pub vsync: bool,
    pub rgb_bits: u8,
    pub alpha_bits: u8,
    pub srgb: bool,
    pub width: u32,
    pub height: u32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct Global {
    pub diffuse_srgb: bool,
    pub framebuffer_srgb: bool,
    pub rain_drop_max: u32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct VirtualStereo {
    pub enabled: bool,
    pub pitch_deg: f32,
    pub yaw_deg: f32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct ClusteredLightShading {
    pub cluster_side: f32,
    pub light_index: Option<u32>,
    pub min_light_count: u32,
    pub animate_z: Option<f32>,
    pub animate_light_count: Option<f32>,
    pub max_cluster_count: u32,
    pub max_active_cluster_count: u32,
    pub max_light_index_count: u32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct PrefixSum {
    pub pass_0_threads: u32,
    pub pass_1_threads: u32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct GenericCamera {
    pub maximum_smoothness: f32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct Camera {
    pub z0: f32,
    pub z1: f32,
    pub positional_velocity: f32,
    pub angular_velocity: f32,
    pub zoom_velocity: f32,
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

pub fn read(configuration_path: &std::path::Path) -> Root {
    match std::fs::read_to_string(&configuration_path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(configuration) => configuration,
            Err(err) => panic!("Failed to parse configuration file {:?}: {}.", configuration_path, err),
        },
        Err(err) => panic!("Failed to read configuration file {:?}: {}.", configuration_path, err),
    }
}
