const COUNT: usize = 512;
type Item = [f32; 4];
type Bytes = [u8; COUNT * std::mem::size_of::<Item>()];
type Items = [Item; COUNT];

const BYTES: &'static Bytes = include_bytes!(concat!(env!("OUT_DIR"), "/unit_sphere_dense.bin"));

pub fn get() -> &'static Items {
    unsafe { &*(BYTES.as_ptr() as *const Items) }
}
