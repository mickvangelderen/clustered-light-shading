use cgmath::*;

pub trait FromHmd<T> {
    fn from_hmd(val: T) -> Self;
}

pub trait HmdInto<T> {
    fn hmd_into(self) -> T;
}

impl<T, U> HmdInto<U> for T
where
    U: FromHmd<T>,
{
    #[inline]
    fn hmd_into(self) -> U {
        U::from_hmd(self)
    }
}

impl FromHmd<[[f32; 4]; 4]> for Matrix4<f32> {
    #[inline]
    fn from_hmd(m: [[f32; 4]; 4]) -> Self {
        Matrix4::from_cols(
            Vector4::new(m[0][0], m[1][0], m[2][0], m[3][0]),
            Vector4::new(m[0][1], m[1][1], m[2][1], m[3][1]),
            Vector4::new(m[0][2], m[1][2], m[2][2], m[3][2]),
            Vector4::new(m[0][3], m[1][3], m[2][3], m[3][3]),
        )
    }
}

impl FromHmd<[[f32; 4]; 3]> for Matrix4<f32> {
    #[inline]
    fn from_hmd(m: [[f32; 4]; 3]) -> Self {
        Matrix4::from_cols(
            Vector4::new(m[0][0], m[1][0], m[2][0], 0.0),
            Vector4::new(m[0][1], m[1][1], m[2][1], 0.0),
            Vector4::new(m[0][2], m[1][2], m[2][2], 0.0),
            Vector4::new(m[0][3], m[1][3], m[2][3], 1.0),
        )
    }
}

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

pub trait ArrayExt {
    fn dominant_axis(self) -> usize;
}

impl <S> ArrayExt for Vector3<S> where S: PartialOrd {
    fn dominant_axis(self) -> usize {
        let mut dominant_axis = 0;
        for axis in 1..3 {
            if self[axis] > self[dominant_axis] {
                dominant_axis = axis
            }
        }
        dominant_axis
    }
}

pub trait RadExt {
    fn cast<U>(self) -> Option<Rad<U>> where U: num_traits::Float;
}

impl<S> RadExt for Rad<S> where S: num_traits::Float {
    #[inline]
    fn cast<U>(self) -> Option<Rad<U>> where U: num_traits::Float {
        Some(Self(num_traits::cast(self.0)?))
    }
}
