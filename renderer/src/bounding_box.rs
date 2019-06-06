use cgmath::*;
use crate::cgmath_ext::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BoundingBox<S> {
    pub min: Point3<S>,
    pub max: Point3<S>,
}

impl<S: BaseFloat> BoundingBox<S> {
    pub fn from_point(p: Point3<S>) -> Self {
        BoundingBox { min: p, max: p }
    }

    pub fn enclose(self, p: Point3<S>) -> Self {
        BoundingBox {
            min: Point3::partial_min_element_wise(self.min, p),
            max: Point3::partial_max_element_wise(self.max, p),
        }
    }
}

impl<S: BaseFloat> BoundingBox<S> {
    pub fn delta(&self) -> Vector3<S> {
        self.max - self.min
    }
}

impl<S: BaseFloat> BoundingBox<S> {
    /// Component-wise casting to another type
    #[inline]
    pub fn cast<T: BaseFloat>(self) -> Option<BoundingBox<T>> {
        Some(BoundingBox {
            min: self.min.cast()?,
            max: self.max.cast()?,
        })
    }
}
