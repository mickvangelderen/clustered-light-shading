use crate::*;

pub struct World {
    pub epoch: Instant,
    pub running: bool,
    pub focus: bool,
    pub win_dpi: f64,
    pub win_size: glutin::dpi::PhysicalSize,
    pub resource_dir: PathBuf,
    pub configuration_path: PathBuf,
    pub keyboard_state: KeyboardState,
    pub tick: u64,
    pub paused: bool,
    pub current: ::incremental::Current,
    pub clear_color: [f32; 3],
    pub window_mode: WindowMode,
    pub display_mode: u32,
    pub depth_prepass: bool,
    pub gl_log_regex: Regex,
    pub target_camera_key: CameraKey,
    pub transition_camera: camera::TransitionCamera,
    pub cameras: CameraMap<camera::SmoothCamera>,
    pub rain_drops: Vec<rain::Particle>,
    pub shader_compiler: ShaderCompiler,
}

impl World {
    pub fn target_camera(&self) -> &camera::SmoothCamera {
        &self.cameras[self.target_camera_key]
    }

    pub fn target_camera_mut(&mut self) -> &mut camera::SmoothCamera {
        &mut self.cameras[self.target_camera_key]
    }
}
