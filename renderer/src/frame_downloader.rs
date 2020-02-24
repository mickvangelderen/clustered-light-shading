use crate::*;

use std::collections::VecDeque;
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

pub struct EncoderThread {
    tx: mpsc::SyncSender<(PathBuf, Image)>,
    handle: thread::JoinHandle<()>,
}

impl Default for EncoderThread {
    fn default() -> Self {
        let (tx, rx) = mpsc::sync_channel::<(PathBuf, Image)>(2);

        let handle = thread::spawn(move || {
            for (path, image) in rx.iter() {
                // Clone bytes and flip-y while we're at it.
                let image = unsafe {
                    let mut bytes = Vec::<u8>::with_capacity(image.image_byte_count());
                    bytes.set_len(image.image_byte_count());

                    let row_byte_count = image.row_byte_count() as isize;
                    let dst = bytes.as_mut_ptr();
                    let src = image.bytes.as_ptr();

                    for y in 0..image.height as isize {
                        std::ptr::copy_nonoverlapping(
                            src.offset((image.height as isize - 1 - y) * row_byte_count),
                            dst.offset(y * row_byte_count),
                            row_byte_count as usize,
                        );
                    }

                    Image { bytes: bytes, ..image }
                };

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

struct Transfer {
    frame_index: FrameIndex,
    buffer: Buffer,
    path: PathBuf,
}

pub struct FrameDownloader {
    buffers: VecDeque<Buffer>,
    transfers: VecDeque<Transfer>,
    next_thread_index: usize,
    thread_pool: Vec<EncoderThread>,
}

impl FrameDownloader {
    pub fn new() -> Self {
        Self {
            buffers: Default::default(),
            transfers: Default::default(),
            next_thread_index: 0,
            thread_pool: std::iter::repeat_with(EncoderThread::default).take(6).collect(),
        }
    }

    pub fn process_transfers(&mut self, gl: &gl::Gl, frame_index: FrameIndex) {
        while let Some(transfer) = self.transfers.front() {
            if frame_index < transfer.frame_index {
                break;
            }
            let Transfer {
                buffer,
                path,
                ..
            } = self.transfers.pop_front().unwrap();

            // Read buffer data, submit to thread.
            unsafe {
                let src_ptr = gl.map_named_buffer(buffer.name, gl::MapAccessFlag::READ_ONLY) as *const u8;
                if !src_ptr.is_null() {
                    let mut bytes = Vec::with_capacity(buffer.len());
                    bytes.set_len(buffer.len());
                    std::ptr::copy_nonoverlapping(src_ptr, bytes.as_mut_ptr(), buffer.len());

                    let image = Image {
                        bytes,
                        width: buffer.width,
                        height: buffer.height,
                        format: buffer.format,
                    };

                    let thread_index = self.next_thread_index;
                    self.next_thread_index = (self.next_thread_index + 1) % self.thread_pool.len();
                    self.thread_pool[thread_index].tx.send((path, image)).unwrap();
                }
                drop(src_ptr);
                gl.unmap_named_buffer(buffer.name);
            }

            // Re-use buffer.
            self.buffers.push_back(buffer);
        }
    }

    pub fn record_frame(
        &mut self,
        gl: &gl::Gl,
        frame_index: FrameIndex,
        path: PathBuf,
        width: u32,
        height: u32,
        format: Format,
    ) {
        let mut buffer = self.buffers.pop_front().unwrap_or_else(|| Buffer::new(gl));

        unsafe {
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

        self.transfers.push_back(Transfer {
            frame_index: FrameIndex::from_usize(frame_index.to_usize() + 3),
            buffer,
            path,
        });
    }
}

impl Drop for FrameDownloader {
    fn drop(&mut self) {
        // FIXME: Drop buffers and frames!
        for thread in self.thread_pool.drain(..) {
            thread.join().unwrap();
        }
    }
}
