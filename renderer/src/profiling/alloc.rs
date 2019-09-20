use gl_typed as gl;

union BufferName {
    garbage: u32,
    value: gl::BufferName,
}

pub struct AllocBuffer {
    name: BufferName,
    cap: usize,
    len: usize,
}

impl Default for AllocBuffer {
    fn default() -> Self {
        Self {
            name: BufferName { garbage: 0 },
            cap: 0,
            len: 0,
        }
    }
}

impl AllocBuffer {
    fn new(gl: &gl::Gl, cap: usize) -> Self {
        unsafe {
            let name = BufferName {
                value: gl.create_buffer(),
            };
            gl.named_buffer_storage_reserve(name.value, cap, gl::BufferStorageFlag::READ);
            Self { name, cap: cap, len: 0 }
        }
    }

    fn reserve(&mut self, gl: &gl::Gl, mut cap: usize) {
        unsafe {
            if cap > self.cap {
                if cap < (self.cap + self.cap / 2) {
                    cap = self.cap + self.cap / 2;
                }

                cap = ((cap + 15) / 16) * 16;

                // Create a new buffer.
                let new_name = BufferName {
                    value: gl.create_buffer(),
                };
                gl.named_buffer_storage_reserve(new_name.value, cap, gl::BufferStorageFlag::READ);

                // Copy over the data from the old buffer to the new buffer.
                if self.len > 0 {
                    gl.copy_named_buffer_sub_data(self.name.value, new_name.value, 0, 0, self.len);
                }

                // Replace the old buffer with the new buffer.
                let old_name = std::mem::replace(&mut self.name, new_name);
                self.cap = cap;
                gl.delete_buffer(old_name.value);
            }
        }
    }

    fn grow(&mut self, gl: &gl::Gl, byte_count: usize) {
        let padded_byte_count = (byte_count + 15)/16 * 16;
        self.reserve(gl, self.len + padded_byte_count);
        self.len += padded_byte_count;
    }

    pub unsafe fn read(&self, gl: &gl::Gl, offset: usize, out: &mut [u8]) {
        assert!(offset + out.len() < self.cap);
        gl.get_named_buffer_sub_data(self.name.value, offset, out);
    }

    pub unsafe fn copy_from(&mut self, gl: &gl::Gl, src_name: &gl::BufferName, src_offset: usize, byte_count: usize) -> usize {
        let offset = self.len;
        self.grow(gl, byte_count);
        gl.copy_named_buffer_sub_data(src_name, self.name.value, src_offset, offset, byte_count);
        offset
    }

    pub fn reset(&mut self) {
        self.len = 0
    }
}
