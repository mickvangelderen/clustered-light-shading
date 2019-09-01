use derive::EnumNext;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, EnumNext)]
pub enum WindowMode {
    Main,
    Debug,
    Split,
}
