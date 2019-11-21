use crate::*;

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
