use crate::*;

use incremental::{Current, LastComputed, LastModified, LastVerified};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub type SourceIndex = usize;

#[derive(Debug, Clone)]
pub enum Token {
    Literal(String),
    Include(PathBuf),
}

pub type Tokens = Vec<Token>;

#[derive(Debug)]
pub struct Parser {
    include_regex: Regex,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            include_regex: regex::RegexBuilder::new(r#"^[ \t]*#include "(.*)"[ \t]*\r?\n"#)
                .multi_line(true)
                .build()
                .unwrap(),
        }
    }

    pub fn parse(&self, source: &str, source_index: SourceIndex, tokens: &mut Vec<Token>) {
        tokens.clear();
        let mut literal_start = 0;
        let mut current_line = 1;
        for captures in self.include_regex.captures_iter(source) {
            let line = captures.get(0).unwrap();

            // Add literal section.
            let literal_end = line.start();
            if literal_end > literal_start {
                let literal = &source[literal_start..literal_end];
                tokens.push(Token::Literal(format!(
                    "#line {line} {file}\n{literal}",
                    line = current_line,
                    file = source_index,
                    literal = literal,
                )));
                current_line += literal.lines().count();
            }

            // Obtain actual path.
            let relative_path = PathBuf::from(captures.get(1).unwrap().as_str());
            debug_assert!(relative_path.is_relative());

            // Add include section.
            tokens.push(Token::Include(relative_path));
            current_line += 1;

            // New literal starts after the include.
            literal_start = line.end();
        }

        let literal_end = source.len();
        if literal_end > literal_start {
            let literal = &source[literal_start..literal_end];
            tokens.push(Token::Literal(format!(
                "#line {line} {file}\n{literal}",
                line = current_line,
                file = source_index,
                literal = literal,
            )));
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SourceReader {
    File(PathBuf),
    AttenuationMode,
    RenderTechnique,
}

impl SourceReader {
    pub fn read(&self, source_index: SourceIndex, vars: &ShaderVariables, parser: &Parser, tokens: &mut Tokens) {
        match *self {
            SourceReader::File(ref path) => {
                let source = std::fs::read_to_string(path).unwrap();
                parser.parse(&source, source_index, tokens);
            }
            SourceReader::AttenuationMode => {
                let define = match vars.attenuation_mode {
                    AttenuationMode::Step => "ATTENUATION_MODE_STEP",
                    AttenuationMode::Linear => "ATTENUATION_MODE_LINEAR",
                    AttenuationMode::Physical => "ATTENUATION_MODE_PHYSICAL",
                    AttenuationMode::Interpolated => "ATTENUATION_MODE_INTERPOLATED",
                    AttenuationMode::Reduced => "ATTENUATION_MODE_REDUCED",
                    AttenuationMode::Smooth => "ATTENUATION_MODE_SMOOTH",
                };

                tokens.push(Token::Literal(format!(
                    "\
                     #line {line} {source_index}\n\
                     #define {define}\n\
                     ",
                    line = line!() - 2,
                    source_index = source_index,
                    define = define,
                )));
            }
            SourceReader::RenderTechnique => {
                *tokens = vec![Token::Literal(format!(
                    "#define RENDER_TECHNIQUE {}\n",
                    // TODO
                    999
                ))];
            }
        }
    }
}

#[derive(Debug)]
pub struct Source {
    pub reader: SourceReader,
    pub name: PathBuf,
    pub last_modified: LastModified,
    pub last_computed: LastComputed,
    pub tokens: Tokens,
}

impl Source {
    pub fn new(current: &Current, reader: SourceReader, name: PathBuf) -> Self {
        Self {
            reader,
            name,
            last_modified: LastModified::new(current),
            last_computed: LastComputed::dirty(),
            tokens: Vec::new(),
        }
    }

    pub fn update(&mut self, source_index: SourceIndex, vars: &ShaderVariables, parser: &Parser) {
        if self.last_computed.should_compute(&self.last_modified) {
            self.last_computed.update_to(&self.last_modified);
            self.reader.read(source_index, vars, parser, &mut self.tokens);
            println!("Updated {:?}.", self);
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    pub path_to_source_index: HashMap<PathBuf, SourceIndex>,
    pub sources: Vec<Rc<Source>>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            path_to_source_index: HashMap::new(),
            sources: Vec::new(),
        }
    }

    pub fn source_index(&mut self, path: impl AsRef<Path>) -> Option<SourceIndex> {
        self.path_to_source_index.get(path.as_ref()).copied()
    }

    pub fn add_source(&mut self, source_path: PathBuf, source: Source) -> SourceIndex {
        let source_index = self.sources.len();
        self.sources.push(Rc::new(source));
        self.path_to_source_index.insert(source_path, source_index);
        source_index
    }
}

enum Presence {
    Unique,
    Duplicate,
}

fn vec_set_add<T: Copy + PartialEq>(vec: &mut Vec<T>, val: T) -> Presence {
    if vec.iter().find(|&&x| x == val).is_some() {
        Presence::Duplicate
    } else {
        vec.push(val);
        Presence::Unique
    }
}

#[derive(Debug)]
pub struct EntryPoint {
    pub source_index: SourceIndex,
    pub last_verified: LastVerified,
    pub last_computed: LastComputed,
    pub contents: String,
    pub included: Vec<SourceIndex>,
}

impl EntryPoint {
    pub fn new(world: &mut World, relative_path: impl Into<PathBuf>) -> Self {
        let relative_path = relative_path.into();
        let absolute_path: PathBuf = [world.resource_dir.as_path(), relative_path.as_path()].iter().collect();
        let source_index = world.shader_compiler.memory.add_source(
            absolute_path.clone(),
            crate::shader_compiler::Source::new(
                &world.current,
                crate::shader_compiler::SourceReader::File(absolute_path),
                relative_path,
            ),
        );

        EntryPoint {
            source_index,
            last_verified: incremental::LastVerified::dirty(),
            last_computed: incremental::LastComputed::dirty(),
            contents: String::new(),
            included: vec![source_index],
        }
    }

    pub fn update(&mut self, world: &mut World) {
        if self.last_verified.should_verify(&world.current) {
            self.last_verified.update_to(&world.current);
        } else {
            return;
        }

        let mut should_recompute = false;

        {
            let mem = &mut world.shader_compiler.memory;
            for &source_index in self.included.iter() {
                let source = &mem.sources[source_index];
                if self.last_computed.should_compute(&source.last_modified) {
                    should_recompute = true;
                    break;
                }
            }
        }

        if should_recompute {
            self.contents.clear();
            self.included.clear();

            process(self, world, self.source_index);

            println!("Updated {:?}.", self);
        }

        fn process(ep: &mut EntryPoint, world: &mut World, source_index: SourceIndex) {
            // Stop processing if we've already included this file.
            if let Presence::Duplicate = vec_set_add(&mut ep.included, source_index) {
                return;
            }

            let source = Rc::get_mut(&mut world.shader_compiler.memory.sources[source_index]).unwrap();
            source.update(source_index, &world.shader_variables, &world.shader_compiler.parser);

            // Clone the source rc so we can access tokens while mutating the tokens vec.
            let source = Rc::clone(&world.shader_compiler.memory.sources[source_index]);

            ep.last_computed.update_to(&source.last_modified);

            for token in source.tokens.iter() {
                match *token {
                    Token::Literal(ref lit) => {
                        ep.contents.push_str(lit);
                    }
                    Token::Include(ref relative_path) => {
                        let source_index = if relative_path.starts_with("native/") {
                            world
                                .shader_compiler
                                .memory
                                .source_index(relative_path)
                                .expect("Unknown native path.")
                        } else {
                            let parent_path = match source.reader {
                                SourceReader::File(ref path) => path.parent().unwrap(),
                                _ => panic!("Can't include files from native sources."),
                            };
                            let absolute_path = std::fs::canonicalize(parent_path.join(relative_path)).unwrap();

                            world
                                .shader_compiler
                                .memory
                                .source_index(&absolute_path)
                                .unwrap_or_else(|| {
                                    let resource_path = absolute_path.strip_prefix(&world.resource_dir).unwrap();
                                    let source = Source::new(
                                        &world.current,
                                        SourceReader::File(absolute_path.clone()),
                                        resource_path.to_owned(),
                                    );
                                    world.shader_compiler.memory.add_source(absolute_path, source)
                                })
                        };

                        process(ep, world, source_index);
                    }
                }
            }
        }
    }
}

// let attenuation_mode_path = PathBuf::from("intrinsic/ATTENUATION_MODE.glsl");
// let render_technique_path = PathBuf::from("intrinsic/RENDER_TECHNIQUE.glsl");

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

pub struct ShaderVariables {
    pub attenuation_mode: AttenuationMode,
    pub render_technique: RenderTechnique,
}

pub struct ShaderCompiler {
    pub memory: Memory,
    pub parser: Parser,
    pub attenuation_mode_index: SourceIndex,
    pub render_technique_index: SourceIndex,
}

impl ShaderCompiler {
    pub fn new(current: &Current) -> Self {
        let parser = Parser::new();
        let mut memory = Memory::new();

        let attenuation_mode_index = memory.add_source(
            PathBuf::from("native/ATTENUATION_MODE"),
            Source::new(current, SourceReader::AttenuationMode, PathBuf::from(file!())),
        );

        let render_technique_index = memory.add_source(
            PathBuf::from("native/RENDER_TECHNIQUE"),
            Source::new(current, SourceReader::RenderTechnique, PathBuf::from(file!())),
        );

        Self {
            memory,
            parser,
            attenuation_mode_index,
            render_technique_index,
        }
    }
}
