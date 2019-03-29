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

trait Clamp {
    fn clamp(&self, min: Self, max: Self) -> Self;
}

impl<T> Clamp for T
where
    T: PartialOrd,
    T: Copy,
{
    fn clamp(&self, min: Self, max: Self) -> Self {
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
    pub fn update(&mut self, update: &CameraUpdate) {
        // Direct delta_position along yaw angle.
        let delta_position =
            Quaternion::from_axis_angle(Vector3::unit_y(), -self.yaw) * update.delta_position;

        // Compute updates based on old state and input.
        let new_position =
            self.position + delta_position * self.positional_velocity * update.delta_time;
        let new_yaw = self.yaw + update.delta_yaw * self.angular_velocity * update.delta_time;
        let new_pitch = self.pitch + update.delta_pitch * self.angular_velocity * update.delta_time;
        let new_fovy =
            self.fovy + Rad(update.delta_scroll) * self.zoom_velocity * update.delta_time;

        // Apply updates.
        self.position = new_position;
        self.yaw = new_yaw % Rad::full_turn();
        self.pitch = new_pitch.clamp(Rad::from(Deg(-89.0)), Rad::from(Deg(89.0)));
        self.fovy = new_fovy.clamp(Rad::from(Deg(10.0)), Rad::from(Deg(80.0)));
    }

    pub fn orientation(&self) -> Quaternion<f32> {
        Quaternion::from_axis_angle(Vector3::unit_y(), -self.yaw)
            * Quaternion::from_axis_angle(Vector3::unit_x(), -self.pitch)
    }

    pub fn pos_from_wld_to_cam(&self) -> Matrix4<f32> {
        // Directly construct the inverse cam_to_wld transformation matrix.
        Matrix4::from(self.orientation().invert()) * Matrix4::from_translation(-self.position)
    }
}
