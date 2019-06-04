use cgmath::*;

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

    // #[inline]
    // pub fn x0(&self) -> S {
    //     self.x0
    // }

    // #[inline]
    // pub fn y0(&self) -> S {
    //     self.y0
    // }

    // #[inline]
    // pub fn x1(&self) -> S {
    //     self.x0 + self.dx
    // }

    // #[inline]
    // pub fn y1(&self) -> S {
    //     self.y0 + self.dy
    // }

    // #[inline]
    // pub fn dx(&self) -> S {
    //     self.dx
    // }

    // #[inline]
    // pub fn dy(&self) -> S {
    //     self.dy
    // }

    // #[inline]
    // pub fn p0(&self) -> [S; 2] {
    //     [self.x0(), self.y0()]
    // }

    // #[inline]
    // pub fn p1(&self) -> [S; 2] {
    //     [self.x1(), self.y1()]
    // }

    // #[inline]
    // pub fn dp(&self) -> [S; 2] {
    //     [self.dx(), self.dy()]
    // }

    // #[inline]
    // pub fn rx(&self) -> (S, S) {
    //     (self.x0(), self.x1())
    // }

    // #[inline]
    // pub fn ry(&self) -> (S, S) {
    //     (self.y0(), self.y1())
    // }
}
