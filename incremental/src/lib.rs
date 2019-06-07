#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Current(usize);

impl Current {
    pub fn new() -> Self {
        Current(0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Modified(usize);

impl Modified {
    pub fn new(current: &Current) -> Self {
        Modified(current.0)
    }

    pub fn mark(&mut self, current: &mut Current) {
        current.0 += 1;
        self.0 = current.0;
    }

    pub fn after(&self, verified: &Verified) -> bool {
        self.0 > verified.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Verified(usize);

impl Verified {
    pub fn new(current: &Current) -> Self {
        Verified(current.0)
    }

    pub fn before(&self, current: &Current) -> bool {
        self.0 < current.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type SourceIndex = usize;
    const VERTEX_SOURCE_INDEX: SourceIndex = 0;
    const FRAGMENT_SOURCE_INDEX: SourceIndex = 1;
    const SHARED_SOURCE_INDEX: SourceIndex = 2;

    struct Source {
        contents: String,
        modified: Modified,
    }

    impl Source {
        fn new(current: &Current, contents: String) -> Self {
            Source {
                contents,
                modified: Modified::new(current),
            }
        }
    }

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
        contents: String,
        source_indices: I,
        verified: Verified,
        modified: Modified,
    }

    impl<I> ShaderName<I>
    where
        I: AsRef<[SourceIndex]>,
    {
        fn new(current: &Current, source_indices: I, sources: &[Source]) -> Self {
            ShaderName {
                contents: {
                    let mut contents = String::new();
                    Self::compute(&mut contents, &source_indices, sources);
                    contents
                },
                source_indices,
                verified: Verified::new(current),
                modified: Modified::new(current),
            }
        }

        fn compute(contents: &mut String, source_indices: &I, sources: &[Source]) {
            for &index in source_indices.as_ref() {
                contents.push_str(&sources[index].contents);
            }
        }

        fn update(&mut self, current: &Current, sources: &mut [Source]) -> Modified {
            if self.verified.before(current) {
                // Verify dependencies.
                for source in self.source_indices.as_ref().iter().map(|&i| &sources[i]) {
                    if source.modified > self.modified {
                        self.modified = source.modified;
                    }
                }

                if self.modified.after(&self.verified) {
                    self.contents.clear();
                    Self::compute(&mut self.contents, &self.source_indices, sources);
                }

                self.verified.0 = current.0;
            }

            self.modified
        }
    }

    #[test]
    fn verify_shader_works() {
        let current = &mut Current::new();

        let sources = &mut [
            Source::new(current, "vertex 0\n".to_string()),
            Source::new(current, "fragment 0\n".to_string()),
            Source::new(current, "shared 0\n".to_string()),
        ];

        let vs = &mut ShaderName::new(
            current,
            VertexSourceIndices,
            sources,
        );

        let fs = &mut ShaderName::new(
            current,
            FragmentSourceIndices,
            sources,
        );

        assert_eq!(Modified(0), vs.update(current, sources));
        assert_eq!("shared 0\nvertex 0\n", &vs.contents);

        assert_eq!(Modified(0), fs.update(current, sources));
        assert_eq!("shared 0\nfragment 0\n", &fs.contents);

        sources[0].contents.clear();
        sources[0].contents.push_str("vertex 1\n");
        sources[0].modified.mark(current);

        assert_eq!(Modified(1), vs.update(current, sources));
        assert_eq!("shared 0\nvertex 1\n", &vs.contents);

        assert_eq!(Modified(0), fs.update(current, sources));
        assert_eq!("shared 0\nfragment 0\n", &fs.contents);

        sources[2].contents.clear();
        sources[2].contents.push_str("shared 1\n");
        sources[2].modified.mark(current);

        assert_eq!(Modified(2), vs.update(current, sources));
        assert_eq!("shared 1\nvertex 1\n", &vs.contents);

        assert_eq!(Modified(2), fs.update(current, sources));
        assert_eq!("shared 1\nfragment 0\n", &fs.contents);
    }
}
