use cgmath::*;
use gl_typed as gl;

#[derive(Debug, Copy, Clone)]
pub struct Viewport<S> {
    pub origin: Point2<S>,
    pub dimensions: Vector2<S>,
}

impl<S> Viewport<S>
where
    S: BaseNum
{
    #[inline]
    pub fn from_dimensions(dimensions: Vector2<S>) -> Self {
        Viewport {
            origin: Point2::origin(),
            dimensions,
        }
    }

    #[inline]
    pub fn from_coordinates(p0: Point2<S>, p1: Point2<S>) -> Self {
        Viewport {
            origin: p0,
            dimensions: p1 - p0,
        }
    }
}

impl Viewport<i32> {
    pub fn set(&self, gl: &gl::Gl) {
        unsafe {
            gl.viewport(self.origin.x, self.origin.y, self.dimensions.x, self.dimensions.y);
        }
    }
}
