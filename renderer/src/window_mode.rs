use derive::EnumNext;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, EnumNext)]
pub enum WindowMode {
    Main,
    Debug,
    Split,
}

#[derive(Debug)]
pub enum WindowModeBox<M, D, S> {
    Main(M),
    Debug(D),
    Split(S),
}
