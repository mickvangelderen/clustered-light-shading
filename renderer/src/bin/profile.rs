use renderer::configuration::{self, Configuration};
use std::path::PathBuf;
use std::process::Command;
pub(crate) use log::*;

pub fn run_with_configuration(configuration_path: &str) {
    let _ = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "renderer",
            "--",
            "--configuration-path",
            configuration_path,
        ])
        .status()
        .expect("failed to execute process");

    let _ = Command::new("cargo")
        .args(&["run", "--bin", "profiling_reader"])
        .status()
        .expect("failed to execute process");
}

struct Scene {
    short_name: &'static str,
    scene_path: PathBuf,
    replay_path: PathBuf,
}

enum Technique {
    Ortho { size: f64 },
    Persp { size: u32 },
}

impl Technique {
    pub fn name(&self) -> String {
        match *self {
            Self::Ortho { size } => format!("ortho_{:4.0}", size * 100.0),
            Self::Persp { size } => format!("persp_{:4}", size),
        }
    }

    pub fn apply(&self, cfg: &mut Configuration) {
        match *self {
            Self::Ortho { size } => {
                cfg.clustered_light_shading.orthographic_sides = configuration::Vector3 {
                    x: size,
                    y: size,
                    z: size,
                };
            }
            Self::Persp { size } => {
                cfg.clustered_light_shading.perspective_pixels = configuration::Vector2 { x: size, y: size };
            }
        }
    }
}

struct Lighting {
    count: u32,
    intensity: f64,
}

pub fn main() {
    env_logger::init();

    let scenes = [
        Scene {
            short_name: "bistro",
            scene_path: PathBuf::from("bistro/Bistro_Exterior.bin"),
            replay_path: PathBuf::from("replay_bistro.bin"),
        },
        Scene {
            short_name: "suntem",
            scene_path: PathBuf::from("sun_temple/SunTemple.bin"),
            replay_path: PathBuf::from("replay_sun_temple.bin"),
        },
    ];

    let base_cfg = Configuration::read("resources/profile_configuration.toml");

    let lightings = [
        Lighting {
            count: 1000,
            intensity: 20.0,
        },
        Lighting {
            count: 10_000,
            intensity: 10.0,
        },
        Lighting {
            count: 100_000,
            intensity: 5.0,
        },
    ];

    let techniques: Vec<Technique> = [1.0, 1.5, 2.0, 4.0]
        .iter()
        .map(|&n| Technique::Ortho { size: n })
        .chain([16, 24, 32, 48, 64].iter().map(|&n| Technique::Persp { size: n }))
        .collect();

    for scene in scenes.iter() {
        for lighting in lightings.iter() {
            for technique in techniques.iter() {
                let name = format!(
                    "{}_{:07}_{}",
                    scene.short_name,
                    lighting.count,
                    technique.name(),
                );
                info!("Profiling {}...", &name);
                let mut cfg = base_cfg.clone();
                cfg.global.mode = configuration::ApplicationMode::Replay;
                cfg.global.scene_path = scene.scene_path.clone();
                cfg.replay.path = scene.replay_path.clone();
                cfg.profiling.name = Some(PathBuf::from(name.clone()));

                cfg.light.attenuation.i = lighting.intensity;
                cfg.rain.max_count = lighting.count;

                technique.apply(&mut cfg);

                let profiling_dir = PathBuf::from("profiling").join(name);
                let configuration_path = profiling_dir.join("configuration.toml");
                cfg.write(&configuration_path).unwrap();
                run_with_configuration(configuration_path.to_str().unwrap());
            }
        }
    }
}
