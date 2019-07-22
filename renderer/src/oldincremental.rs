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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Modified(Revision);

impl Modified {
    pub const NONE: Self = Self(Revision(0));

    #[inline]
    pub const fn clean(global: &Global) -> Self {
        Modified(global.0)
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct Verified(Revision);

#[derive(Debug)]
#[repr(transparent)]
struct Computed(Revision);

pub struct Leaf<T> {
    pub modified: Modified,
    pub value: T,
}

impl<T> Leaf<T> {
    #[inline]
    pub fn clean(global: &Global, value: T) -> Self {
        Leaf {
            modified: Modified::clean(global),
            value,
        }
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

