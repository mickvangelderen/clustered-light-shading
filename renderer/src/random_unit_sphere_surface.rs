pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 256;
const COUNT: usize = WIDTH * HEIGHT;
type Item = [u8; 3];
type Bytes = [u8; COUNT * std::mem::size_of::<Item>()];
type Items = [Item; COUNT];

const BYTES: &'static Bytes = include_bytes!(concat!(env!("OUT_DIR"), "/unit_sphere_surface.bin"));

pub fn get() -> &'static Items {
    unsafe { &*(BYTES.as_ptr() as *const Items) }
}
