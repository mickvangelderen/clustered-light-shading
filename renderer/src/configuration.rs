use crate::camera;

pub const FILE_PATH: &'static str = "configuration.toml";

#[derive(serde::Deserialize, Debug, Copy, Clone, Default)]
pub struct Root {
    pub clustered_light_shading: ClusteredLightShading,
    pub camera: GenericCamera,
    pub main_camera: Camera,
    pub debug_camera: Camera,
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
    pub positional_velocity: f32,
    pub angular_velocity: f32,
    pub zoom_velocity: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            positional_velocity: 2.0,
            angular_velocity: 0.4,
            zoom_velocity: 1.0,
        }
    }
}

impl Into<camera::CameraProperties> for Camera {
    fn into(self) -> camera::CameraProperties {
        let Camera { positional_velocity, angular_velocity, zoom_velocity } = self;
        camera::CameraProperties {
            positional_velocity,
            angular_velocity,
            zoom_velocity,
        }
    }
}
