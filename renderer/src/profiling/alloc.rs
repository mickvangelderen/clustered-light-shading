use gl_typed as gl;

pub struct BufferView {
    pub(super) name: gl::BufferName,
    pub(super) byte_offset: usize,
    pub(super) byte_count: usize,
}

impl BufferView {
    pub fn name(&self) -> gl::BufferName {
        self.name
    }

    pub unsafe fn clear_0u32(&mut self, gl: &gl::Gl) {
        assert_eq!(0, self.byte_count % 4);
        gl.clear_named_buffer_sub_data(
            self.name,
            gl::R32UI,
            self.byte_offset,
            self.byte_count,
            gl::RED,
            gl::UNSIGNED_INT,
            None,
        );
    }

    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    pub fn byte_count(&self) -> usize {
        self.byte_count
    }
}

pub struct AllocBuffer {
    name: gl::BufferName,
    cap: usize,
    len: usize,
}

fn make_mult_4_usize(n: usize) -> usize {
    ((n + 3) / 4) * 4
}


impl AllocBuffer {
    #[allow(unused)]
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

    fn reserve(&mut self, gl: &gl::Gl, mut new_cap: usize) {
        assert!(new_cap > 0);

        if new_cap <= self.cap {
            // Nothing to do.
            return;
        }

        // Ensure we grow by at least 1.5*self.cap and the new capacity is a multiple of 4.
        new_cap = make_mult_4_usize(std::cmp::max(new_cap, self.cap + self.cap / 2));

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

    fn grow(&mut self, gl: &gl::Gl, byte_count: usize) -> usize {
        assert_eq!(0, byte_count % 4);
        let offset = self.len;
        self.reserve(gl, offset + byte_count);
        // Set len after because reserve depends on it's current value.
        self.len = offset + byte_count;
        offset
    }

    pub fn alloc<T>(&mut self, gl: &gl::Gl, count: usize) -> BufferView {
        let byte_count = make_mult_4_usize(std::mem::size_of::<T>() * count);
        let byte_offset = self.grow(gl, byte_count);
        BufferView {
            name: self.name,
            byte_offset,
            byte_count,
        }
    }

    pub unsafe fn read(&self, gl: &gl::Gl, offset: usize, out: &mut [u8]) {
        assert!(offset + out.len() < self.cap);
        gl.get_named_buffer_sub_data(self.name, offset, out);
    }

    pub fn reset(&mut self) {
        self.len = 0
    }
}

