#[repr(C)]
pub struct Frustrum {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
    pub z0: f32,
    pub z1: f32,
}

unsafe fn reinterpret<A, B>(a: &A) -> &B {
    assert_eq!(::std::mem::size_of::<A>(), ::std::mem::size_of::<B>(),);
    &*(a as *const A as *const B)
}

impl AsRef<[f32; 6]> for Frustrum {
    fn as_ref(&self) -> &[f32; 6] {
        unsafe { reinterpret(self) }
    }
}
