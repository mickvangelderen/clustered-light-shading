use crate::*;

use ::incremental::{Branch, Graph as GraphRevision, Leaf, RootToken, Token};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::io::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

pub type FileIndex = usize;

#[derive(Debug)]
pub struct File {
    path: PathBuf,
    signal: Leaf<()>,
    sections: Branch<Vec<Section>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Section {
    Verbatim(String),
    Include(SourceIndex),
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

    fn parse(&self, graph: &Graph, file_index: usize, contents: &str, sections: &mut Vec<Section>) {
        sections.clear();
        let mut verbatim_start = 0;
        let mut current_line = 1;
        for captures in self.include_regex.captures_iter(contents) {
            let line = captures.get(0).unwrap();

            // Add verbatim section.
            let verbatim_end = line.start();
            if verbatim_end > verbatim_start {
                let verbatim = &contents[verbatim_start..verbatim_end];
                sections.push(Section::Verbatim(format!(
                    "#line {line} {file}\n{verbatim}",
                    line = current_line,
                    file = file_index + 100,
                    verbatim = verbatim,
                )));
                current_line += verbatim.lines().count();
            }

            let file_index = {
                let mut files = graph.files.borrow_mut();
                let mut path_to_index = graph.path_to_index.borrow_mut();

                // Obtain actual path.
                let relative_path = Path::new(captures.get(1).unwrap().as_str());
                debug_assert!(relative_path.is_relative());

                let path = std::fs::canonicalize(
                    files[file_index]
                        .path
                        .parent()
                        .expect("Path must have a parent.")
                        .join(relative_path),
                )
                .unwrap();

                // Ensure path has an index.
                match path_to_index.get(&path) {
                    Some(&file_index) => file_index,
                    None => {
                        let file_index = files.len();
                        files.push(Rc::new(File {
                            path: path.clone(),
                            signal: graph.revision.leaf(()),
                            sections: graph.revision.branch(Vec::new()),
                        }));
                        path_to_index.insert(path, file_index);
                        file_index
                    }
                }
            };

            // Add include section.
            sections.push(Section::Include(SourceIndex::File(file_index)));
            current_line += 1;

            // New verbatim starts after the include.
            verbatim_start = line.end();
        }

        let verbatim_end = contents.len();
        if verbatim_end > verbatim_start {
            let verbatim = &contents[verbatim_start..verbatim_end];
            sections.push(Section::Verbatim(format!(
                "#line {line} {file_index}\n{verbatim}",
                line = current_line,
                file_index = 999,
                verbatim = verbatim,
            )));
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SourceIndex {
    File(FileIndex),
    AttenuationMode,
    RenderTechnique,
}

#[derive(Debug)]
pub struct Shader {
    main: FileIndex,
    name: Branch<ShaderName>,
}

fn collect_section<'a>(
    graph: &'a Graph,
    token: &mut impl Token,
    file_index: FileIndex,
    sources: &mut Vec<Ref<'a, str>>,
    included: &mut Vec<SourceIndex>,
) {
    let sections = graph.sections(token, file_index);

    // TODO: https://users.rust-lang.org/t/transposition-and-refcell/30430
    for i in 0..sections.len() {
        let section = Ref::map(Ref::clone(&sections), |sections| &sections[i]);
        match *section {
            Section::Verbatim(_) => {
                sources.push(Ref::map(section, |section| match *section {
                    Section::Verbatim(ref verbatim) => verbatim.as_str(),
                    _ => unreachable!(),
                }));
            }
            Section::Include(source_index) => {
                if included.iter().find(|&&item| item == source_index).is_none() {
                    included.push(source_index);
                    match source_index {
                        SourceIndex::File(file_index) => collect_section(graph, token, file_index, sources, included),
                        _ => unimplemented!(),
                    }
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
    pub path_to_index: RefCell<HashMap<PathBuf, FileIndex>>,
    // Need RefCell and Rc to allow pushing new elements onto the Vec while we update elements.
    pub files: RefCell<Vec<Rc<File>>>,
    pub parser: Parser,
    pub revision: GraphRevision,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            path_to_index: RefCell::new(HashMap::new()),
            files: RefCell::new(Vec::new()),
            parser: Parser::new(),
            revision: GraphRevision::new(),
        }
    }
}

impl Graph {
    pub fn sections<'a>(&'a self, token: &mut impl Token, file_index: FileIndex) -> Ref<'a, Vec<Section>> {
        let file = {
            let files = self.files.borrow();
            Rc::clone(&files[file_index])
        };

        file.sections.verify(&self.revision, token, move |token| {
            let _ = file.signal.read(token);

            token.compute(|sections| {
                let contents = std::fs::read_to_string(&file.path).unwrap();
                self.parser.parse(self, file_index, &contents, sections);
            });
        })
    }
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
