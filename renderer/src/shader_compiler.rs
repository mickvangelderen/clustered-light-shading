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
    LightSpace,
    AttenuationMode,
    RenderTechnique,
}

impl SourceReader {
    pub fn read(&self, source_index: SourceIndex, vars: &Variables, parser: &Parser, tokens: &mut Tokens) {
        match *self {
            SourceReader::File(ref path) => {
                let source = std::fs::read_to_string(path).unwrap();
                parser.parse(&source, source_index, tokens);
            }
            SourceReader::LightSpace => {
                let define = match vars.light_space {
                    LightSpace::Wld => "LIGHT_SPACE_WLD",
                    LightSpace::Cam => "LIGHT_SPACE_CAM",
                    LightSpace::Hmd => "LIGHT_SPACE_HMD",
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
                let define = match vars.render_technique {
                    RenderTechnique::Clustered => "RENDER_TECHNIQUE_CLUSTERED",
                    RenderTechnique::Naive => "RENDER_TECHNIQUE_NAIVE",
                    RenderTechnique::Tiled => "RENDER_TECHNIQUE_TILED",
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

    pub fn update(&mut self, source_index: SourceIndex, vars: &Variables, parser: &Parser) {
        if self.last_computed.should_compute(&self.last_modified) {
            self.last_computed.update_to(&self.last_modified);
            self.tokens.clear();
            self.reader.read(source_index, vars, parser, &mut self.tokens);
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

    pub fn update(&mut self, world: &mut World) -> bool {
        if self.last_verified.should_verify(&world.current) {
            self.last_verified.update_to(&world.current);
        } else {
            return false;
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

        return if should_recompute {
            self.contents.clear();
            self.included.clear();

            process(self, world, self.source_index);

            true
        } else {
            false
        };

        fn process(ep: &mut EntryPoint, world: &mut World, source_index: SourceIndex) {
            // Stop processing if we've already included this file.
            if let Presence::Duplicate = vec_set_add(&mut ep.included, source_index) {
                return;
            }

            let source = Rc::get_mut(&mut world.shader_compiler.memory.sources[source_index]).unwrap();
            source.update(source_index, &world.shader_compiler.variables, &world.shader_compiler.parser);

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

pub struct Variables {
    pub light_space: LightSpace,
    pub attenuation_mode: AttenuationMode,
    pub render_technique: RenderTechnique,
}

pub struct NativeSourceIndices {
    pub light_space: SourceIndex,
    pub attenuation_mode: SourceIndex,
    pub render_technique: SourceIndex,
}

pub struct ShaderCompiler {
    pub memory: Memory,
    pub parser: Parser,
    pub variables: Variables,
    pub indices: NativeSourceIndices,
}

impl ShaderCompiler {
    pub fn new(current: &Current, variables: Variables) -> Self {
        let parser = Parser::new();
        let mut memory = Memory::new();
        let indices = NativeSourceIndices {
            light_space: memory.add_source(
                PathBuf::from("native/LIGHT_SPACE"),
                Source::new(current, SourceReader::LightSpace, PathBuf::from(file!())),
            ),
            attenuation_mode: memory.add_source(
                PathBuf::from("native/ATTENUATION_MODE"),
                Source::new(current, SourceReader::AttenuationMode, PathBuf::from(file!())),
            ),
            render_technique: memory.add_source(
                PathBuf::from("native/RENDER_TECHNIQUE"),
                Source::new(current, SourceReader::RenderTechnique, PathBuf::from(file!())),
            ),
        };

        Self {
            memory,
            parser,
            variables,
            indices,
        }
    }

    pub fn source_mut(&mut self, source_index: SourceIndex) -> &mut Source {
        Rc::get_mut(&mut self.memory.sources[source_index]).unwrap()
    }

    pub fn light_space(&self) -> LightSpace {
        self.variables.light_space
    }

    pub fn replace_light_space(&mut self, current: &mut Current, light_space: LightSpace) -> LightSpace {
        let old = std::mem::replace(&mut self.variables.light_space, light_space);
        if old != light_space {
            self.source_mut(self.indices.light_space)
                .last_modified
                .modify(current);
        }
        old
    }

    pub fn attenuation_mode(&self) -> AttenuationMode {
        self.variables.attenuation_mode
    }

    pub fn replace_attenuation_mode(
        &mut self,
        current: &mut Current,
        attenuation_mode: AttenuationMode,
    ) -> AttenuationMode {
        let old = std::mem::replace(&mut self.variables.attenuation_mode, attenuation_mode);
        if old != attenuation_mode {
            self.source_mut(self.indices.attenuation_mode)
                .last_modified
                .modify(current);
        }
        old
    }

    pub fn render_technique(&self) -> RenderTechnique {
        self.variables.render_technique
    }

    pub fn replace_render_technique(
        &mut self,
        current: &mut Current,
        render_technique: RenderTechnique,
    ) -> RenderTechnique {
        let old = std::mem::replace(&mut self.variables.render_technique, render_technique);
        if old != render_technique {
            self.source_mut(self.indices.render_technique)
                .last_modified
                .modify(current);
        }
        old
    }
}
