use crate::*;

use ::incremental::{Branch, Graph as GraphRevision, Leaf, RootToken, Token};
use std::cell::{Ref, RefCell};
use std::io::prelude::*;
use std::path::PathBuf;

pub type FileIndex = usize;

#[derive(Debug)]
pub struct File {
    path: PathBuf,
    signal: Leaf<()>,
    sections: Branch<Vec<Section>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Section {
    Verbatim(String),
    Include(PathBuf),
}

#[derive(Debug)]
pub struct Parser {
    include_regex: Regex,
}

impl Parser {
    fn new() -> Self {
        Self {
            include_regex: regex::RegexBuilder::new(r#"^\s*#include "(.*)"\s*\r?\n"#)
                .multi_line(true)
                .build()
                .unwrap(),
        }
    }

    fn parse(&self, file: usize, contents: &str, sections: &mut Vec<Section>) {
        sections.clear();
        let mut verbatim_start = 0;
        let mut current_line = 1;
        for captures in self.include_regex.captures_iter(contents) {
            let line = captures.get(0).unwrap();
            let path = captures.get(1).unwrap().as_str();

            // Add verbatim section.
            let verbatim_end = line.start();
            if verbatim_end > verbatim_start {
                let verbatim = &contents[verbatim_start..verbatim_end];
                sections.push(Section::Verbatim(format!(
                    "#line {line} {file}\n{verbatim}",
                    line = current_line,
                    file = file,
                    verbatim = verbatim,
                )));
                current_line += verbatim.lines().count();
            }

            // Add include section.
            sections.push(Section::Include(PathBuf::from(path)));
            current_line += 1;

            // New verbatim starts after the include.
            verbatim_start = line.end();
        }

        let verbatim_end = contents.len();
        if verbatim_end > verbatim_start {
            let verbatim = &contents[verbatim_start..verbatim_end];
            sections.push(Section::Verbatim(format!(
                "#line {line} {file}\n{verbatim}",
                line = current_line,
                file = 999,
                verbatim = verbatim,
            )));
        }
    }
}

#[test]
fn parse_sections() {
    let contents = r##"line 1
line 2
#include "header.glsl"
line 4
"##;
    let parser = Parser::new();

    let mut sections = Vec::new();

    parser.parse(999, &contents, &mut sections);

    assert_eq!(
        vec![
            Section::Verbatim(String::from("#line 1 999\nline 1\nline 2\n")),
            Section::Include(PathBuf::from("header.glsl")),
            Section::Verbatim(String::from("#line 4 999\nline 4\n")),
        ],
        sections
    );
}

impl File {
    pub fn sections<'a>(&'a self, graph: &'a Graph, token: &mut impl Token, index: FileIndex) -> Ref<'a, Vec<Section>> {
        self.sections.verify(&graph.revision, token, |token| {
            let _ = self.signal.read(token);

            token.compute(|value| {
                let contents = std::fs::read_to_string(&self.path).unwrap();
                graph.parser.parse(index, &contents, value);
            });
        })
    }
}

#[derive(Debug)]
pub struct Shader {
    main: FileIndex,
    name: Branch<ShaderName>,
}

fn collect_section<'a>(graph: &'a Graph, token: &mut impl Token, file_index: FileIndex, sources: &mut Vec<Ref<'a, str>>, included: &mut Vec<FileIndex>) {
    let sections = graph.files[file_index].sections(graph, token, file_index);

