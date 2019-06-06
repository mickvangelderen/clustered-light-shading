pub trait ElementStateExt {
    fn to_f32(self) -> f32;

    fn is_pressed(self) -> bool;
    fn is_released(self) -> bool;
}

impl ElementStateExt for glutin::ElementState {
    #[inline]
    fn to_f32(self) -> f32 {
        match self {
            glutin::ElementState::Released => 0.0,
            glutin::ElementState::Pressed => 1.0,
        }
    }

    #[inline]
    fn is_pressed(self) -> bool {
        match self {
            glutin::ElementState::Released => false,
            glutin::ElementState::Pressed => true,
        }
    }

    #[inline]
    fn is_released(self) -> bool {
        match self {
            glutin::ElementState::Released => true,
            glutin::ElementState::Pressed => false,
        }
    }
}
