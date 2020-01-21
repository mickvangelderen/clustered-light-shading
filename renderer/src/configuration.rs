use crate::camera;
use crate::profiling::ProfilingConfiguration;
use cgmath::*;
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Attenuation {
    pub i: f64,
    pub i0: f64,
    pub r0: f64,
}

impl Attenuation {
    pub fn r1(&self) -> f64 {
        (self.i / self.i0).sqrt()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Configuration {
    pub global: GlobalConfiguration,
    pub light: LightConfiguration,
    pub rain: RainConfiguration,
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
    pub const DEFAULT_PATH: &'static str = "resources/configuration.toml";

    pub fn read(configuration_path: impl AsRef<std::path::Path>) -> Self {
        let configuration_path = configuration_path.as_ref();
        match std::fs::read_to_string(configuration_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(configuration) => configuration,
                Err(err) => panic!("Failed to parse configuration file {:?}: {}.", configuration_path, err),
            },
            Err(err) => panic!("Failed to read configuration file {:?}: {}.", configuration_path, err),
        }
    }

    pub fn write(&self, configuration_path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let contents = toml::Value::try_from(self).unwrap().to_string();
        std::fs::create_dir_all(configuration_path.as_ref().parent().unwrap())?;
        std::fs::write(configuration_path, contents)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RecordConfiguration {
    pub path: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ReplayConfiguration {
    pub run_count: usize,
    pub path: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GlobalConfiguration {
    pub diffuse_srgb: bool,
    pub mode: ApplicationMode,
    pub scene_path: PathBuf,
    pub sample_count: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct LightConfiguration {
    pub display: bool,
    pub attenuation: Attenuation,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RainConfiguration {
    pub max_count: u32,
    pub bounds_min: Vector3<f64>,
    pub bounds_max: Vector3<f64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct VirtualStereoConfiguration {
    pub enabled: bool,
    pub l_mat: [[f64; 4]; 4],
    pub l_tan: [f64; 4],
    pub r_mat: [[f64; 4]; 4],
    pub r_tan: [f64; 4],
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ApplicationMode {
    Normal,
    Record,
    Replay,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClusteringProjection {
    Orthographic,
    Perspective,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClusteringGrouping {
    Individual,
    Enclosed,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum FragmentCountingStrategy {
    Depth,
    Geometry,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct ClusteredLightShadingConfiguration {
    pub fragment_counting_strategy: FragmentCountingStrategy,
    pub projection: ClusteringProjection,
    pub grouping: ClusteringGrouping,
    pub perspective_pixels: Vector2<u32>,
    pub orthographic_sides: Vector3<f64>,
    pub max_clusters: u32,
    pub max_active_clusters: u32,
    pub max_light_indices: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct PrefixSumConfiguration {
    pub pass_0_threads: u32,
    pub pass_1_threads: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone)]
pub struct GenericCameraConfiguration {
    pub maximum_smoothness: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone)]
pub struct CameraConfiguration {
    pub near: f32,
    pub far: f32,
    pub positional_velocity: f32,
    pub angular_velocity: f32,
    pub zoom_velocity: f32,
}

impl Into<camera::CameraProperties> for CameraConfiguration {
    fn into(self) -> camera::CameraProperties {
        let CameraConfiguration {
            near,
            far,
            positional_velocity,
            angular_velocity,
            zoom_velocity,
        } = self;
        camera::CameraProperties {
            z0: -far,
            z1: -near,
            positional_velocity,
            angular_velocity,
            zoom_velocity,
        }
    }
}