    // TODO: https://users.rust-lang.org/t/transposition-and-refcell/30430
    for i in 0..sections.len() {
        let section = Ref::map(Ref::clone(&sections), |sections| &sections[i]);
        match *section {
            Section::Verbatim(_) => {
                sources.push(Ref::map(section, |section| {
                    match *section {
                        Section::Verbatim(ref verbatim) => verbatim.as_str(),
                        _ => unreachable!(),
                    }
                }));
            },
            Section::Include(ref path) => {
                let file_index = graph.files.iter().position(|file: &File| &file.path == path).unwrap();
                if included.iter().find(|&&item| item == file_index).is_none() {
                    included.push(file_index);
                    collect_section(graph, token, file_index, sources, included)
                }
            }
        }
    }
}

impl Shader {
    pub fn name<'a>(&'a self, graph: &'a Graph, token: &mut impl Token, gl: &gl::Gl) -> Ref<'a, ShaderName> {
        self.name.verify(&graph.revision, token, |token| {
            let mut sources = Vec::new();
            let mut included = Vec::new();

            collect_section(graph, token, self.main, &mut sources, &mut included);

            token.compute(move |name| {
                name.compile(gl, sources.iter().map(|s| s.as_bytes()));
            });
        })
    }
}

#[derive(Debug)]
pub struct Program {
    shaders: Vec<Shader>,
    name: Branch<ProgramName>,
}

impl Program {
    pub fn name<'a>(&'a self, graph: &'a Graph, token: &mut impl Token, gl: &gl::Gl) -> Ref<'a, ProgramName> {
        self.name.verify(&graph.revision, token, |token| {
            let shader_iter = self.shaders.iter().map(|shader| shader.name(graph, token, gl));
            token.compute(|name| name.link(gl));
        })
    }
}

#[derive(Debug)]
pub struct Graph {
    pub files: Vec<File>,
    pub basic_program: Program,
    pub parser: Parser,
    pub revision: GraphRevision,
}

// pub struct Shader {
//     header: String,
//     source_indices: Vec<usize>,
//     light_space: bool,
//     render_technique: bool,
//     attenuation_mode: bool,
//     name: ShaderName,
// }

// impl Shader {
//     pub fn new(gl: &gl::Gl, kind: impl Into<gl::ShaderKind>, header: String, source_indices: Vec<usize>) -> Self {
//         Self {
//             header,
//             source_indices,
//             light_space: false,
//             render_technique: false,
//             attenuation_mode: false,
//             name: ShaderName::new(gl, kind.into()),
//         }
//     }

//     pub fn update(&mut self, gl: &gl::Gl, world: &mut World) -> ic::Modified {
//         let global = &world.global;
//         if self.branch.verify(global) {
//             let modified = self
//                 .source_indices
//                 .iter()
//                 .map(|&i| world.sources[i].modified)
//                 .chain(
//                     [
//                         (self.light_space, world.light_space.modified),
//                         (self.render_technique, world.render_technique.modified),
//                         (self.attenuation_mode, world.attenuation_mode.modified),
//                     ]
//                     .iter()
//                     .flat_map(
//                         |&(does_depend, modified)| {
//                             if does_depend {
//                                 Some(modified)
//                             } else {
//                                 None
//                             }
//                         },
//                     ),
//                 )
//                 .max()
//                 .unwrap_or(ic::Modified::NONE);

//             if self.branch.recompute(&modified) {
//                 let sources: Vec<[String; 2]> = self
//                     .source_indices
//                     .iter()
//                     .map(|&i| [format!("#line 1 {}\n", i + 1), world.sources[i].read()])
//                     .collect();

//                 self.light_space = sources
//                     .iter()
//                     .any(|[_, source]| world.light_space_regex.is_match(source));

//                 self.render_technique = sources
//                     .iter()
//                     .any(|[_, source]| world.render_technique_regex.is_match(source));

//                 self.attenuation_mode = sources
//                     .iter()
//                     .any(|[_, source]| world.attenuation_mode_regex.is_match(source));

//                 self.name.compile(
//                     gl,
//                     [
//                         COMMON_DECLARATION,
//                         CAMERA_BUFFER_DECLARATION,
//                         crate::light::LIGHT_BUFFER_DECLARATION,
//                         crate::cluster_shading::CLUSTER_BUFFER_DECLARATION,
//                         self.header.as_str(),
//                     ]
//                     .iter()
//                     .copied()
//                     .chain(
//                         [
//                             if self.light_space {
//                                 Some(world.light_space.value.source())
//                             } else {
//                                 None
//                             },
//                             if self.render_technique {
//                                 Some(world.render_technique.value.source())
//                             } else {
//                                 None
//                             },
//                             if self.attenuation_mode {
//                                 Some(world.attenuation_mode.value.source())
//                             } else {
//                                 None
//                             },
//                         ]
//                         .iter()
//                         .flat_map(|&x| x),
//                     )
//                     .chain(sources.iter().flat_map(|x| x.iter().map(|s| s.as_str()))),
//                 );

//                 if self.name.is_uncompiled() {
//                     let log = self.name.log(gl);

//                     let log = world.gl_log_regex.replace_all(&log, |captures: &regex::Captures| {
//                         let i: usize = captures[0].parse().unwrap();
//                         if i > 0 {
//                             let i = i - 1;
//                             let path = world.sources[i].path.strip_prefix(&world.resource_dir).unwrap();
//                             path.display().to_string()
//                         } else {
//                             "<generated header>".to_string()
//                         }
//                     });

//                     error!("Compile error:\n{}", log);
//                 }
//             }
//         }

//         self.branch.modified()
//     }

//     pub fn name<'a>(&'a self, global: &'a ic::Global) -> &'a ShaderName {
//         self.branch.panic_if_outdated(global);
//         &self.name
//     }
// }

// pub struct Program {
//     shaders: Vec<Shader>,
//     branch: ic::Branch,
//     name: ProgramName,
// }

// impl Program {
//     pub fn new(gl: &gl::Gl, shaders: Vec<Shader>) -> Self {
//         let mut program_name = ProgramName::new(gl);

//         program_name.attach(gl, shaders.iter().map(|shader| &shader.name));

//         Self {
//             shaders,
//             branch: ic::Branch::dirty(),
//             name: program_name,
//         }
//     }

//     pub fn modified(&self) -> ic::Modified {
//         self.branch.modified()
//     }

//     pub fn update(&mut self, gl: &gl::Gl, world: &mut World) -> ic::Modified {
//         if self.branch.verify(&world.global) {
//             let modified = self
//                 .shaders
//                 .iter_mut()
//                 .map(|shader| shader.update(gl, world))
//                 .max()
//                 .unwrap_or(self.branch.modified());

//             if self.branch.recompute(&modified) {
//                 self.name.link(gl);

//                 if self.name.is_unlinked()
//                     && self
//                         .shaders
//                         .iter()
//                         .all(|shader| shader.name(&world.global).is_compiled())
//                 {
//                     let log = self.name.log(gl);

//                     let log = world.gl_log_regex.replace_all(&log, |captures: &regex::Captures| {
//                         let i: usize = captures[0].parse().unwrap();
//                         if i > 0 {
//                             let i = i - 1;
//                             let path = world.sources[i].path.strip_prefix(&world.resource_dir).unwrap();
//                             path.display().to_string()
//                         } else {
//                             "<generated header>".to_string()
//                         }
//                     });

//                     error!("Link error:\n{}", log);
//                 }
//             }
//         }

//         self.branch.modified()
//     }

//     pub fn name<'a>(&'a self, global: &'a ic::Global) -> &'a ProgramName {
//         self.branch.panic_if_outdated(global);
//         &self.name
//     }
// }

// /// Utility function to create a very common single file vertex and single file fragment shader.
// pub fn vs_fs_program(gl: &gl::Gl, world: &mut World, vs: &'static str, fs: &'static str) -> Program {
//     Program::new(
//         gl,
//         vec![
//             Shader::new(gl, gl::VERTEX_SHADER, String::new(), vec![world.add_source(vs)]),
//             Shader::new(gl, gl::FRAGMENT_SHADER, String::new(), vec![world.add_source(fs)]),
//         ],
//     )
// }
