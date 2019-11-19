use cgmath::*;
use num_traits::{cast, Float};

#[derive(Debug, Copy, Clone)]
pub struct Range3<T> {
    pub x0: T,
    pub x1: T,
    pub y0: T,
    pub y1: T,
    pub z0: T,
    pub z1: T,
}

impl<T: Float> Range3<T> {
    #[inline]
    pub fn from_vector(v: Vector3<T>) -> Self {
        Self {
            x0: T::zero(),
            x1: v.x,
            y0: T::zero(),
            y1: v.y,
            z0: T::zero(),
            z1: v.z,
        }
    }

    #[inline]
    pub fn from_point(p: Point3<T>) -> Self {
        Self {
            x0: p.x,
            x1: p.x,
            y0: p.y,
            y1: p.y,
            z0: p.z,
            z1: p.z,
        }
    }

    #[inline]
    pub fn from_points<I>(points: I) -> Option<Self>
    where
        I: IntoIterator<Item = Point3<T>>,
    {
        let mut points = points.into_iter();
        if let Some(first) = points.next() {
            Some(points.fold(Self::from_point(first), Self::include_point))
        // let mut range = Self::from_point(first);
        // for point in points {
        //     range = range.include_point(point);
        // }
        // Some(range)
        } else {
            None
        }
    }

    #[inline]
    pub fn include_point(self, p: Point3<T>) -> Self {
        Range3 {
            x0: self.x0.min(p.x),
            x1: self.x1.max(p.x),
            y0: self.y0.min(p.y),
            y1: self.y1.max(p.y),
            z0: self.z0.min(p.z),
            z1: self.z1.max(p.z),
        }
    }

    #[inline]
    pub fn dx(&self) -> T {
        self.x1 - self.x0
    }

    #[inline]
    pub fn dy(&self) -> T {
        self.y1 - self.y0
    }

    #[inline]
    pub fn dz(&self) -> T {
        self.z1 - self.z0
    }

    #[inline]
    pub fn min(&self) -> Point3<T> {
        Point3::new(self.x0, self.y0, self.z0)
    }

    #[inline]
    pub fn max(&self) -> Point3<T> {
        Point3::new(self.x1, self.y1, self.z1)
    }

    #[inline]
    pub fn center(&self) -> Point3<T> {
        let half: T = T::one() / (T::one() + T::one());
        Point3::new(
            (self.x0 + self.x1) * half,
            (self.y0 + self.y1) * half,
            (self.z0 + self.z1) * half,
        )
    }

    #[inline]
    pub fn delta(&self) -> Vector3<T> {
        // NOTE: Would use `self.max() - self.min()` but cgmath uses BaseNum
        // and it will affect everything depending on it so we just write
        // the code ourselves.
        Vector3::new(self.dx(), self.dy(), self.dz())
    }

    #[inline]
    pub fn vertices(&self) -> [Point3<T>; 8] {
        [
            Point3::new(self.x0, self.y0, self.z0),
            Point3::new(self.x1, self.y0, self.z0),
            Point3::new(self.x0, self.y1, self.z0),
            Point3::new(self.x1, self.y1, self.z0),
            Point3::new(self.x0, self.y0, self.z1),
            Point3::new(self.x1, self.y0, self.z1),
            Point3::new(self.x0, self.y1, self.z1),
            Point3::new(self.x1, self.y1, self.z1),
        ]
    }

    #[inline]
    pub fn line_mesh_indices(&self) -> [[u32; 2]; 12] {
        [
            // Back
            [0b000, 0b001],
            [0b001, 0b011],
            [0b011, 0b010],
            [0b010, 0b000],
            // Front
            [0b100, 0b101],
            [0b101, 0b111],
            [0b111, 0b110],
            [0b110, 0b100],
            // Side
            [0b000, 0b100],
            [0b001, 0b101],
            [0b010, 0b110],
            [0b011, 0b111],
        ]
    }

    #[inline]
    pub fn far_vertices(&self) -> [Point3<T>; 4] {
        [
            Point3::new(self.x0, self.y0, self.z0),
            Point3::new(self.x1, self.y0, self.z0),
            Point3::new(self.x0, self.y1, self.z0),
            Point3::new(self.x1, self.y1, self.z0),
        ]
    }

    #[inline]
    pub fn near_vertices(&self) -> [Point3<T>; 4] {
        [
            Point3::new(self.x0, self.y0, self.z1),
            Point3::new(self.x1, self.y0, self.z1),
            Point3::new(self.x0, self.y1, self.z1),
            Point3::new(self.x1, self.y1, self.z1),
        ]
    }

    #[inline]
    pub fn cast<U>(self) -> Option<Range3<U>>
    where
        U: Float,
    {
        Some(Range3 {
            x0: cast(self.x0)?,
            x1: cast(self.x1)?,
            y0: cast(self.y0)?,
            y1: cast(self.y1)?,
            z0: cast(self.z0)?,
            z1: cast(self.z1)?,
        })
    }
}
