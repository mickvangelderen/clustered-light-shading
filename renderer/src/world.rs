use crate::*;

pub struct World {
    pub running: bool,
    pub focus: bool,
    pub win_dpi: f64,
    pub win_size: glutin::dpi::PhysicalSize,
    pub resource_dir: PathBuf,
    pub configuration_path: PathBuf,
    pub keyboard_state: KeyboardState,
    pub tick: u64,
    pub global: ic::Global,
    pub clear_color: [f32; 3],
    pub window_mode: WindowMode,
    pub depth_prepass: bool,
    pub light_space: ic::Leaf<LightSpace>,
    pub light_space_regex: Regex,
    pub render_technique: ic::Leaf<RenderTechnique>,
    pub render_technique_regex: Regex,
    pub attenuation_mode: ic::Leaf<AttenuationMode>,
    pub attenuation_mode_regex: Regex,
    pub gl_log_regex: Regex,
    pub sources: Vec<ShaderSource>,
    pub target_camera_key: CameraKey,
    pub transition_camera: camera::TransitionCamera,
    pub cameras: CameraMap<camera::SmoothCamera>,
    pub rain_drops: Vec<rain::Particle>,
}

impl World {
    pub fn target_camera(&self) -> &camera::SmoothCamera {
        &self.cameras[self.target_camera_key]
    }

    pub fn target_camera_mut(&mut self) -> &mut camera::SmoothCamera {
        &mut self.cameras[self.target_camera_key]
    }

    pub fn add_source(&mut self, path: impl AsRef<Path>) -> usize {
        let index = self.sources.len();
        self.sources.push(ShaderSource {
            path: self.resource_dir.join(path),
            modified: ic::Modified::clean(&self.global),
        });
        index
    }
}

