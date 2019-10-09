pub trait ValueAsBytes {
    unsafe fn value_as_bytes_mut(&mut self) -> &mut [u8];
}

impl<T> ValueAsBytes for T {
    unsafe fn value_as_bytes_mut(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self as *mut Self as *mut u8, std::mem::size_of::<Self>())
    }
}
