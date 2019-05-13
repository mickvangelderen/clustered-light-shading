use cgmath::*;

#[derive(Debug)]
pub struct CameraUpdate {
    pub delta_time: f32,
    pub delta_position: Vector3<f32>,
    pub delta_yaw: Rad<f32>,
    pub delta_pitch: Rad<f32>,
    pub delta_scroll: f32,
}

#[derive(Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub fovy: Rad<f32>,
    pub positional_velocity: f32,
    pub angular_velocity: f32,
    pub zoom_velocity: f32,
}

trait Clamp: Sized {
    fn clamp(&self, range: (Self, Self)) -> Self;
}

impl<T> Clamp for T
where
    T: PartialOrd,
    T: Copy,
{
    fn clamp(&self, range: (Self, Self)) -> Self {
        let (min, max) = range;
        if *self > max {
            max
        } else if *self < min {
            min
        } else {
            *self
        }
    }
}

impl Camera {
    fn pitch_range() -> (Rad<f32>, Rad<f32>) {
        (Rad::from(Deg(-89.0)), Rad::from(Deg(89.0)))
    }
    fn fovy_range() -> (Rad<f32>, Rad<f32>) {
        (Rad::from(Deg(10.0)), Rad::from(Deg(120.0)))
    }

    pub fn update(&mut self, update: &CameraUpdate) {
        // Direct delta_position along yaw angle.
        let delta_position = Quaternion::from_axis_angle(Vector3::unit_y(), self.yaw) * update.delta_position;
        // Compute updates based on old state and input.
        let new_position = self.position + delta_position * self.positional_velocity * update.delta_time;
        let new_yaw = self.yaw + update.delta_yaw * self.angular_velocity * update.delta_time;
        let new_pitch = self.pitch + update.delta_pitch * self.angular_velocity * update.delta_time;
        let new_fovy = self.fovy + Rad(update.delta_scroll) * self.zoom_velocity * update.delta_time;

        // Apply updates.
        self.position = new_position;
        self.pitch = new_pitch.clamp(Self::pitch_range());
        self.yaw = new_yaw;
        self.fovy = new_fovy.clamp(Self::fovy_range());
    }

    pub fn interpolate(&mut self, b: &Camera, t: f32) {
        let u = 1.0 - t;

        self.position = self.position * t + b.position * u;
        self.yaw = self.yaw * t + b.yaw * u;
        self.pitch = (self.pitch * t + b.pitch * u).clamp(Self::pitch_range());
        self.fovy = (self.fovy * t + b.fovy * u).clamp(Self::fovy_range());
    }

    // Camera to world space.

    pub fn rot_from_cam_to_wld(&self) -> Quaternion<f32> {
        Quaternion::from_axis_angle(Vector3::unit_y(), self.yaw)
            * Quaternion::from_axis_angle(Vector3::unit_x(), self.pitch)
    }

    pub fn pos_from_cam_to_wld(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position) * Matrix4::from(self.rot_from_cam_to_wld())
    }

    // World to camera space.

    pub fn rot_from_wld_to_cam(&self) -> Quaternion<f32> {
        Quaternion::from_axis_angle(Vector3::unit_x(), -self.pitch)
            * Quaternion::from_axis_angle(Vector3::unit_y(), -self.yaw)
    }

    pub fn pos_from_wld_to_cam(&self) -> Matrix4<f32> {
        Matrix4::from(self.rot_from_wld_to_cam()) * Matrix4::from_translation(-self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[macro_use]
    use cgmath::*;

    #[test]
    fn mathy() {
        let yaw = Quaternion::from_axis_angle(Vector3::unit_y(), Deg(90.0));
        let pitch = Quaternion::from_axis_angle(Vector3::unit_x(), Deg(10.0));
        cgmath::assert_relative_eq!(Vector3::new(0.0, 0.0, -1.0), yaw * Vector3::unit_x());
        cgmath::assert_relative_eq!(Vector3::new(1.0, 0.0, 0.0), pitch * Vector3::unit_x());
        cgmath::assert_relative_eq!(
            Vector3::new(0.0, 0.0, -1.0),
            (yaw * pitch) * Vector3::unit_x(),
            epsilon = 0.00001
        );
    }

    #[test]
    fn inverse_identities() {
        let camera = Camera {
            position: Vector3::new(1.0, 2.0, 3.0),
            pitch: Rad::from(Deg(45.0)),
            yaw: Rad::from(Deg(90.0)),
            fovy: Rad::from(Deg(90.0)),
            positional_velocity: 1.0,
            angular_velocity: 1.0,
            zoom_velocity: 1.0,
        };

        cgmath::assert_relative_eq!(
            Quaternion::from_sv(1.0, Vector3::zero()),
            camera.rot_from_cam_to_wld() * camera.rot_from_wld_to_cam(),
        );

        cgmath::assert_relative_eq!(
            Matrix4::identity(),
            camera.pos_from_cam_to_wld() * camera.pos_from_wld_to_cam(),
        );

        let p_wld = Vector3::new(0.0, 1.0, 0.0);
        let p_cam = camera.pos_from_wld_to_cam() * p_wld.extend(1.0);

        cgmath::assert_relative_eq!(Vector4::new(3.0, -f32::sqrt(2.0), 0.0, 1.0), p_cam, epsilon = 0.00001);
    }
}
