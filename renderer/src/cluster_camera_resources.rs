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
}

pub struct ClusterCameraParameters {
    // Depth pass.
    pub frame_dims: Vector2<i32>,

    pub wld_to_cam: Matrix4<f64>,
    pub cam_to_wld: Matrix4<f64>,

    pub cam_to_clp: Matrix4<f64>,
    pub clp_to_cam: Matrix4<f64>,

    // Cluster orientation and dimensions.
    pub wld_to_hmd: Matrix4<f64>,
    pub hmd_to_wld: Matrix4<f64>,

    pub hmd_to_clp: Matrix4<f64>,
    pub clp_to_hmd: Matrix4<f64>,
}

pub struct ClusterCameraResourcesPool {
    resources: Vec<ClusterCameraResources>,
    used: usize,
}

impl ClusterCameraResourcesPool {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            used: 0,
        }
    }

    pub fn next_unused(&mut self, gl: &gl::Gl, parameters: ClusterCameraParameters) -> ClusterCameraResourcesIndex {
        let index = self.used;
        self.used += 1;

        if self.resources.len() < index + 1 {
            self.resources.push(ClusterCameraResources::new(&gl, parameters));
        } else {
            self.resources[index].parameters = parameters;
        }

        ClusterCameraResourcesIndex(index)
    }

    pub fn used_slice(&self) -> &[ClusterCameraResources] {
        &self.resources[0..self.used]
    }

    pub fn used_count(&self) -> usize {
        self.used
    }

    pub fn reset(&mut self) {
        self.used = 0;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ClusterCameraResourcesIndex(pub usize);

impl std::ops::Index<ClusterCameraResourcesIndex> for ClusterCameraResourcesPool {
    type Output = ClusterCameraResources;

    fn index(&self, index: ClusterCameraResourcesIndex) -> &Self::Output {
        &self.resources[index.0]
    }
}

impl std::ops::IndexMut<ClusterCameraResourcesIndex> for ClusterCameraResourcesPool {
    fn index_mut(&mut self, index: ClusterCameraResourcesIndex) -> &mut Self::Output {
        &mut self.resources[index.0]
    }
}
