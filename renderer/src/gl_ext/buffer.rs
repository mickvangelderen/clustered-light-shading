use gl_typed as gl;
use gl_typed::Gl;

pub struct FixedCapacityBuffer {
    name: gl::BufferName,
    cap: usize,
    len: usize,
}

#[derive(Debug)]
pub enum AllocError {
    InsufficientCapacity
}

impl FixedCapacityBuffer {
    pub fn new(gl: &Gl, label: &str, capacity: usize, flags: gl::BufferStorageFlag) -> Self {
        unsafe {
            let name = gl.create_buffer();
            gl.buffer_label(&name, label);
            gl.named_buffer_storage_reserve(name, capacity, flags);
            Self {
                name,
                cap: capacity,
                len: 0,
            }
        }
    }

    pub fn name(&self) -> gl::BufferName {
        self.name
    }

    pub fn alloc<T>(&mut self, gl: &Gl, count: usize) -> BufferSlice {
        self.try_alloc::<T>(gl, count).unwrap()
    }

    pub fn try_alloc<T>(&mut self, _gl: &Gl, count: usize) -> Result<BufferSlice, AllocError> {
        let slice = BufferSlice::new::<T>(self.len, count);

        if slice.byte_end() <= self.cap {
            self.len = slice.byte_end();
            Ok(slice)
        } else {
            Err(AllocError::InsufficientCapacity)
        }
    }

    pub fn clear(&mut self, gl: &Gl) {
        unsafe {
            gl.invalidate_buffer_data(self.name);
            self.len = 0;
        }
    }
}

pub struct BufferSlice {
    pub byte_offset: usize,
    pub byte_count: usize,
}

impl BufferSlice {
    pub fn new<T>(byte_offset: usize, count: usize) -> Self {
        Self {
            byte_offset: ((byte_offset + 3) / 4) * 4,
            byte_count: count * std::mem::size_of::<T>(),
        }
    }

    #[inline]
    pub fn byte_end(&self) -> usize {
        self.byte_offset + self.byte_count
    }
}
