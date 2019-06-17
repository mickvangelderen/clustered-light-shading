#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Revision(u64);

impl Revision {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Global(Revision);

impl Global {
    pub fn new() -> Self {
        // Start at one so we can use 0 as dirty.
        Global(Revision(1))
    }

    pub fn mark(&mut self, modified: &mut Modified) {
        self.0.increment();
        modified.0 = self.0;
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Modified(Revision);

impl Modified {
    pub const NONE: Self = Self(Revision(0));
}

#[derive(Debug)]
#[repr(transparent)]
struct Verified(Revision);

#[derive(Debug)]
#[repr(transparent)]
struct Computed(Revision);

pub struct Leaf<T> {
    modified: Modified,
    value: T,
}

impl<T> Leaf<T> {
    #[inline]
    pub fn clean(global: &Global, value: T) -> Self {
        Leaf {
            modified: Modified(global.0),
            value,
        }
    }

    #[inline]
    pub fn modified(&self) -> Modified {
        Modified(self.modified.0)
    }

    #[inline]
    pub fn read(&self) -> &T {
        &self.value
    }
}

impl<T> Leaf<T> {
    #[inline]
    pub fn write<F>(&mut self, global: &mut Global, f: F) where F: FnOnce(&mut T) -> bool {
        if f(&mut self.value) {
            global.mark(&mut self.modified);
        }
    }

    #[inline]
    pub fn write_always<F>(&mut self, global: &mut Global, f: F) where F: FnOnce(&mut T) {
        f(&mut self.value);
        global.mark(&mut self.modified);
    }

    #[inline]
    pub fn replace_always(&mut self, global: &mut Global, value: T) -> T {
        let value = std::mem::replace(&mut self.value, value);
        global.mark(&mut self.modified);
        value
    }
}

impl<T> Leaf<T>
where
    T: PartialEq,
{
    #[inline]
    pub fn replace(&mut self, global: &mut Global, value: T) -> T {
        if self.value != value {
            let value = std::mem::replace(&mut self.value, value);
            global.mark(&mut self.modified);
            value
        } else {
            value
        }
    }
}

pub struct Branch {
    verified: Verified,
    computed: Computed,
}

impl Branch {
    #[inline]
    pub fn dirty() -> Self {
        Self {
            verified: Verified(Revision(0)),
            computed: Computed(Revision(0)),
        }
    }

    #[inline]
    pub fn clean(global: &Global) -> Self {
        Self {
            verified: Verified(global.0),
            computed: Computed(global.0),
        }
    }

    #[inline]
    pub fn panic_if_outdated(&self, global: &Global) {
        if self.verified.0 < global.0 {
            panic_outdated()
        }
    }

    #[inline]
    #[must_use]
    pub fn verify(&mut self, global: &Global) -> bool {
        if self.verified.0 < global.0 {
            self.verified.0 = global.0;
            true
        } else {
            false
        }
    }

    #[inline]
    #[must_use]
    pub fn recompute(&mut self, modified: &Modified) -> bool {
        if self.computed.0 < modified.0 {
            self.computed.0 = modified.0;
            true
        } else {
            false
        }
    }

    #[inline]
    #[must_use]
    pub fn modified(&self) -> Modified {
        Modified(self.computed.0)
    }
}

#[cold]
fn panic_outdated() -> ! {
    panic!("Tried to read outdated value.");
}

#[cfg(test)]
mod tests {
    use super::*;

    type SourceIndex = usize;
    const VERTEX_SOURCE_INDEX: SourceIndex = 0;
    const FRAGMENT_SOURCE_INDEX: SourceIndex = 1;
    const SHARED_SOURCE_INDEX: SourceIndex = 2;

    type Source = Leaf<String>;

    #[derive(Copy, Clone)]
    struct VertexSourceIndices;

    impl AsRef<[SourceIndex]> for VertexSourceIndices {
        fn as_ref(&self) -> &[SourceIndex] {
            &[SHARED_SOURCE_INDEX, VERTEX_SOURCE_INDEX]
        }
    }

    #[derive(Copy, Clone)]
    struct FragmentSourceIndices;

    impl AsRef<[SourceIndex]> for FragmentSourceIndices {
        fn as_ref(&self) -> &[SourceIndex] {
            &[SHARED_SOURCE_INDEX, FRAGMENT_SOURCE_INDEX]
        }
    }

    struct ShaderName<I> {
        source_indices: I,
        branch: Branch,
        contents: String,
    }

    impl<I> ShaderName<I>
    where
        I: AsRef<[SourceIndex]>,
    {
        fn new(source_indices: I) -> Self {
            ShaderName {
                source_indices,
                branch: Branch::dirty(),
                contents: String::new(),
            }
        }

        fn update_contents(&mut self, global: &Global, sources: &mut [Source]) -> Modified {
            if self.branch.verify(global) {
                let modified = &self.source_indices
                    .as_ref()
                    .iter()
                    .map(|&i| sources[i].modified())
                    .max()
                    .unwrap_or(Modified::NONE);

                if self.branch.recompute(modified) {
                    self.contents.clear();
                    for &index in self.source_indices.as_ref() {
                        self.contents.push_str(&sources[index].read());
                    }
                }
            }

            self.branch.modified()
        }

        fn contents<'a>(&'a self, global: &'a Global) -> &'a str {
            self.branch.panic_if_outdated(global);
            &self.contents
        }
    }

    // struct ProgramName<V, S>
    // where
    //     V: AsRef<[SourceIndex]>,
    //     S: AsRef<[SourceIndex]>,
    // {
    //     contents: Branch<String>,
    //     vertex_shader_name: ShaderName<V>,
    //     fragment_shader_name: ShaderName<S>,
    // }

    // impl<V, S> ProgramName<V, S>
    // where
    //     V: AsRef<[SourceIndex]>,
    //     S: AsRef<[SourceIndex]>,
    // {
    //     fn new(vertex_shader_name: ShaderName<V>, fragment_shader_name: ShaderName<S>) -> Self {
    //         ProgramName {
    //             contents: Branch::dirty(String::new()),
    //             vertex_shader_name,
    //             fragment_shader_name,
    //         }
    //     }

    //     fn update_contents(&mut self, global: &Global, sources: &mut [Source]) -> Modified {
    //         let Self { contents, vertex_shader_name, fragment_shader_name } = self;
    //         contents.update(
    //             global,
    //             || {
    //                 let modified1 = vertex_shader_name.update_contents(global, sources);
    //                 let modified2 = fragment_shader_name.update_contents(global, sources);
    //                 return std::cmp::max(modified1, modified2);
    //             },
    //             |contents| {
    //                 contents.clear();
    //                 contents.push_str(vertex_shader_name.contents(global));
    //                 contents.push_str(fragment_shader_name.contents(global));
    //             },
    //         )
    //     }
    // }

    #[test]
    fn verify_shader_works() {
        let global = &mut Global::new();

        let sources = &mut [
            Source::clean(global, "vertex 0\n".to_string()),
            Source::clean(global, "fragment 0\n".to_string()),
            Source::clean(global, "shared 0\n".to_string()),
        ];

        let vs = &mut ShaderName::new(VertexSourceIndices);

        let fs = &mut ShaderName::new(FragmentSourceIndices);

        assert_eq!(Modified(Revision(1)), vs.update_contents(global, sources));
        assert_eq!("shared 0\nvertex 0\n", vs.contents(global));

        assert_eq!(Modified(Revision(1)), fs.update_contents(global, sources));
        assert_eq!("shared 0\nfragment 0\n", fs.contents(global));

        let source = sources[0].replace(global, "vertex 1\n".to_string());

        assert_eq!(Modified(Revision(2)), vs.update_contents(global, sources));
        assert_eq!("shared 0\nvertex 1\n", vs.contents(global));

        assert_eq!(Modified(Revision(1)), fs.update_contents(global, sources));
        assert_eq!("shared 0\nfragment 0\n", fs.contents(global));

        sources[2].value.clear();
        sources[2].value.push_str("shared 1\n");
        global.mark(&mut sources[2].modified);

        assert_eq!(Modified(Revision(3)), vs.update_contents(global, sources));
        assert_eq!("shared 1\nvertex 1\n", vs.contents(global));

        assert_eq!(Modified(Revision(3)), fs.update_contents(global, sources));
        assert_eq!("shared 1\nfragment 0\n", fs.contents(global));
    }

    // #[test]
    // fn verify_program_works() {
    //     let global = &mut Global::new();

    //     let sources = &mut [
    //         Source::new(global, "vertex 0\n".to_string()),
    //         Source::new(global, "fragment 0\n".to_string()),
    //         Source::new(global, "shared 0\n".to_string()),
    //     ];

    //     let pr = &mut global.versioned(ProgramName::new(
    //         global.versioned(ShaderName::new(VertexSourceIndices, sources)),
    //         global.versioned(ShaderName::new(FragmentSourceIndices, sources)),
    //     ));

    //     assert_eq!(Modified(0), pr.update(global, sources));
    //     assert_eq!("shared 0\nvertex 0\nshared 0\nfragment 0\n", &pr.value.contents);

    //     assert_eq!(Modified(0), pr.update(global, sources));
    //     assert_eq!("shared 0\nvertex 0\nshared 0\nfragment 0\n", &pr.value.contents);

    //     sources[0].contents.clear();
    //     sources[0].contents.push_str("vertex 1\n");
    //     sources[0].modified.mark(global);

    //     assert_eq!(Modified(1), pr.update(global, sources));
    //     assert_eq!("shared 0\nvertex 1\nshared 0\nfragment 0\n", &pr.value.contents);

    //     assert_eq!(Modified(1), pr.update(global, sources));
    //     assert_eq!("shared 0\nvertex 1\nshared 0\nfragment 0\n", &pr.value.contents);
    // }
}
