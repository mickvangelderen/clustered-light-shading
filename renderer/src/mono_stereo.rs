use derive::EnumNext;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, EnumNext)]
pub enum MonoStereo {
    Mono,
    Stereo,
}

#[derive(Debug)]
pub enum MonoStereoBox<M, S> {
    Mono(M),
    Stereo(S),
}
