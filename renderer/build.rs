use kernel_generator::*;

fn slice_to_bytes<T>(s: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            s.as_ptr() as *const u8,
            std::mem::size_of_val(s),
        )
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../kernel-generator");

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let mut values = [[0f32; 4]; 1024];
    generate_hbao_kernel(&mut values[0..512], 0.5);
    generate_unit_vectors(&mut values[512..1024]);
    std::fs::write(out_dir.join("hbao_kernel.bin"), slice_to_bytes(&values[..])).unwrap();
//     std::fs::write(out_dir.join("hbao_kernel.rs"), br#"""
//     pub const HBAO_KERNEL: &'static [[f32; 3]; 64] = unsafe {
//         &*(include_bytes!(concat!(env!("OUT_DIR"), "hbao_kernel.bin")).as_ptr() as *const [[f32; 3]; 64])
//     };
// """#).unwrap();

}
