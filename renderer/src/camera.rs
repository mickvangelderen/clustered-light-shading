use crate::clamp::*;
use cgmath::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct CameraTransform {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub fovy: Rad<f32>,
}

impl CameraTransform {
    fn interpolate(self, other: Self, t: f32) -> Self {
        let s = 1.0 - t;
        Self {
            position: EuclideanSpace::from_vec(self.position.to_vec() * s + other.position.to_vec() * t),
            yaw: self.yaw * s + other.yaw * t,
            pitch: self.pitch * s + other.pitch * t,
            fovy: self.fovy * s + other.fovy * t,
        }
    }

    #[inline]
    fn pitch_range() -> (Rad<f32>, Rad<f32>) {
        (Rad::from(Deg(-90.0)), Rad::from(Deg(90.0)))
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
        Matrix4::from_translation(self.position.to_vec()) * Matrix4::from(self.rot_to_parent())
    }

    #[inline]
    pub fn rot_from_parent(&self) -> Quaternion<f32> {
        Quaternion::from_axis_angle(Vector3::unit_x(), -self.pitch)
            * Quaternion::from_axis_angle(Vector3::unit_y(), -self.yaw)
    }

    #[inline]
    pub fn pos_from_parent(&self) -> Matrix4<f32> {
        Matrix4::from(self.rot_from_parent()) * Matrix4::from_translation(-self.position.to_vec())
    }
}

#[derive(Debug)]
pub struct CameraCorrection {
    pub delta_yaw: Rad<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct CameraProperties {
    pub z0: f32,
    pub z1: f32,
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

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Camera {
    pub properties: CameraProperties,
    pub transform: CameraTransform,
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

    #[inline]
    pub fn interpolate(a: Self, b: Self, t: f32) -> Camera {
        Camera {
            properties: b.properties,
            transform: CameraTransform::interpolate(a.transform, b.transform, t),
        }
    }
}

#[derive(Debug)]
pub struct SmoothCamera {
    pub properties: CameraProperties,
    pub current_transform: CameraTransform,
    pub target_transform: CameraTransform,
    pub smooth_enabled: bool,
    pub current_smoothness: f32,
    pub maximum_smoothness: f32,
}

impl SmoothCamera {
    #[inline]
    pub fn update(&mut self, delta: &CameraDelta) {
        self.target_transform.update(&CameraDelta {
            time: delta.time,
            position: delta.position * self.properties.positional_velocity,
            yaw: delta.yaw * self.properties.angular_velocity,
            pitch: delta.pitch * self.properties.angular_velocity,
            fovy: delta.fovy * self.properties.zoom_velocity,
        });

        let correction = self.target_transform.correction();
        self.target_transform.correct(&correction);
        self.current_transform.correct(&correction);

        self.current_smoothness = self.target_smoothness() * 0.2 + self.current_smoothness * 0.8;

        self.current_transform = CameraTransform::interpolate(
            self.current_transform,
            self.target_transform,
            1.0 - self.current_smoothness,
        );
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

    #[inline]
    pub fn current_to_camera(&self) -> Camera {
        Camera {
            properties: self.properties,
            transform: self.current_transform,
        }
    }
}

#[derive(Debug)]
pub struct TransitionCamera {
    pub start_camera: Camera,
    pub current_camera: Camera,
    pub progress: f32,
}

pub struct TransitionCameraUpdate<'a> {
    pub delta_time: f32,
    pub end_camera: &'a Camera,
}

impl TransitionCamera {
    #[inline]
    pub fn start_transition(&mut self) {
        self.start_camera = self.current_camera;
        self.progress = 0.0;
    }

    #[inline]
    pub fn update(&mut self, update: TransitionCameraUpdate) {
        self.progress += update.delta_time * 4.0;
        if self.progress > 1.0 {
            self.progress = 1.0;
        }

        // Bring current yaw within (-half turn, half turn) of
        // the target yaw without changing the actual angle.
        let start_yaw = self.start_camera.transform.yaw;
        let end_yaw = update.end_camera.transform.yaw;
        self.start_camera.transform.yaw = end_yaw
            + Rad((start_yaw - end_yaw + Rad::turn_div_2())
                .0
                .rem_euclid(Rad::full_turn().0))
            - Rad::turn_div_2();

        let x = self.progress;
        let t = x * x * (3.0 - 2.0 * x);

        self.current_camera = Camera::interpolate(self.start_camera, *update.end_camera, t);
    }
}
