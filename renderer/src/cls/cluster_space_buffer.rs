use crate::*;

#[repr(C)]
pub struct ClusterSpaceBuffer {
    dimensions: Vector3<u32>,
    cluster_count: u32,
    frustum: Frustum<f32>,
    _pad0: [f32; 2],
    cam_to_clp: ClusterSpaceCoefficients,
    clp_to_cam: ClusterSpaceCoefficients,
}

#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct ClusterSpaceCoefficients {
    ax: f32,
    bx: f32,
    ay: f32,
    by: f32,
    az: f32,
    bz: f32,
    _pad: [f32; 2],
}

/// linear interpolation coefficients
fn lic<T: num_traits::Float>(x0: T, x1: T, y0: T, y1: T) -> (T, T) {
    let d = T::one() / (x1 - x0);
    ((y1 - y0) * d, (x1 * y0 - x0 * y1) * d)
}

impl ClusterSpaceCoefficients {
    #[inline]
    pub fn orthographic(frustum: &Frustum<f64>, dimensions: Vector3<f64>) -> Self {
        let (ax, bx) = lic(frustum.x0, frustum.x1, 0.0, dimensions.x);
        let (ay, by) = lic(frustum.y0, frustum.y1, 0.0, dimensions.y);
        let (az, bz) = lic(frustum.z0, frustum.z1, 0.0, dimensions.z);
        Self {
            ax: ax as f32,
            bx: bx as f32,
            ay: ay as f32,
            by: by as f32,
            az: az as f32,
            bz: bz as f32,
            _pad: Default::default(),
        }
    }

    #[inline]
    pub fn inverse_orthographic(frustum: &Frustum<f64>, dimensions: Vector3<f64>) -> Self {
        let (ax, bx) = lic(0.0, dimensions.x, frustum.x0, frustum.x1);
        let (ay, by) = lic(0.0, dimensions.y, frustum.y0, frustum.y1);
        let (az, bz) = lic(0.0, dimensions.z, frustum.z0, frustum.z1);
        Self {
            ax: ax as f32,
            bx: bx as f32,
            ay: ay as f32,
            by: by as f32,
            az: az as f32,
            bz: bz as f32,
            _pad: Default::default(),
        }
    }

    #[inline]
    pub fn perspective(frustum: &Frustum<f64>, dimensions: Vector3<f64>) -> Self {
        let (ax, bx) = lic(frustum.x0, frustum.x1, 0.0, dimensions.x);
        let (ay, by) = lic(frustum.y0, frustum.y1, 0.0, dimensions.y);
        // // Linear.
        // let (az, bz) = {
        //     let d = one / (frustum.z1 - frustum.z0);
        //     (
        //         (range.z1 - range.z0) * (frustum.z0 * frustum.z1) * d,
        //         (frustum.z1 * range.z1 - frustum.z0 * range.z0) * d,
        //     )
        // };

        // Geometric: z_clp = Z - ln(z_cam / fz1) / ln(1.0 + d)
        let (az, bz) = {
            let add_d_1 = (frustum.z0 / frustum.z1).powf(1.0/dimensions.z);
            ((1.0 / frustum.z1), 1.0 / add_d_1.ln())
        };

        Self {
            ax: ax as f32,
            bx: bx as f32,
            ay: ay as f32,
            by: by as f32,
            az: az as f32,
            bz: bz as f32,
            _pad: Default::default(),
        }
    }

    #[inline]
    pub fn inverse_perspective(frustum: &Frustum<f64>, dimensions: Vector3<f64>) -> Self {
        let (ax, bx) = lic(0.0, dimensions.x, frustum.x0, frustum.x1);
        let (ay, by) = lic(0.0, dimensions.y, frustum.y0, frustum.y1);
        // // Linear
        // let (az, bz) = {
        //     let d = one / ((dimensions.z1) * frustum.z0 * frustum.z1);
        //     ((frustum.z1 - frustum.z0) * d, (-dimensions.z * frustum.z1) * d)
        // };

        // Geometric: z_cam = fz1 * (1.0 + d)^(Z - z_clp)
        let (az, bz) = {
            let add_d_1 = (frustum.z0 / frustum.z1).powf(1.0/dimensions.z);
            (frustum.z1, add_d_1)
        };

        Self {
            ax: ax as f32,
            bx: bx as f32,
            ay: ay as f32,
            by: by as f32,
            az: az as f32,
            bz: bz as f32,
            _pad: Default::default(),
        }
    }
}

impl ClusterSpaceBuffer {
    pub fn from(
        resources: &ClusterResources
    ) -> Self {
        let dimensions = resources.computed.dimensions;
        Self {
            dimensions,
            cluster_count: dimensions.product(),
            frustum: resources.computed.frustum.cast().unwrap(),
            _pad0: Default::default(),
            cam_to_clp: resources.computed.cam_to_clp,
            clp_to_cam: resources.computed.clp_to_cam,
        }
    }
}
