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
    cam_to_clp_coeffs: [LerpCoeffs<f32>; 3],
    _pad2: [f32; 2],
    clp_to_cam_coeffs: [LerpCoeffs<f32>; 3],
    _pad3: [f32; 2],
    wld_to_cam: Matrix4<f32>,
    cam_to_wld: Matrix4<f32>,
}

impl ClusterSpaceBuffer {
    pub fn new(dimensions: Vector3<u32>, frustum: Frustum<f64>, wld_to_cam: Matrix4<f64>) -> Self {
        Self {
            dimensions,
            cluster_count: dimensions.product(),
            frustum: frustum.cast().unwrap(),
            _pad1: Default::default(),
            cam_to_clp_coeffs: [
                LerpCoeffs::new(frustum.x0, frustum.x1, 0.0, dimensions.x as f64)
                    .cast()
                    .unwrap(),
                LerpCoeffs::new(frustum.y0, frustum.y1, 0.0, dimensions.y as f64)
                    .cast()
                    .unwrap(),
                LerpCoeffs::new(frustum.z0, frustum.z1, 0.0, dimensions.z as f64)
                    .cast()
                    .unwrap(),
            ],
            _pad2: Default::default(),
            clp_to_cam_coeffs: [
                LerpCoeffs::new(0.0, dimensions.x as f64, frustum.x0, frustum.x1)
                    .cast()
                    .unwrap(),
                LerpCoeffs::new(0.0, dimensions.y as f64, frustum.y0, frustum.y1)
                    .cast()
                    .unwrap(),
                LerpCoeffs::new(0.0, dimensions.z as f64, frustum.z0, frustum.z1)
                    .cast()
                    .unwrap(),
            ],
            _pad3: Default::default(),
            wld_to_cam: wld_to_cam.cast().unwrap(),
            cam_to_wld: wld_to_cam.invert().unwrap().cast().unwrap(),
        }
    }
}
