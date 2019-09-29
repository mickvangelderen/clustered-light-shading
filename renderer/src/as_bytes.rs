pub trait ValueAsBytes {
    fn value_as_bytes(&self) -> &[u8];
    unsafe fn value_as_bytes_mut(&mut self) -> &mut [u8];
}

impl<T> ValueAsBytes for T {
    fn value_as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, std::mem::size_of::<Self>()) }
    }

    unsafe fn value_as_bytes_mut(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self as *mut Self as *mut u8, std::mem::size_of::<Self>())
    }
}

pub trait SliceAsBytes {
    fn slice_as_bytes(&self) -> &[u8];
    unsafe fn slice_as_bytes_mut(&mut self) -> &mut [u8];
}

impl<T> SliceAsBytes for [T] {
    fn slice_as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, std::mem::size_of_val(self)) }
    }

    unsafe fn slice_as_bytes_mut(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.as_mut_ptr() as *mut u8, std::mem::size_of_val(self))
    }
}

pub trait VecAsBytes {
    fn vec_as_bytes(&self) -> &[u8];
    unsafe fn vec_as_bytes_mut(&mut self) -> &mut [u8];
}

impl<T> VecAsBytes for Vec<T> {
    fn vec_as_bytes(&self) -> &[u8] {
        (&self[..]).slice_as_bytes()
    }

    unsafe fn vec_as_bytes_mut(&mut self) -> &mut [u8] {
        (&mut self[..]).slice_as_bytes_mut()
    }
}
