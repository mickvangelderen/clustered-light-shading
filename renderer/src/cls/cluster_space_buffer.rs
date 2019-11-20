use crate::*;
use num_traits::{cast, Float};

#[derive(Debug, Copy, Clone)]
struct LerpCoeffs<T> {
    a: T,
    b: T,
}

impl<T> LerpCoeffs<T>
where
    T: Float,
{
    #[inline]
    fn new(x0: T, x1: T, y0: T, y1: T) -> Self {
        Self {
            a: (y1 - y0) / (x1 - x0),
            b: (y0 * x1 - y1 * x0) / (x1 - x0),
        }
    }

    #[inline]
    fn cast<U>(self) -> Option<LerpCoeffs<U>>
    where
        U: Float,
    {
        Some(LerpCoeffs {
            a: cast(self.a)?,
            b: cast(self.b)?,
        })
    }
}

#[repr(C)]
pub struct ClusterSpaceBuffer {
    dimensions: Vector3<u32>,
    cluster_count: u32,
    frustum: Frustum<f32>,
    _pad1: [f32; 2],
    clu_clp_to_clu_cam: Matrix4<f32>,
}

impl ClusterSpaceBuffer {
    pub fn new(dimensions: Vector3<u32>, frustum: Frustum<f64>, clu_clp_to_clu_cam: Matrix4<f64>) -> Self {
        Self {
            dimensions,
            cluster_count: dimensions.product(),
            frustum: frustum.cast().unwrap(),
            _pad1: Default::default(),
            clu_clp_to_clu_cam: clu_clp_to_clu_cam.cast().unwrap(),
        }
    }
}
