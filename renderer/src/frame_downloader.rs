use crate::*;

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Format {
    RGBA,
}

impl Format {
    #[inline]
    pub fn byte_count(&self) -> usize {
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

impl Into<image::ColorType> for Format {
    fn into(self) -> image::ColorType {
        match self {
            Format::RGBA => image::ColorType::RGBA(8),
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
        self.width as usize * self.height as usize * self.format.byte_count()
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
    pub fn image_byte_count(&self) -> usize {
        self.height as usize * self.row_byte_count()
    }

    pub fn row_byte_count(&self) -> usize {
        self.width as usize * self.format.byte_count()
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

pub struct EncoderThread {
    tx: mpsc::SyncSender<(PathBuf, Image)>,
    handle: thread::JoinHandle<()>,
}

impl Default for EncoderThread {
    fn default() -> Self {
        let (tx, rx) = mpsc::sync_channel::<(PathBuf, Image)>(2);

        let handle = thread::spawn(move || {
            for (path, image) in rx.iter() {
                let mut file = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
                let mut encoder = image::jpeg::JPEGEncoder::new_with_quality(&mut file, 95);
                encoder
                    .encode(&image.bytes, image.width, image.height, image.format.into())
                    .unwrap();
            }
        });

        Self { tx, handle }
    }
}

impl EncoderThread {
    pub fn join(self) -> thread::Result<()> {
        let Self { tx, handle } = self;
        std::mem::drop(tx);
        handle.join()
    }
}

pub struct FrameDownloader {
    buffer_ring: BufferRing,
    last_image: Image,
    thread_pool: Vec<EncoderThread>,
}

impl FrameDownloader {
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            buffer_ring: BufferRing::new(gl),
            last_image: Default::default(),
            thread_pool: std::iter::repeat_with(EncoderThread::default).take(6).collect(),
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
                {
                    assert_eq!(buffer.format.byte_count(), 4);
                    gl.pixel_store_pack_alignment(gl::PixelAlignment::P4);
                }
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
            let image_frame_index = frame_index.to_usize() - BufferRing::CAPACITY;

            let path = frames_dir.join(&format!("{}.jpg", image_frame_index));

            // Clone bytes and flip-y while we're at it.
            let image = unsafe {
                let mut bytes = Vec::<u8>::with_capacity(self.last_image.image_byte_count());
                bytes.set_len(self.last_image.image_byte_count());

                let row_byte_count = self.last_image.row_byte_count() as isize;
                let dst = bytes.as_mut_ptr();
                let src = self.last_image.bytes.as_ptr();

                for y in 0..self.last_image.height as isize {
                    std::ptr::copy_nonoverlapping(
                        src.offset((self.last_image.height as isize - 1 - y) * row_byte_count),
                        dst.offset(y * row_byte_count),
                        row_byte_count as usize,
                    );
                }

                Image {
                    bytes: bytes,
                    width: self.last_image.width,
                    height: self.last_image.height,
                    format: self.last_image.format,
                }
            };

            let thread_index = image_frame_index % self.thread_pool.len();
            self.thread_pool[thread_index].tx.send((path, image)).unwrap();
        }

        &self.last_image
    }
}

impl Drop for FrameDownloader {
    fn drop(&mut self) {
        for thread in self.thread_pool.drain(..) {
            thread.join().unwrap();
        }
    }
}
