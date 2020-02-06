use crate::*;

impl_enum_and_enum_map! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
    enum CameraStage => struct CameraStages {
        Camera => camera,
        CountFrags => count_frags,
        CountOpaqueMaskedFrags => count_opaque_masked_frags,
        CountOpaqueFrags => count_opaque_frags,
        CountMaskedFrags => count_masked_frags,
        CountTransparentFrags => count_transparent_frags,
    }
}

impl CameraStage {
    pub const VALUES: [CameraStage; 2] = [CameraStage::Camera, CameraStage::CountFrags];

    pub fn title(self) -> &'static str {
        match self {
            CameraStage::Camera => "camera",
            CameraStage::CountFrags => "count frags",
            CameraStage::CountOpaqueMaskedFrags => "opaque and masked",
            CameraStage::CountOpaqueFrags => "opaque",
            CameraStage::CountMaskedFrags => "masked",
            CameraStage::CountTransparentFrags => "transparent",
        }
    }
}

#[derive(Debug)]
pub struct ClusterCameraParameters {
    pub main_resources_index: usize,
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

    pub fn reset(
        &mut self,
        _gl: &gl::Gl,
        _profiling_context: &mut ProfilingContext,
        parameters: ClusterCameraParameters,
    ) {
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
