use std::io;

pub trait ReadExt {
    unsafe fn read_val<T>(&mut self) -> io::Result<T>;
    unsafe fn read_vec<T>(&mut self, element_count: usize) -> io::Result<Vec<T>>;
    fn read_bytes(&mut self, byte_count: usize) -> io::Result<Vec<u8>>;
}

impl<R: io::Read> ReadExt for R {
    unsafe fn read_val<T>(&mut self) -> io::Result<T> {
        let mut value = std::mem::MaybeUninit::<T>::uninit();
        self.read_exact(std::slice::from_raw_parts_mut(
            value.as_mut_ptr() as *mut u8,
            std::mem::size_of::<T>(),
        ))?;
        Ok(value.assume_init())
    }

    unsafe fn read_vec<T>(&mut self, element_count: usize) -> io::Result<Vec<T>> {
        let element_byte_count = std::mem::size_of::<T>();
        let byte_count = element_count * element_byte_count;
        let mut elements = Vec::with_capacity(element_count);
        self.read_exact(std::slice::from_raw_parts_mut(
            elements.as_mut_ptr() as *mut u8,
            byte_count,
        ))?;
        elements.set_len(element_count);
        Ok(elements)
    }

    fn read_bytes(&mut self, byte_count: usize) -> io::Result<Vec<u8>> {
        unsafe {
            let mut bytes = Vec::with_capacity(byte_count);
            self.read_exact(std::slice::from_raw_parts_mut(
                bytes.as_mut_ptr(),
                byte_count,
            ))?;
            bytes.set_len(byte_count);
            Ok(bytes)
        }
    }
}

pub trait SeekExt {
    fn pos(&mut self) -> u64;
}

impl<S: io::Seek> SeekExt for S {
    fn pos(&mut self) -> u64 {
        self.seek(io::SeekFrom::Current(0)).unwrap()
    }
}
