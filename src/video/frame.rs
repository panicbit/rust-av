use std::slice;
use std::os::raw::c_int;
use smallvec::SmallVec;
use ffi;
use ffi::{
    AVFrame,
    AVPixelFormat,
    av_frame_alloc,
    av_frame_free,
    av_frame_get_buffer,
};
use super::MAX_PLANES;
use video;
use errors::*;

pub struct Frame {
    ptr: *mut ffi::AVFrame,
    pixel_format: AVPixelFormat,
}

// See https://github.com/panicbit/rust-av/issues/28
unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Frame {
    /// # Panics
    ///
    /// Panics if `width`, `height` or `align` exceed `c_int::max_value()`.
    pub fn new(width: usize, height: usize, pixel_format: AVPixelFormat, align: usize) -> Result<Self> {
        unsafe {
            assert!(width <= c_int::max_value() as usize, "VideoFrame width exceeds c_int::max_value()");
            assert!(height <= c_int::max_value() as usize, "VideoFrame height exceeds c_int::max_value()");
            assert!(align <= c_int::max_value() as usize, "VideoFrame align exceeds c_int::max_value()");

            let mut frame = av_frame_alloc();
            if frame.is_null() {
                bail!("Could not allocate video frame");
            }

            // Fill in required information
            (*frame).pts = 0;
            (*frame).format = pixel_format as c_int;
            (*frame).width = width as c_int;
            (*frame).height = height as c_int;

            // Allocate actual frame buffer.
            let res = av_frame_get_buffer(frame, align as c_int);
            if res < 0 {
                av_frame_free(&mut frame);
                bail!("Could not allocate video frame buffer: 0x{:X}", res);
            }

            Ok(Self::from_ptr(frame, pixel_format))
        }
    }
}

impl Frame {
    pub fn pixel_format(&self) -> ffi::AVPixelFormat {
        self.pixel_format
    }

    pub fn width(&self) -> usize {
        self.as_ref().width as usize
    }

    pub fn height(&self) -> usize {
        self.as_ref().height as usize
    }

    pub fn linesize(&self, channel: usize) -> usize {
        self.as_ref().linesize[channel] as usize
    }

    pub fn linesizes(&self) -> [usize; MAX_PLANES as usize] {
        let mut linesizes = [0; MAX_PLANES as usize];
        for channel in 0..MAX_PLANES as usize {
            linesizes[channel] = self.linesize(channel) as usize;
        }
        linesizes
    }

    pub fn pts(&self) -> i64 {
        self.as_ref().pts
    }

    pub fn set_pts(&mut self, pts: i64) {
        self.as_mut().pts = pts;
    }

    pub fn channel(&self, channel_index: usize) -> &[u8] {
        unsafe {
            let buf_len = self.height() * self.linesize(channel_index);
            slice::from_raw_parts(self.as_ref().data[channel_index], buf_len)
        }
    }

    pub fn is_compatible_with_encoder(&self, encoder: &video::Encoder) -> bool {
           self.pixel_format() == encoder.pixel_format()
        && self.width() == encoder.width()
        && self.height() == encoder.height()
    }

    pub fn channel_mut(&mut self, channel_index: usize) -> &mut [u8] {
        unsafe {

            if ffi::av_frame_make_writable(self.ptr) < 0 {
                panic!("av_frame_make_writable failed (OOM?)");
            }

            let buf_len = self.height() * self.linesize(channel_index);

            slice::from_raw_parts_mut(self.as_mut().data[channel_index], buf_len)
        }
    }

    pub fn data(&self) -> SmallVec<[&[u8]; MAX_PLANES]> {
        unsafe {
            let num_planes = ffi::av_pix_fmt_count_planes(self.pixel_format());
            if num_planes < 0 {
                panic!("num planes negative (invalid pixel_format)");
            }
            let mut planes = SmallVec::<[&[u8]; MAX_PLANES]>::new();

            for i in 0..num_planes as usize {
                let buf_len = self.height() * self.linesize(i);
                let plane = self.as_ref().data[i];
                let plane = slice::from_raw_parts(plane, buf_len);
                planes.push(plane);
            }

            planes
        }
    }

    pub fn data_mut(&self) -> SmallVec<[&mut [u8]; MAX_PLANES]> {
        unsafe {
            if ffi::av_frame_make_writable(self.ptr) < 0 {
                panic!("av_frame_make_writable failed (OOM?)");
            }

            let num_planes = ffi::av_pix_fmt_count_planes(self.pixel_format());
            if num_planes < 0 {
                panic!("num planes negative (invalid pixel_format)");
            }
            let mut planes = SmallVec::<[&mut [u8]; MAX_PLANES]>::new();

            for i in 0..num_planes as usize {
                let buf_len = self.height() * self.linesize(i);
                let plane = self.as_ref().data[i];
                let plane = slice::from_raw_parts_mut(plane, buf_len);
                planes.push(plane);
            }

            planes
        }
    }

    pub fn fill_channel(&mut self, channel_index: usize, source: &[u8]) -> Result<()> {
        unsafe {
            use std::cmp::min;

            if ffi::av_frame_make_writable(self.ptr) < 0 {
                panic!("av_frame_make_writable failed (OOM?)");
            }

            let source_linesize = source.len() / self.height();
            let target_linesize = self.linesize(channel_index);
            let linesize = min(source_linesize, target_linesize);
            let channel = self.channel_mut(channel_index);

            let source_lines = source.chunks(source_linesize);
            let target_lines = channel.chunks_mut(target_linesize);

            for (target, source) in target_lines.zip(source_lines) {
                target[..linesize].copy_from_slice(&source[..linesize]);
            }

            Ok(())
        }
    }
}

impl Frame {
    pub fn as_ref(&self) -> &AVFrame {
        unsafe { &*self.ptr }
    }

    pub fn as_mut(&mut self) -> &mut AVFrame {
        unsafe { &mut *self.ptr }
    }

    pub unsafe fn from_ptr(ptr: *mut AVFrame, pixel_format: AVPixelFormat) -> Self {
        Frame {
            ptr: ptr,
            pixel_format: pixel_format,
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVFrame {
        self.ptr
    }

    pub fn into_raw(self) -> *mut AVFrame {
        self.ptr
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            ffi::av_frame_free(&mut self.ptr);
        }
    }
}
