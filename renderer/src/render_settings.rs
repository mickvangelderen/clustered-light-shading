pub enum LightAssignment {
    CPU,
    GPU,
}

pub enum ClusterSpace {
    SharedPerspective,
    SharedOrthogonal,
    IndividualPerspective,
}

pub enum ShadingTechnique {
    Naive,
    Tiled,
    Clustered,
}

pub enum IndexingTechnique {
    Naive,
    Morton,
}

// obj - wld -[cam]- bdy -[pose]- hmd
//                    |-[eye]- cam -[proj]- clp
//                    |-[frus bounds]- cls

#[repr(C, align(256))]
pub struct GlobalData {
    pos_from_wld_to_cam: Matrix4<f64>,
    pos_from_cam_to_wld: Matrix4<f64>,
}

#[repr(C)]
pub union GlobalDataUnion {
    tiled: GlobalTiledData,
    clustered: GlobalClusteredData,
}

pub struct Cluster([u32; 32]);

pub struct ClusterPage([[[Cluster; 8]; 8]; 8]);

#[repr(C, align(256))]
pub struct GlobalClusterData {
    pos_from_wld_to_cls: Matrix4<f64>,
    pos_from_cls_to_wld: Matrix4<f64>,
}

#[repr(C, align(256))]
pub struct GlobalTiledData {
    // Don't know what I need yet.
}

pub struct ViewData {

}

pub struct ViewClusterData {

}

pub fn render(shading_technique: ShadingTechnique) {

    match shading_technique {
        ShadingTechnique::Naive => {
            let global_data = GlobalData::new();
            let view_data = ViewData::new();
        }
        ShadingTechnique::Tiled => {
            let global_data = TiledGlobalData::new();
            let view_data = TiledViewData::new();
        }
        ShadingTechnique::Clustered { cluster_space, light_assignment } => {
            match cluster_space

        }
    }
}
