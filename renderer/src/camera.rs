use crate::clamp::*;
use cgmath::*;

#[derive(Debug, Copy, Clone)]
pub struct CameraState {
    pub position: Vector3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub fovy: Rad<f32>,
}

#[derive(Debug)]
pub struct CameraStateCorrection {
    pub delta_yaw: Rad<f32>,
}

#[derive(Debug, Copy, Clone)]
pub struct CameraProperties {
    pub positional_velocity: f32,
    pub angular_velocity: f32,
    pub zoom_velocity: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct CameraUpdate {
    pub delta_time: f32,
    pub delta_position: Vector3<f32>,
    pub delta_yaw: Rad<f32>,
    pub delta_pitch: Rad<f32>,
    pub delta_fovy: Rad<f32>,
}

#[derive(Debug)]
pub struct Camera {
    pub properties: CameraProperties,
    pub state: CameraState,
}

#[derive(Debug)]
pub struct SmoothCamera {
    pub properties: CameraProperties,
    pub current_smoothness: f32,
    pub target_smoothness: f32,
    pub current_state: CameraState,
    pub target_state: CameraState,
}

impl CameraState {
    #[inline]
    fn pitch_range() -> (Rad<f32>, Rad<f32>) {
        (Rad::from(Deg(-89.0)), Rad::from(Deg(89.0)))
    }

    #[inline]
    fn fovy_range() -> (Rad<f32>, Rad<f32>) {
        (Rad::from(Deg(10.0)), Rad::from(Deg(120.0)))
    }

    #[inline]
    fn update_without_correction(&mut self, properties: &CameraProperties, update: &CameraUpdate) {
        let CameraProperties {
            positional_velocity,
            angular_velocity,
            zoom_velocity,
        } = *properties;
        let CameraUpdate {
            delta_time,
            delta_position,
            delta_yaw,
            delta_pitch,
            delta_fovy,
        } = *update;
        // Direct delta_position along yaw angle.
        let delta_position =
            Quaternion::from_axis_angle(Vector3::unit_y(), self.yaw) * delta_position;

        *self = CameraState {
            position: self.position + delta_position * positional_velocity * delta_time,
            yaw: (self.yaw + delta_yaw * angular_velocity * delta_time),
            pitch: (self.pitch + delta_pitch * angular_velocity * delta_time)
                .clamp_range(Self::pitch_range()),
            fovy: (self.fovy + delta_fovy * zoom_velocity * delta_time)
                .clamp_range(Self::fovy_range()),
        };
    }

    #[inline]
    fn compute_correction(&self) -> CameraStateCorrection {
        let new_yaw = self.yaw % Rad::full_turn();
        CameraStateCorrection {
            delta_yaw: new_yaw - self.yaw,
        }
    }

    #[inline]
    fn correct(&mut self, correction: &CameraStateCorrection) {
        self.yaw += correction.delta_yaw;
    }

    #[inline]
    fn interpolate(&mut self, b: &CameraState, t: f32) {
        let u = 1.0 - t;

        self.position = self.position * t + b.position * u;
        self.yaw = self.yaw * t + b.yaw * u;
        self.pitch = (self.pitch * t + b.pitch * u).clamp_range(Self::pitch_range());
        self.fovy = (self.fovy * t + b.fovy * u).clamp_range(Self::fovy_range());
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
    pub fn update(&mut self, update: &CameraUpdate) {
        self.state.update_without_correction(&self.properties, update);
        self.state.correct(&self.state.compute_correction());
    }

    #[inline]
    pub fn rot_to_parent(&self) -> Quaternion<f32> {
        self.state.rot_to_parent()
    }

    #[inline]
    pub fn pos_to_parent(&self) -> Matrix4<f32> {
        self.state.pos_to_parent()
    }

    #[inline]
    pub fn rot_from_parent(&self) -> Quaternion<f32> {
        self.state.rot_from_parent()
    }

    #[inline]
    pub fn pos_from_parent(&self) -> Matrix4<f32> {
        self.state.pos_from_parent()
    }
}

impl SmoothCamera {
    #[inline]
    pub fn new(smoothness: f32, camera: Camera) -> Self {
        let Camera { properties, state } = camera;
        SmoothCamera {
            properties,
            current_smoothness: smoothness,
            target_smoothness: smoothness,
            current_state: state,
            target_state: state,
        }
    }

    #[inline]
    pub fn update(&mut self, update: &CameraUpdate) {
        self.current_smoothness = self.target_smoothness * 0.8 + self.current_smoothness * 0.2;
        self.target_state
            .update_without_correction(&self.properties, update);
        let correction = self.target_state.compute_correction();
        self.target_state.correct(&correction);
        self.current_state
            .interpolate(&self.target_state, self.current_smoothness);
        self.current_state.correct(&correction);
    }

    #[inline]
    pub fn rot_to_parent(&self) -> Quaternion<f32> {
        self.current_state.rot_to_parent()
    }

    #[inline]
    pub fn pos_to_parent(&self) -> Matrix4<f32> {
        self.current_state.pos_to_parent()
    }

    #[inline]
    pub fn rot_from_parent(&self) -> Quaternion<f32> {
        self.current_state.rot_from_parent()
    }

    #[inline]
    pub fn pos_from_parent(&self) -> Matrix4<f32> {
        self.current_state.pos_from_parent()
    }
}
