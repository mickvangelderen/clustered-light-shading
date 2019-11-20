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
            CameraStage::RenderDepth => "cluster.camera.depth",
            CameraStage::CountFrags => "cluster.camera.count_frags",
        }
    }
}

#[derive(Debug)]
pub struct ClusterCameraParameters {
    pub frame_dims: Vector2<i32>,

    // pub wld_to_cam: Matrix4<f64>,
    // pub cam_to_wld: Matrix4<f64>,

    // pub cam_to_clp: Matrix4<f64>,
    // pub clp_to_cam: Matrix4<f64>,

    pub wld_to_ren_clp: Matrix4<f64>,
    pub ren_clp_to_wld: Matrix4<f64>,

    pub frustum: Frustum<f64>,
}

pub struct ClusterCameraResources {
    pub profilers: CameraStages<SampleIndex>,
    pub parameters: ClusterCameraParameters,
}

impl ClusterCameraResources {
    pub fn new(_gl: &gl::Gl, profiling_context: &mut ProfilingContext, parameters: ClusterCameraParameters) -> Self {
        Self {
            profilers: CameraStages::new(|stage| profiling_context.add_sample(stage.title())),
            parameters,
        }
    }

    pub fn reset(&mut self, _gl: &gl::Gl, _profiling_context: &mut ProfilingContext, parameters: ClusterCameraParameters) {
        self.parameters = parameters;
    }
}

impl_frame_pool! {
    ClusterCameraResourcesPool,
    ClusterCameraResources,
    ClusterCameraResourcesIndex,
    ClusterCameraResourcesIndexIter,
    (gl: &gl::Gl, profiling_context: &mut ProfilingContext, parameters: ClusterCameraParameters),
}
