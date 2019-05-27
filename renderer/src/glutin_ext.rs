pub trait ElementStateExt {
    fn to_f32(self) -> f32;
}

impl ElementStateExt for glutin::ElementState {
    fn to_f32(self) -> f32 {
        match self {
            glutin::ElementState::Released => 0.0,
            glutin::ElementState::Pressed => 1.0,
        }
    }
}
