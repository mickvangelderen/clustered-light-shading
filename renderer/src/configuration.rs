use crate::camera;
use crate::profiling::ProfilingConfiguration;
use std::path::PathBuf;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Configuration {
    pub global: GlobalConfiguration,
    pub window: crate::WindowConfiguration,
    pub gl: crate::GlConfiguration,
    pub clustered_light_shading: ClusteredLightShadingConfiguration,
    pub virtual_stereo: VirtualStereoConfiguration,
    pub camera: GenericCameraConfiguration,
    pub main_camera: CameraConfiguration,
    pub debug_camera: CameraConfiguration,
    pub prefix_sum: PrefixSumConfiguration,
    pub profiling: ProfilingConfiguration,
    pub record: RecordConfiguration,
    pub replay: ReplayConfiguration,
}

impl Configuration {
    pub const DEFAULT_PATH: &'static str = "configuration.toml";

    pub fn read(configuration_path: &std::path::Path) -> Self {
        match std::fs::read_to_string(&configuration_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(configuration) => configuration,
                Err(err) => panic!("Failed to parse configuration file {:?}: {}.", configuration_path, err),
            },
            Err(err) => panic!("Failed to read configuration file {:?}: {}.", configuration_path, err),
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct RecordConfiguration {
    pub path: PathBuf,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ReplayConfiguration {
    pub run_count: usize,
    pub path: PathBuf,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct GlobalConfiguration {
    pub diffuse_srgb: bool,
    pub rain_drop_max: u32,
    pub mode: ApplicationMode,
    pub scene_path: PathBuf,
    pub sample_count: u32,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct VirtualStereoConfiguration {
    pub enabled: bool,
    pub pitch_deg: f32,
    pub yaw_deg: f32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ApplicationMode {
    Normal,
    Record,
    Replay,
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
pub struct ClusteredLightShadingConfiguration {
    pub projection: ClusteringProjection,
    pub grouping: ClusteringGrouping,
    pub perspective_sides: Vector3<f64>,
    pub orthographic_sides: Vector3<f64>,
    pub max_clusters: u32,
    pub max_active_clusters: u32,
    pub max_light_indices: u32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct PrefixSumConfiguration {
    pub pass_0_threads: u32,
    pub pass_1_threads: u32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct GenericCameraConfiguration {
    pub maximum_smoothness: f32,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub struct CameraConfiguration {
    pub z0: f32,
    pub z1: f32,
    pub positional_velocity: f32,
    pub angular_velocity: f32,
    pub zoom_velocity: f32,
}

impl Into<camera::CameraProperties> for CameraConfiguration {
    fn into(self) -> camera::CameraProperties {
        let CameraConfiguration {
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
