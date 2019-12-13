pub(crate) use log::*;
use renderer::configuration::{self, Configuration};
use std::path::PathBuf;
use std::process::Command;

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
            Self::Ortho { size } => format!("ortho_{:04.0}", size * 100.0),
            Self::Persp { size } => format!("persp_{:04}", size),
        }
    }

    pub fn apply(&self, cfg: &mut Configuration) {
        match *self {
            Self::Ortho { size } => {
                cfg.clustered_light_shading.projection = configuration::ClusteringProjection::Orthographic;
                cfg.clustered_light_shading.orthographic_sides = configuration::Vector3 {
                    x: size,
                    y: size,
                    z: size,
                };
            }
            Self::Persp { size } => {
                cfg.clustered_light_shading.projection = configuration::ClusteringProjection::Perspective;
                cfg.clustered_light_shading.perspective_pixels = configuration::Vector2 { x: size, y: size };
            }
        }
    }
}

struct Lighting {
    count: u32,
    attenuation: configuration::Attenuation,
}

impl Lighting {
    pub fn apply(&self, cfg: &mut Configuration) {
        cfg.rain.max_count = self.count;
        cfg.light.attenuation = self.attenuation;
    }
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
            replay_path: PathBuf::from("replay_suntem.bin"),
        },
    ];

    let base_cfg = Configuration::read("resources/profile_configuration.toml");

    let lightings = [
        Lighting {
            count: 1000,
            attenuation: configuration::Attenuation {
                i: 100.0,
                i0: 0.5,
                r0: 0.1,
            },
        },
        Lighting {
            count: 10_000,
            attenuation: configuration::Attenuation {
                i: 20.0,
                i0: 0.5,
                r0: 0.1,
            },
        },
        Lighting {
            count: 100_000,
            attenuation: configuration::Attenuation {
                i: 1.0,
                i0: 0.5,
                r0: 0.1,
            },
        },
    ];

    let techniques: Vec<Technique> = [1.0, 2.0, 4.0, 8.0, 16.0]
        .iter()
        .map(|&n| Technique::Ortho { size: n })
        .chain([16, 32, 64, 128].iter().map(|&n| Technique::Persp { size: n }))
        .collect();

    for scene in scenes.iter() {
        for lighting in lightings.iter() {
            for technique in techniques.iter() {
                let name = format!("{}_{:07}_{}", scene.short_name, lighting.count, technique.name());
                info!("Profiling {}...", &name);
                let mut cfg = base_cfg.clone();
                cfg.global.mode = configuration::ApplicationMode::Replay;
                cfg.global.scene_path = scene.scene_path.clone();
                cfg.replay.path = scene.replay_path.clone();
                cfg.profiling.name = Some(PathBuf::from(name.clone()));

                lighting.apply(&mut cfg);
                technique.apply(&mut cfg);

                let profiling_dir = PathBuf::from("profiling").join(name);
                let configuration_path = profiling_dir.join("configuration.toml");
                cfg.write(&configuration_path).unwrap();
                run_with_configuration(configuration_path.to_str().unwrap());
            }
        }
    }

    let tuned_techniques = vec![Technique::Ortho { size: 4.0 }, Technique::Persp { size: 64 }];

    let groupings = vec![
        ("indi", configuration::ClusteringGrouping::Individual),
        ("encl", configuration::ClusteringGrouping::Enclosed),
    ];

    for scene in scenes.iter() {
        for lighting in lightings.iter() {
            for technique in tuned_techniques.iter() {
                for &(grouping_name, grouping) in groupings.iter() {
                    let name = format!(
                        "stereo_{}_{:07}_{}_{}",
                        scene.short_name,
                        lighting.count,
                        grouping_name,
                        technique.name()
                    );
                    info!("Profiling {}...", &name);
                    let mut cfg = base_cfg.clone();
                    cfg.global.mode = configuration::ApplicationMode::Replay;
                    cfg.global.scene_path = scene.scene_path.clone();
                    cfg.replay.path = scene.replay_path.clone();
                    cfg.profiling.name = Some(PathBuf::from(name.clone()));

                    cfg.virtual_stereo.enabled = true;
                    cfg.clustered_light_shading.grouping = grouping;

                    lighting.apply(&mut cfg);
                    technique.apply(&mut cfg);

                    let profiling_dir = PathBuf::from("profiling").join(name);
                    let configuration_path = profiling_dir.join("configuration.toml");
                    cfg.write(&configuration_path).unwrap();
                    run_with_configuration(configuration_path.to_str().unwrap());
                }
            }
        }
    }
}
