const HBAO_KERNEL: &'static [u8; 64*4*4] = include_bytes!(concat!(env!("OUT_DIR"), "/hbao_kernel.bin"));

pub fn hbao_kernel_ref() -> &'static [[f32; 4]; 64] {
    unsafe {
        assert_eq!(std::mem::size_of::<[[f32; 4]; 64]>(), std::mem::size_of::<[u8; 64*4*4]>());
        &*(HBAO_KERNEL.as_ptr() as *const [[f32; 4]; 64])
    }
}
