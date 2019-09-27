use gl_typed as gl;

pub struct AllocBuffer {
    name: gl::BufferName,
    cap: usize,
    len: usize,
}

fn make_mult_16_usize(n: usize) -> usize {
    ((n + 15) / 16) * 16
}

impl AllocBuffer {
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            let name = gl.create_buffer();
            Self { name, cap: 0, len: 0 }
        }
    }

    pub fn with_capacity(gl: &gl::Gl, cap: usize) -> Self {
        assert!(cap > 0);
        unsafe {
            let name = gl.create_buffer();
            gl.named_buffer_storage_reserve(name, cap, gl::BufferStorageFlag::READ);
            Self { name, cap: cap, len: 0 }
        }
    }

    pub fn reserve(&mut self, gl: &gl::Gl, mut new_cap: usize) {
        assert!(new_cap > 0);

        if new_cap <= self.cap {
            // Nothing to do.
            return;
        }

        // Ensure we grow by at least 1.5*self.cap.
        if new_cap < (self.cap + self.cap / 2) {
            new_cap = self.cap + self.cap / 2;
        }

        // Ensure new_cap is a multiple of 16.
        new_cap = make_mult_16_usize(new_cap);

        unsafe {
            if self.cap == 0 {
                // We have never called buffer storage so we can just do it now.
                gl.named_buffer_storage_reserve(self.name, new_cap, gl::BufferStorageFlag::READ);
            } else {
                // Create a new buffer with new capacity.
                let new_name = gl.create_buffer();
                gl.named_buffer_storage_reserve(new_name, new_cap, gl::BufferStorageFlag::READ);

                // Copy over the data from the old buffer to the new buffer.
                if self.len > 0 {
                    gl.copy_named_buffer_sub_data(self.name, new_name, 0, 0, self.len);
                }

                // Replace the old buffer with the new buffer.
                let old_name = std::mem::replace(&mut self.name, new_name);
                gl.delete_buffer(old_name);
            }

            // In all cases, we now have a buffer with capacity new_cap. 
            self.cap = new_cap;
        }
    }

    fn grow(&mut self, gl: &gl::Gl, byte_count: usize) {
        let padded_byte_count = make_mult_16_usize(byte_count);
        self.reserve(gl, self.len + padded_byte_count);
        self.len += padded_byte_count;
    }

    pub unsafe fn read(&self, gl: &gl::Gl, offset: usize, out: &mut [u8]) {
        assert!(offset + out.len() < self.cap);
        gl.get_named_buffer_sub_data(self.name, offset, out);
    }

    pub unsafe fn copy_from(
        &mut self,
        gl: &gl::Gl,
        src_name: &gl::BufferName,
        src_offset: usize,
        byte_count: usize,
    ) -> usize {
        let offset = self.len;
        self.grow(gl, byte_count);
        gl.copy_named_buffer_sub_data(src_name, self.name, src_offset, offset, byte_count);
        offset
    }

    pub fn reset(&mut self) {
        self.len = 0
    }
}
