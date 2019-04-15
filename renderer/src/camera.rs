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
    pub smooth_position: Vector3<f32>,
    pub position: Vector3<f32>,
    pub smooth_yaw: Rad<f32>,
    pub yaw: Rad<f32>,
    pub smooth_pitch: Rad<f32>,
    pub pitch: Rad<f32>,
    pub smooth_fovy: Rad<f32>,
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

        // NOTE: We interpolate per simulation update, we can't scale these by delta_time.
        let smooth_weight = 0.8;
        let actual_weight = 1.0 - smooth_weight;

        // Apply updates.
        self.smooth_position = self.smooth_position * smooth_weight + new_position * actual_weight;
        self.position = new_position;

        let correction = Rad::full_turn() * (new_yaw / Rad::full_turn()).trunc();
        self.smooth_yaw = (self.smooth_yaw * smooth_weight + new_yaw * actual_weight) - correction;
        self.yaw = new_yaw - correction;

        let min_pitch = Rad::from(Deg(-89.0));
        let max_pitch = Rad::from(Deg(89.0));
        self.smooth_pitch = (self.smooth_pitch * smooth_weight + new_pitch * actual_weight)
            .clamp(min_pitch, max_pitch);
        self.pitch = new_pitch.clamp(min_pitch, max_pitch);

        let min_fovy = Rad::from(Deg(10.0));
        let max_fovy = Rad::from(Deg(120.0));
        self.smooth_fovy =
            (self.smooth_fovy * smooth_weight + new_fovy * actual_weight).clamp(min_fovy, max_fovy);
        self.fovy = new_fovy.clamp(min_fovy, max_fovy);
    }

    pub fn smooth_orientation(&self) -> Quaternion<f32> {
        Quaternion::from_axis_angle(Vector3::unit_y(), -self.smooth_yaw)
            * Quaternion::from_axis_angle(Vector3::unit_x(), -self.smooth_pitch)
    }

    pub fn orientation(&self) -> Quaternion<f32> {
        Quaternion::from_axis_angle(Vector3::unit_y(), -self.yaw)
            * Quaternion::from_axis_angle(Vector3::unit_x(), -self.pitch)
    }

    pub fn smooth_pos_from_wld_to_cam(&self) -> Matrix4<f32> {
        // Directly construct the inverse cam_to_wld transformation matrix.
        Matrix4::from(self.smooth_orientation().invert())
            * Matrix4::from_translation(-self.smooth_position)
    }

    pub fn pos_from_wld_to_cam(&self) -> Matrix4<f32> {
        // Directly construct the inverse cam_to_wld transformation matrix.
        Matrix4::from(self.orientation().invert()) * Matrix4::from_translation(-self.position)
    }
}
