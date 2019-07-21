use cgmath::*;

pub trait Matrix4BaseNumExt<S> {
    fn truncate(self) -> Matrix3<S>;
}

impl<S: BaseNum> Matrix4BaseNumExt<S> for Matrix4<S> {
    fn truncate(self) -> Matrix3<S> {
        Matrix3::from_cols(self[0].truncate(), self[1].truncate(), self[2].truncate())
    }
}

pub trait Matrix4BaseFloatExt<S>: Sized {
    fn from_scale_vector(vector: Vector3<S>) -> Self;
}

impl<S: BaseFloat> Matrix4BaseFloatExt<S> for Matrix4<S> {
    fn from_scale_vector(vector: Vector3<S>) -> Self {
        Matrix4::from_nonuniform_scale(vector.x, vector.y, vector.z)
    }
}

pub trait PartialOrdExt {
    fn partial_min(self, rhs: Self) -> Self;
    fn partial_max(self, rhs: Self) -> Self;
    fn partial_clamp(self, min: Self, max: Self) -> Self;
}

impl<S> PartialOrdExt for S
where
    S: std::cmp::PartialOrd,
{
    fn partial_min(self, rhs: Self) -> Self {
        if self < rhs {
            self
        } else {
            rhs
        }
    }

    fn partial_max(self, rhs: Self) -> Self {
        if self > rhs {
            self
        } else {
            rhs
        }
    }

    fn partial_clamp(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

pub trait Point3Ext<S1> {
    fn zip<S2, U, F>(p1: Point3<S1>, p2: Point3<S2>, f: F) -> Point3<U>
    where
        F: FnMut(S1, S2) -> U;
    fn zip3<S2, S3, U, F>(p1: Point3<S1>, p2: Point3<S2>, p3: Point3<S3>, f: F) -> Point3<U>
    where
        F: FnMut(S1, S2, S3) -> U;
}

impl<S1> Point3Ext<S1> for Point3<S1> {
    fn zip<S2, U, F>(p1: Point3<S1>, p2: Point3<S2>, mut f: F) -> Point3<U>
    where
        F: FnMut(S1, S2) -> U,
    {
        Point3 {
            x: f(p1.x, p2.x),
            y: f(p1.y, p2.y),
            z: f(p1.z, p2.z),
        }
    }
    fn zip3<S2, S3, U, F>(p1: Point3<S1>, p2: Point3<S2>, p3: Point3<S3>, mut f: F) -> Point3<U>
    where
        F: FnMut(S1, S2, S3) -> U,
    {
        Point3 {
            x: f(p1.x, p2.x, p3.x),
            y: f(p1.y, p2.y, p3.y),
            z: f(p1.z, p2.z, p3.z),
        }
    }
}

pub trait ElementWiseExt<Rhs = Self> {
    fn partial_min_element_wise(self, min: Rhs) -> Self;
    fn partial_max_element_wise(self, max: Rhs) -> Self;
    fn partial_clamp_element_wise(self, min: Rhs, max: Rhs) -> Self;
}

impl<S> ElementWiseExt for Point3<S>
where
    S: PartialOrd,
{
    fn partial_min_element_wise(self, min: Self) -> Self {
        Point3::zip(self, min, S::partial_min)
    }

    fn partial_max_element_wise(self, max: Self) -> Self {
        Point3::zip(self, max, S::partial_max)
    }

    fn partial_clamp_element_wise(self, min: Self, max: Self) -> Self {
        Point3::zip3(self, min, max, S::partial_clamp)
    }
}
