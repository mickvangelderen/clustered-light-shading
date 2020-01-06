use crate::*;

use incremental::{Current, LastComputed, LastModified, LastVerified};
use renderer::configuration::ClusteringProjection;
use renderer::*;

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
    PrefixSum,
    ClusteredLightShading,
    Profiling,
    SampleCount,
    DepthPrepass,
}

impl SourceReader {
    pub fn read(&self, source_index: SourceIndex, vars: &Variables, parser: &Parser, tokens: &mut Tokens) {
        match *self {
            SourceReader::File(ref path) => {
                let source = match std::fs::read_to_string(path) {
                    Ok(source) => source,
                    Err(error) => {
                        error!("Failed to read {:?}: {}", path, error);
                        String::new()
                    }
                };

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
                    AttenuationMode::Reduced => "ATTENUATION_MODE_REDUCED",
                    AttenuationMode::PhyRed1 => "ATTENUATION_MODE_PHY_RED_1",
                    AttenuationMode::PhyRed2 => "ATTENUATION_MODE_PHY_RED_2",
                    AttenuationMode::Smooth => "ATTENUATION_MODE_SMOOTH",
                    AttenuationMode::PhySmo1 => "ATTENUATION_MODE_PHY_SMO_1",
                    AttenuationMode::PhySmo2 => "ATTENUATION_MODE_PHY_SMO_2",
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
            SourceReader::PrefixSum => {
                tokens.push(Token::Literal(format!(
                    "\
                     #line {} {}\n\
                     #define PASS_0_THREADS {}\n\
                     #define PASS_1_THREADS {}\n\
                     ",
                    line!() - 3,
                    source_index,
                    vars.prefix_sum.pass_0_threads,
                    vars.prefix_sum.pass_1_threads,
                )));
            }
            SourceReader::ClusteredLightShading => {
                let clustering_projection = match vars.clustered_light_shading.projection {
                    ClusteringProjection::Orthographic => "CLUSTERING_PROJECTION_ORTHOGRAPHIC",
                    ClusteringProjection::Perspective => "CLUSTERING_PROJECTION_PERSPECTIVE",
                };

                tokens.push(Token::Literal(format!(
                    "\
                     #line {} {}\n\
                     #define CLUSTERING_PROJECTION_ORTHOGRAPHIC 1\n\
                     #define CLUSTERING_PROJECTION_PERSPECTIVE 2\n\
                     #define CLUSTERING_PROJECTION {}\n\
                     \n\
                     #define CLUSTERED_LIGHT_SHADING_MAX_CLUSTERS {}\n\
                     #define CLUSTERED_LIGHT_SHADING_MAX_ACTIVE_CLUSTERS {}\n\
                     #define CLUSTERED_LIGHT_SHADING_MAX_LIGHT_INDICES {}\n\
                     ",
                    line!() - 7,
                    source_index,
                    clustering_projection,
                    vars.clustered_light_shading.max_clusters,
                    vars.clustered_light_shading.max_active_clusters,
                    vars.clustered_light_shading.max_light_indices,
                )));
            }
            SourceReader::Profiling => {
                tokens.push(Token::Literal(format!(
                    "\
                     #line {} {}\n\
                     #define PROFILING_TIME_SENSITIVE {}\n\
                     ",
                    line!() - 2,
                    source_index,
                    match vars.profiling.time_sensitive {
                        true => 1,
                        false => 0,
                    }
                )));
            }
            SourceReader::SampleCount => {
                tokens.push(Token::Literal(format!(
                    "\
                     #line {} {}\n\
                     #define SAMPLE_COUNT {}\n\
                     ",
                    line!() - 2,
                    source_index,
                    vars.sample_count,
                )));
            }
            SourceReader::DepthPrepass => {
                tokens.push(Token::Literal(format!(
                    "\
                    #line {} {}\n\
                    #define DEPTH_PREPASS {}\n\
                    ",
                    line!() - 2,
                    source_index,
                    match vars.depth_prepass {
                        true => 1,
                        false => 0,
                    }
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
        match self.path_to_source_index.get(&source_path) {
            Some(&source_index) => source_index,
            None => {
                let source_index = self.sources.len();
                self.sources.push(Rc::new(source));
                self.path_to_source_index.insert(source_path, source_index);
                source_index
            }
        }
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
    pub fixed_header: String,
    pub last_verified: LastVerified,
    pub last_computed: LastComputed,
    pub contents: String,
    pub included: Vec<SourceIndex>,
}

impl EntryPoint {
    pub fn new(
        context: &mut ShaderCompilationContext,
        relative_path: impl Into<PathBuf>,
        fixed_header: String,
    ) -> Self {
        let relative_path = relative_path.into();
        let absolute_path: PathBuf = [context.resource_dir, relative_path.as_path()].iter().collect();
        let source_index = context.shader_compiler.memory.add_source(
            absolute_path.clone(),
            crate::shader_compiler::Source::new(
                &context.current,
                crate::shader_compiler::SourceReader::File(absolute_path),
                relative_path,
            ),
        );

        EntryPoint {
            source_index,
            fixed_header,
            last_verified: incremental::LastVerified::dirty(),
            last_computed: incremental::LastComputed::dirty(),
            contents: String::new(),
            included: vec![source_index],
        }
    }

    pub fn update(&mut self, context: &mut ShaderCompilationContext) -> bool {
        if self.last_verified.should_verify(&context.current) {
            self.last_verified.update_to(&context.current);
        } else {
            return false;
        }

        let should_recompute = self.included.iter().any(|&source_index| {
            self.last_computed
                .should_compute(&context.shader_compiler.memory.sources[source_index].last_modified)
        });

        return if should_recompute {
            self.contents.clear();
            self.included.clear();

            process(self, context, self.source_index);

            true
        } else {
            false
        };

        fn process(ep: &mut EntryPoint, context: &mut ShaderCompilationContext, source_index: SourceIndex) {
            // Stop processing if we've already included this file.
            if let Presence::Duplicate = vec_set_add(&mut ep.included, source_index) {
                return;
            }

            let source = Rc::get_mut(&mut context.shader_compiler.memory.sources[source_index]).unwrap();
            source.update(
                source_index,
                &context.shader_compiler.variables,
                &context.shader_compiler.parser,
            );

            // Clone the source rc so we can access tokens while mutating the tokens vec.
            let source = Rc::clone(&context.shader_compiler.memory.sources[source_index]);

            ep.last_computed.update_to(&source.last_modified);

            for token in source.tokens.iter() {
                match *token {
                    Token::Literal(ref lit) => {
                        ep.contents.push_str(lit);
                    }
                    Token::Include(ref relative_path) => {
                        let maybe_source_index = if relative_path.starts_with("native/") {
                            Some(
                                context
                                    .shader_compiler
                                    .memory
                                    .source_index(relative_path)
                                    .expect("Unknown native path."),
                            )
                        } else {
                            let parent_path = match source.reader {
                                SourceReader::File(ref path) => path.parent().unwrap(),
                                _ => panic!("Can't include files from native sources."),
                            };
                            match std::fs::canonicalize(parent_path.join(relative_path)) {
                                Ok(absolute_path) => Some(
                                    context
                                        .shader_compiler
                                        .memory
                                        .source_index(&absolute_path)
                                        .unwrap_or_else(|| {
                                            let resource_path =
                                                absolute_path.strip_prefix(&context.resource_dir).unwrap();
                                            let source = Source::new(
                                                &context.current,
                                                SourceReader::File(absolute_path.clone()),
                                                resource_path.to_owned(),
                                            );
                                            context.shader_compiler.memory.add_source(absolute_path, source)
                                        }),
                                ),
                                Err(error) => {
                                    error!(
                                        "Failed to get canonical path of {:?}: {}",
                                        parent_path.join(relative_path),
                                        error
                                    );
                                    None
                                }
                            }
                        };

                        if let Some(source_index) = maybe_source_index {
                            process(ep, context, source_index);
                        }
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
    pub prefix_sum: configuration::PrefixSumConfiguration,
    pub clustered_light_shading: configuration::ClusteredLightShadingConfiguration,
    pub profiling: ProfilingVariables,
    pub sample_count: u32,
    pub depth_prepass: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ProfilingVariables {
    pub time_sensitive: bool,
}

pub struct NativeSourceIndices {
    pub light_space: SourceIndex,
    pub attenuation_mode: SourceIndex,
    pub render_technique: SourceIndex,
    pub prefix_sum: SourceIndex,
    pub clustered_light_shading: SourceIndex,
    pub profiling: SourceIndex,
    pub sample_count: SourceIndex,
    pub depth_prepass: SourceIndex,
}

pub struct ShaderCompilationContext<'a> {
    pub resource_dir: &'a Path,
    pub current: &'a mut incremental::Current,
    pub shader_compiler: &'a mut ShaderCompiler,
}

pub struct ShaderCompiler {
    pub log_regex: Regex,
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
                Source::new(
                    current,
                    SourceReader::LightSpace,
                    PathBuf::from(concat!(file!(), "LIGHT_SPACE")),
                ),
            ),
            attenuation_mode: memory.add_source(
                PathBuf::from("native/ATTENUATION_MODE"),
                Source::new(
                    current,
                    SourceReader::AttenuationMode,
                    PathBuf::from(concat!(file!(), "ATTENUATION_MODE")),
                ),
            ),
            render_technique: memory.add_source(
                PathBuf::from("native/RENDER_TECHNIQUE"),
                Source::new(
                    current,
                    SourceReader::RenderTechnique,
                    PathBuf::from(concat!(file!(), "RENDER_TECHNIQUE")),
                ),
            ),
            prefix_sum: memory.add_source(
                PathBuf::from("native/PREFIX_SUM"),
                Source::new(
                    current,
                    SourceReader::PrefixSum,
                    PathBuf::from(concat!(file!(), "PREFIX_SUM")),
                ),
            ),
            clustered_light_shading: memory.add_source(
                PathBuf::from("native/CLUSTERED_LIGHT_SHADING"),
                Source::new(
                    current,
                    SourceReader::ClusteredLightShading,
                    PathBuf::from(concat!(file!(), "CLUSTERED_LIGHT_SHADING")),
                ),
            ),
            profiling: memory.add_source(
                PathBuf::from("native/PROFILING"),
                Source::new(
                    current,
                    SourceReader::Profiling,
                    PathBuf::from(concat!(file!(), "PROFILING")),
                ),
            ),
            sample_count: memory.add_source(
                PathBuf::from("native/SAMPLE_COUNT"),
                Source::new(
                    current,
                    SourceReader::SampleCount,
                    PathBuf::from(concat!(file!(), "SAMPLE_COUNT")),
                ),
            ),
            depth_prepass: memory.add_source(
                PathBuf::from("native/DEPTH_PREPASS"),
                Source::new(
                    current,
                    SourceReader::DepthPrepass,
                    PathBuf::from(concat!(file!(), "DEPTH_PREPASS")),
                ),
            ),
        };

        Self {
            log_regex: RegexBuilder::new(r"^\d+").multi_line(true).build().unwrap(),
            memory,
            parser,
            variables,
            indices,
        }
    }

    /// Replaces source indices with their paths in an OpenGL error log.
    pub fn process_log(&self, log: &str) -> String {
        self.log_regex
            .replace_all(log, |captures: &regex::Captures| {
                let i: usize = captures[0].parse().unwrap();
                self.memory.sources[i].name.to_str().unwrap()
            })
            .to_string()
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
            self.source_mut(self.indices.light_space).last_modified.modify(current);
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

    pub fn prefix_sum(&self) -> configuration::PrefixSumConfiguration {
        self.variables.prefix_sum
    }

    pub fn replace_prefix_sum(
        &mut self,
        current: &mut Current,
        value: configuration::PrefixSumConfiguration,
    ) -> configuration::PrefixSumConfiguration {
        if self.variables.prefix_sum != value {
            self.source_mut(self.indices.prefix_sum).last_modified.modify(current);
        }
        std::mem::replace(&mut self.variables.prefix_sum, value)
    }

    pub fn replace_clustered_light_shading(
        &mut self,
        current: &mut Current,
        value: configuration::ClusteredLightShadingConfiguration,
    ) -> configuration::ClusteredLightShadingConfiguration {
        if self.variables.clustered_light_shading != value {
            self.source_mut(self.indices.clustered_light_shading)
                .last_modified
                .modify(current);
        }
        std::mem::replace(&mut self.variables.clustered_light_shading, value)
    }

    pub fn replace_profiling(&mut self, current: &mut Current, value: ProfilingVariables) -> ProfilingVariables {
        if self.variables.profiling != value {
            self.source_mut(self.indices.profiling).last_modified.modify(current);
        }
        std::mem::replace(&mut self.variables.profiling, value)
    }

    pub fn replace_sample_count(&mut self, current: &mut Current, value: u32) -> u32 {
        if self.variables.sample_count != value {
            self.source_mut(self.indices.sample_count).last_modified.modify(current);
        }
        std::mem::replace(&mut self.variables.sample_count, value)
    }

    pub fn depth_prepass(&self) -> bool {
        self.variables.depth_prepass
    }

    pub fn replace_depth_prepass(&mut self, current: &mut Current, value: bool) -> bool {
        if self.variables.depth_prepass != value {
            self.source_mut(self.indices.depth_prepass)
                .last_modified
                .modify(current);
        }
        std::mem::replace(&mut self.variables.depth_prepass, value)
    }
}
