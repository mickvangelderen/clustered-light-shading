use crate::*;

impl_enum_and_enum_map! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
    enum CameraStage => struct CameraStages {
        Camera => camera,
        CountFrags => count_frags,
    }
}

impl CameraStage {
    pub const VALUES: [CameraStage; 2] = [CameraStage::Camera, CameraStage::CountFrags];

    pub fn title(self) -> &'static str {
        match self {
            CameraStage::Camera => "camera",
            CameraStage::CountFrags => "count frags",
        }
    }
}

#[derive(Debug)]
pub struct ClusterCameraParameters {
    pub draw_resources_index: usize,

    pub frame_dims: Vector2<i32>,

    pub camera: CameraParameters,
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
