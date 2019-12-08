use crate::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Format {
    RGBA,
}

impl Format {
    #[inline]
    pub fn bytes(&self) -> usize {
        match *self {
            Format::RGBA => 4,
        }
    }
}

impl Into<gl::Format> for Format {
    fn into(self) -> gl::Format {
        match self {
            Format::RGBA => gl::Format::Rgba,
        }
    }
}

struct Buffer {
    name: gl::BufferName,
    width: u32,
    height: u32,
    format: Format,
}

impl Buffer {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            name: unsafe { gl.create_buffer() },
            width: 0,
            height: 0,
            format: Format::RGBA,
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn drop(self, gl: &gl::Gl) {
        unsafe { gl.delete_buffer(self.name) }
    }

    #[inline]
    pub fn resize(&mut self, gl: &gl::Gl, width: u32, height: u32, format: Format) {
        unsafe {
            let old_len = self.len();
            self.width = width;
            self.height = height;
            self.format = format;
            let new_len = self.len();
            if old_len != new_len {
                if old_len != 0 {
                    gl.delete_buffer(self.name);
                    self.name = gl.create_buffer();
                }
                if new_len != 0 {
                    gl.named_buffer_storage_reserve(self.name, new_len, gl::BufferStorageFlag::READ);
                }
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.width as usize * self.height as usize * self.format.bytes()
    }
}

pub struct Image {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: Format,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            bytes: Vec::new(),
            width: 0,
            height: 0,
            format: Format::RGBA,
        }
    }
}

impl Image {
    #[inline]
    pub fn desired_len(&self) -> usize {
        self.width as usize * self.height as usize * self.format.bytes()
    }
}

struct BufferRing {
    ring: [Buffer; Self::CAPACITY],
}

impl BufferRing {
    pub const CAPACITY: usize = 3;

    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            ring: [Buffer::new(gl), Buffer::new(gl), Buffer::new(gl)],
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn drop(self, gl: &gl::Gl) {
        let [a, b, c] = self.ring;
        a.drop(gl);
        b.drop(gl);
        c.drop(gl);
    }
}

impl std::ops::Index<FrameIndex> for BufferRing {
    type Output = Buffer;

    fn index(&self, index: FrameIndex) -> &Self::Output {
        &self.ring[index.to_usize() % Self::CAPACITY]
    }
}

impl std::ops::IndexMut<FrameIndex> for BufferRing {
    fn index_mut(&mut self, index: FrameIndex) -> &mut Self::Output {
        &mut self.ring[index.to_usize() % Self::CAPACITY]
    }
}

pub struct FrameDownloader {
    buffer_ring: BufferRing,
    last_image: Image,
}

impl FrameDownloader {
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            buffer_ring: BufferRing::new(gl),
            last_image: Default::default(),
        }
    }

    pub fn record_frame(
        &mut self,
        frames_dir: &Path,
        gl: &gl::Gl,
        frame_index: FrameIndex,
        width: u32,
        height: u32,
        format: Format,
    ) -> &Image {
        let buffer = &mut self.buffer_ring[frame_index];

        unsafe {
            if frame_index.to_usize() >= BufferRing::CAPACITY {
                // Read back.
                self.last_image.width = buffer.width;
                self.last_image.height = buffer.height;
                self.last_image.format = buffer.format;

                let src_ptr = gl.map_named_buffer(buffer.name, gl::MapAccessFlag::READ_ONLY) as *const u8;
                if !src_ptr.is_null() {
                    self.last_image.bytes.resize(buffer.len(), 0u8);
                    let dst_ptr = self.last_image.bytes.as_mut_ptr();
                    std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, buffer.len());
                }
                drop(src_ptr);
                gl.unmap_named_buffer(buffer.name);
            }

            // Record.
            buffer.resize(gl, width, height, format);
            if buffer.len() > 0 {
                gl.bind_buffer(gl::PIXEL_PACK_BUFFER, buffer.name);
                gl.read_pixels(
                    0,
                    0,
                    buffer.width as i32,
                    buffer.height as i32,
                    buffer.format,
                    gl::UNSIGNED_BYTE,
                    std::ptr::null_mut(),
                );
                gl.unbind_buffer(gl::PIXEL_PACK_BUFFER);
            }
        }

        if frame_index.to_usize() >= BufferRing::CAPACITY {
            let frame_index = FrameIndex::from_usize(frame_index.to_usize() - BufferRing::CAPACITY);
            let frame_path = frames_dir.join(&format!("{}.bmp", frame_index.to_usize()));
            let mut file = std::io::BufWriter::new(std::fs::File::create(frame_path).unwrap());
            file.write_all(&crate::bmp::rgba_header(self.last_image.width, self.last_image.height))
                .unwrap();
            file.write_all(&self.last_image.bytes).unwrap();
        }

        &self.last_image
    }
}
