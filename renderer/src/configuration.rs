use crate::camera;
use std::path::PathBuf;

pub const FILE_PATH: &'static str = "configuration.toml";

#[derive(serde::Deserialize, Debug, Clone)]
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

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Window {
    pub vsync: bool,
    pub rgb_bits: u8,
    pub alpha_bits: u8,
    pub srgb: bool,
    pub width: u32,
    pub height: u32,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Global {
    pub diffuse_srgb: bool,
    pub framebuffer_srgb: bool,
    pub rain_drop_max: u32,
    pub record: Option<PathBuf>,
    pub replay: Option<PathBuf>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct VirtualStereo {
    pub enabled: bool,
    pub pitch_deg: f32,
    pub yaw_deg: f32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClusteringProjection {
    Orthographic,
    Perspective,
}

#[derive(serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClusteringGrouping {
    Individual,
    Enclosed,
}

#[derive(serde::Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vector3<T> {
    pub fn to_array(self) -> [T; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(serde::Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct ClusteredLightShading {
    pub projection: ClusteringProjection,
    pub grouping: ClusteringGrouping,
    pub perspective_sides: Vector3<f64>,
    pub orthographic_sides: Vector3<f64>,
    pub max_clusters: u32,
    pub max_active_clusters: u32,
    pub max_light_indices: u32,
}

impl ClusteredLightShading {
    pub fn cluster_sides(&self) -> cgmath::Vector3<f64> {
        let sides: [f64; 3] = match self.projection {
            ClusteringProjection::Perspective => self.perspective_sides.to_array(),
            ClusteringProjection::Orthographic => self.orthographic_sides.to_array(),
        };
        sides.into()
    }
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
