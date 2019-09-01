use crate::*;

impl_enum_and_enum_map! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
    enum CameraStage => struct CameraStages {
        RenderDepth => render_depth,
        CountFrags => count_frags,
    }
}

impl CameraStage {
    pub const VALUES: [CameraStage; 2] = [CameraStage::RenderDepth, CameraStage::CountFrags];

    pub fn title(self) -> &'static str {
        match self {
            CameraStage::RenderDepth => "render depth",
            CameraStage::CountFrags => "count #frags",
        }
    }
}

#[derive(Debug)]
pub struct ClusterCameraParameters {
    // Depth pass.
    pub frame_dims: Vector2<i32>,

    pub wld_to_cam: Matrix4<f64>,
    pub cam_to_wld: Matrix4<f64>,

    pub cam_to_clp: Matrix4<f64>,
    pub clp_to_cam: Matrix4<f64>,

    pub frustum: Frustum<f64>,
}

pub struct ClusterCameraResources {
    pub profilers: CameraStages<Profiler>,
    pub parameters: ClusterCameraParameters,
}

impl ClusterCameraResources {
    pub fn new(gl: &gl::Gl, parameters: ClusterCameraParameters) -> Self {
        Self {
            profilers: CameraStages::new(|_| Profiler::new(gl)),
            parameters,
        }
    }

    pub fn reset(&mut self, _gl: &gl::Gl, parameters: ClusterCameraParameters) {
        self.parameters = parameters;
    }
}

impl_frame_pool! {
    ClusterCameraResourcesPool,
    ClusterCameraResources,
    ClusterCameraResourcesIndex,
    ClusterCameraResourcesIndexIter,
    (gl: &gl::Gl, parameters: ClusterCameraParameters),
}
