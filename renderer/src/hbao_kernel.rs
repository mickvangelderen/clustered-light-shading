type Bytes = [u8; 1024 * std::mem::size_of::<[f32; 4]>()];
type Floats = [[f32; 4]; 1024];

const HBAO_KERNEL: &'static Bytes = include_bytes!(concat!(env!("OUT_DIR"), "/hbao_kernel.bin"));

pub fn hbao_kernel_ref() -> &'static Floats {
    unsafe {
        assert_eq!(std::mem::size_of::<Bytes>(), std::mem::size_of::<Floats>(),);
        &*(HBAO_KERNEL.as_ptr() as *const Floats)
    }
}
