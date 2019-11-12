pub struct Branch(usize);

pub struct Leaf(usize);

pub struct Node(usize);

impl From<Branch> for Node {
    fn from(branch: Branch) -> Self {
        Self(branch.0)
    }
}

impl From<Leaf> for Node {
    fn from(leaf: Leaf) -> Self {
        Self(leaf.0)
    }
}

pub struct Graph {
    revision: u64,
    modified_or_verified: Vec<u64>,
    computed: Vec<u64>,
}

impl Graph {
    // Leaf nodes.
    pub fn modify(&mut self, leaf: Leaf) {
        self.revision += 1;
        self.modified_or_verified[leaf.0] = self.revision;
    }

    pub fn should_recompute_leaf(&mut self, leaf: Leaf) {
        self.computed[leaf.0] < self.modified_or_verified[leaf.0]
    }

    pub fn recompute_leaf(&mut self, leaf: Leaf) {
        self.computed[leaf.0] = self.modified_or_verified[leaf.0]
    }

    // Parent nodes.

    pub fn should_verify(&self, branch: Branch) -> bool {
        self.modified_or_verified[branch.0] < self.revision
    }

    pub fn verify(&mut self, branch: Branch) {
        self.modified_or_verified[branch.0] = self.revision;
    }

    pub fn should_recompute_branch<I>(&self, branch: Branch, dependencies: I) -> bool
    where
        I: IntoIterator<Item = Node>,
    {
        dependencies
            .into_iter()
            .any(|dependency_index| self.computed[dependency_index] > self.computed[branch.0])
    }

    pub fn recompute_branch<I>(&mut self, branch: Branch, dependencies: I)
    where
        I: IntoIterator<Item = Node>,
    {
        for dependency_index in dependencies.into_iter() {
            if self.computed[branch.0] < self.computed[dependency_index] {
                self.computed[branch.0] = self.computed[dependency_index];
            }
        }
    }
}

use std::collections::HashMap;
use std::path::PathBuf;

pub type SourceIndex = usize;

pub enum Token {
    Include(SourceIndex),
    Literal(String),
}

pub struct Source {
    pub file_path: PathBuf,
    pub node: Leaf,
    pub tokens: Vec<Token>,
}

pub struct EntryPoint {
    pub source_index: SourceIndex,
    pub node: Branch,
    pub dependencies: Vec<Node>,
    pub fixed_header: String,
    pub contents: String,
}

impl EntryPoint {
    pub fn update(&mut self) {}
}

pub struct Compiler {
    pub graph: Graph,
    pub sources: Vec<Source>,
    pub entry_points: Vec<EntryPoint>,
}

impl Compiler {
    pub fn modify_file(&mut self, file_path: &Path) {
        for source in self.sources.iter() {
            if &source.file_path == file_path {
                self.graph.modify(source.file_node);
            }
        }
    }

    pub fn update_entry_point(&mut self, entry_point_index: usize) {
        let ep = &mut self.entry_points[entry_point_index];

        let original_modified = graph.modified[ep.node_index];

        if graph.should_verify(ep.node_index) {
            if graph.should_recompute(ep.node_index) {
                let mut dependencies = graph.begin_recompute(ep.node_index);
                dependencies.clear();

                self.process_source(&mut dependencies, ep.source_index);

                graph.end_recompute(ep.node_index, dependencies);
            }

            graph.verify(ep.node_index);
        }

        graph.modified[ep.node_index] != original_modified
    }
}
