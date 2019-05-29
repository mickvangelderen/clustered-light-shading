use crate::clamp::*;
use cgmath::*;

#[derive(Debug, Copy, Clone)]
pub struct CameraTransform {
    pub position: Vector3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub fovy: Rad<f32>,
}

#[derive(Debug)]
pub struct CameraCorrection {
    pub delta_yaw: Rad<f32>,
}

#[derive(Debug, Copy, Clone)]
pub struct CameraProperties {
    pub positional_velocity: f32,
    pub angular_velocity: f32,
    pub zoom_velocity: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct CameraDelta {
    pub time: f32,
    pub position: Vector3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub fovy: Rad<f32>,
}

#[derive(Debug)]
pub struct Camera {
    pub transform: CameraTransform,
    pub properties: CameraProperties,
}

#[derive(Debug)]
pub struct SmoothCamera {
    pub transform: CameraTransform,
    pub smooth_enabled: bool,
    pub current_smoothness: f32,
    pub maximum_smoothness: f32,
}

impl CameraTransform {
    #[inline]
    fn pitch_range() -> (Rad<f32>, Rad<f32>) {
        (Rad::from(Deg(-89.0)), Rad::from(Deg(89.0)))
    }

    #[inline]
    fn fovy_range() -> (Rad<f32>, Rad<f32>) {
        (Rad::from(Deg(10.0)), Rad::from(Deg(120.0)))
    }

    #[inline]
    pub fn update(&mut self, delta: &CameraDelta) {
        *self = CameraTransform {
            // Direct delta_position along yaw angle.
            position: self.position
                + Quaternion::from_axis_angle(Vector3::unit_y(), self.yaw) * delta.position * delta.time,
            yaw: (self.yaw + delta.yaw * delta.time),
            pitch: (self.pitch + delta.pitch * delta.time).clamp_range(Self::pitch_range()),
            fovy: (self.fovy + delta.fovy * delta.time).clamp_range(Self::fovy_range()),
        };
    }

    #[inline]
    pub fn interpolate(&mut self, b: &CameraTransform, t: f32) {
        let s = 1.0 - t;

        self.position = self.position * s + b.position * t;
        self.yaw = self.yaw * s + b.yaw * t;
        self.pitch = self.pitch * s + b.pitch * t;
        self.fovy = self.fovy * s + b.fovy * t;
    }

    #[inline]
    pub fn correction(&self) -> CameraCorrection {
        CameraCorrection {
            delta_yaw: (self.yaw % Rad::full_turn()) - self.yaw,
        }
    }

    #[inline]
    pub fn correct(&mut self, correction: &CameraCorrection) {
        self.yaw += correction.delta_yaw;
    }

    #[inline]
    pub fn rot_to_parent(&self) -> Quaternion<f32> {
        Quaternion::from_axis_angle(Vector3::unit_y(), self.yaw)
            * Quaternion::from_axis_angle(Vector3::unit_x(), self.pitch)
    }

    #[inline]
    pub fn pos_to_parent(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position) * Matrix4::from(self.rot_to_parent())
    }

    #[inline]
    pub fn rot_from_parent(&self) -> Quaternion<f32> {
        Quaternion::from_axis_angle(Vector3::unit_x(), -self.pitch)
            * Quaternion::from_axis_angle(Vector3::unit_y(), -self.yaw)
    }

    #[inline]
    pub fn pos_from_parent(&self) -> Matrix4<f32> {
        Matrix4::from(self.rot_from_parent()) * Matrix4::from_translation(-self.position)
    }
}

impl Camera {
    #[inline]
    pub fn update(&mut self, delta: &CameraDelta) {
        self.transform.update(&CameraDelta {
            time: delta.time,
            position: delta.position * self.properties.positional_velocity,
            yaw: delta.yaw * self.properties.angular_velocity,
            pitch: delta.pitch * self.properties.angular_velocity,
            fovy: delta.fovy * self.properties.zoom_velocity,
        })
    }
}

impl SmoothCamera {
    #[inline]
    pub fn update(&mut self, target: &Camera) {
        self.current_smoothness = self.target_smoothness() * 0.2 + self.current_smoothness * 0.8;
        self.transform
            .interpolate(&target.transform, 1.0 - self.current_smoothness);
    }

    #[inline]
    pub fn target_smoothness(&self) -> f32 {
        if self.smooth_enabled {
            self.maximum_smoothness
        } else {
            0.0
        }
    }

    #[inline]
    pub fn toggle_smoothness(&mut self) {
        self.smooth_enabled = !self.smooth_enabled;
    }
}
