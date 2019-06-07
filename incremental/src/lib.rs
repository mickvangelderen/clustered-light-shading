#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Current(usize);

impl Current {
    pub fn new() -> Self {
        Current(0)
    }

    pub fn versioned<T>(&self, value: T) -> Versioned<T>
    where
        T: Versionable,
    {
        Versioned {
            value,
            verified: Verified(self.0),
            modified: Modified(self.0),
        }
    }

    pub fn compute<'a, T>(&'a self, versioned: &'a mut Versioned<T>, environment: &mut T::Environment) -> Ref<'a, T> where T: Versionable {
        if versioned.verified.0 < self.0 {
            // compute dependencies.

            // compute self if dependencies have been modified since last time.

        }
        Ref::new(self, &versioned.value)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Modified(usize);

impl Modified {
    pub fn new(current: &Current) -> Self {
        Modified(current.0)
    }

    pub fn never() -> Self {
        // TODO: The value 0 is fine since verified cannot be less than 0?
        Modified(0)
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

pub trait Versionable {
    type Environment: ?Sized;

    fn update_dependencies(&mut self, current: &Current, environment: &mut Self::Environment) -> Modified;

    fn update_self(&mut self, environment: &mut Self::Environment);
}

// TODO: Rename to derived?
pub struct Versioned<T>
where
    T: Versionable,
{
    value: T,
    verified: Verified,
    modified: Modified,
}

/// Prevents Current from being used for modification while this reference exists.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
#[must_use]
pub struct Ref<'a, T: ?Sized + 'a> {
    value: &'a T,
    _current: ::std::marker::PhantomData<&'a Current>,
}

impl<T: ?Sized> ::std::ops::Deref for Ref<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<'a, T: ?Sized + 'a> Ref<'a, T> {
    fn new(current: &'a Current, value: &'a T) -> Self {
        Ref {
            value,
            _current: ::std::marker::PhantomData,
        }
    }
}

/// Prevents Current from being used for verification while this reference exists.
#[derive(Debug)]
#[repr(transparent)]
#[must_use]
pub struct Mut<'a, T: ?Sized + 'a> {
    value: &'a mut T,
    _current: ::std::marker::PhantomData<&'a mut Current>,
}

impl<'a, T: ?Sized + 'a> Mut<'a, T> {
    fn new(current: &'a mut Current, value: &'a mut T) -> Self {
        Mut {
            value,
            _current: ::std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized> ::std::ops::Deref for Mut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<T: ?Sized> ::std::ops::DerefMut for Mut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}

pub struct Leaf<T> {
    value: T,
    modified: Modified,
}

impl<T> Leaf<T> {
    pub fn get_mut<'a>(&'a mut self, current: &'a mut Current) -> Mut<'a, T> {
        Mut::new(current, &mut self.value)
    }
}

impl<T> Versioned<T>
where
    T: Versionable,
{
    pub fn update(&mut self, current: &Current, environment: &mut T::Environment) -> Modified {
        if self.verified.0 < current.0 {
            let modified = self.value.update_dependencies(current, environment);

            if modified > self.modified {
                self.value.update_self(environment);
                self.modified.0 = modified.0;
            }

            self.verified.0 = current.0;
        }

        self.modified
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
    }

    impl Source {
        fn new(contents: String) -> Self {
            Source {
                contents,
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
    }

    impl<I> ShaderName<I>
    where
        I: AsRef<[SourceIndex]>,
    {
        fn new(source_indices: I, sources: &[Source]) -> Self {
            ShaderName {
                contents: {
                    let mut contents = String::new();
                    Self::compute(&mut contents, &source_indices, sources);
                    contents
                },
                source_indices,
            }
        }

        fn compute(contents: &mut String, source_indices: &I, sources: &[Source]) {
            for &index in source_indices.as_ref() {
                contents.push_str(&sources[index].contents);
            }
        }
    }

    impl<I> Versionable for ShaderName<I>
    where
        I: AsRef<[SourceIndex]>,
    {
        type Environment = [Source];

        fn update_dependencies(&mut self, _current: &Current, sources: &mut [Source]) -> Modified {
            self.source_indices
                .as_ref()
                .iter()
                .map(|&i| sources[i].modified)
                .max()
                .unwrap()
        }

        fn update_self(&mut self, sources: &mut [Source]) {
            self.contents.clear();
            ShaderName::compute(&mut self.contents, &self.source_indices, sources);
        }
    }

    struct ProgramName<V, S>
    where
        V: AsRef<[SourceIndex]>,
        S: AsRef<[SourceIndex]>,
    {
        contents: String,
        vertex_shader_name: Versioned<ShaderName<V>>,
        fragment_shader_name: Versioned<ShaderName<S>>,
    }

    impl<V, S> ProgramName<V, S>
    where
        V: AsRef<[SourceIndex]>,
        S: AsRef<[SourceIndex]>,
    {
        fn new(
            vertex_shader_name: Versioned<ShaderName<V>>,
            fragment_shader_name: Versioned<ShaderName<S>>,
        ) -> Self {
            ProgramName {
                contents: {
                    let mut contents = String::new();
                    Self::compute(&mut contents, &vertex_shader_name.value, &fragment_shader_name.value);
                    contents
                },
                vertex_shader_name,
                fragment_shader_name,
            }
        }

        fn compute(string: &mut String, vertex_shader_name: &ShaderName<V>, fragment_shader_name: &ShaderName<S>) {
            string.push_str(&vertex_shader_name.contents);
            string.push_str(&fragment_shader_name.contents);
        }
    }

    impl<V, S> Versionable for ProgramName<V, S>
    where
        V: AsRef<[SourceIndex]>,
        S: AsRef<[SourceIndex]>,
    {
        type Environment = [Source];

        fn update_dependencies(&mut self, current: &Current, sources: &mut [Source]) -> Modified {
            let modified1 = self.vertex_shader_name.update(current, sources);
            let modified2 = self.fragment_shader_name.update(current, sources);
            return std::cmp::max(modified1, modified2);
        }

        fn update_self(&mut self, _sources: &mut [Source]) {
            self.contents.clear();
            ProgramName::compute(
                &mut self.contents,
                &self.vertex_shader_name.value,
                &self.fragment_shader_name.value,
            );
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

        let vs = &mut current.versioned(ShaderName::new(VertexSourceIndices, sources));

        let fs = &mut current.versioned(ShaderName::new(FragmentSourceIndices, sources));

        assert_eq!(Modified(0), vs.update(current, sources));
        assert_eq!("shared 0\nvertex 0\n", &vs.value.contents);

        assert_eq!(Modified(0), fs.update(current, sources));
        assert_eq!("shared 0\nfragment 0\n", &fs.value.contents);

        let source = sources[0].get_mut()
        sources[0].contents.clear();
        sources[0].contents.push_str("vertex 1\n");
        sources[0].modified.mark(current);

        assert_eq!(Modified(1), vs.update(current, sources));
        assert_eq!("shared 0\nvertex 1\n", &vs.value.contents);

        assert_eq!(Modified(0), fs.update(current, sources));
        assert_eq!("shared 0\nfragment 0\n", &fs.value.contents);

        sources[2].contents.clear();
        sources[2].contents.push_str("shared 1\n");
        sources[2].modified.mark(current);

        assert_eq!(Modified(2), vs.update(current, sources));
        assert_eq!("shared 1\nvertex 1\n", &vs.value.contents);

        assert_eq!(Modified(2), fs.update(current, sources));
        assert_eq!("shared 1\nfragment 0\n", &fs.value.contents);
    }

    #[test]
    fn verify_program_works() {
        let current = &mut Current::new();

        let sources = &mut [
            Source::new(current, "vertex 0\n".to_string()),
            Source::new(current, "fragment 0\n".to_string()),
            Source::new(current, "shared 0\n".to_string()),
        ];

        let pr = &mut current.versioned(ProgramName::new(
            current.versioned(ShaderName::new(VertexSourceIndices, sources)),
            current.versioned(ShaderName::new(FragmentSourceIndices, sources)),
        ));

        assert_eq!(Modified(0), pr.update(current, sources));
        assert_eq!("shared 0\nvertex 0\nshared 0\nfragment 0\n", &pr.value.contents);

        assert_eq!(Modified(0), pr.update(current, sources));
        assert_eq!("shared 0\nvertex 0\nshared 0\nfragment 0\n", &pr.value.contents);

        sources[0].contents.clear();
        sources[0].contents.push_str("vertex 1\n");
        sources[0].modified.mark(current);

        assert_eq!(Modified(1), pr.update(current, sources));
        assert_eq!("shared 0\nvertex 1\nshared 0\nfragment 0\n", &pr.value.contents);

        assert_eq!(Modified(1), pr.update(current, sources));
        assert_eq!("shared 0\nvertex 1\nshared 0\nfragment 0\n", &pr.value.contents);
    }
}
