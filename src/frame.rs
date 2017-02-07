use std::slice;
use libc::c_int;
use ffi;
use ffi::{
    AVFrame,
    AVPixelFormat,
    av_frame_alloc,
    av_frame_free,
    av_frame_get_buffer,
    AV_NUM_DATA_POINTERS,
};
use codec::VideoEncoder;

pub struct VideoFrame {
    ptr: *mut ffi::AVFrame,
    pixel_format: AVPixelFormat,
}

impl VideoFrame {
    /// # Panics
    ///
    /// Panics if `width`, `height` or `align` exceed `c_int::max_value()`.
    pub fn new(width: usize, height: usize, pixel_format: AVPixelFormat, align: usize) -> Result<VideoFrame, String> {
        unsafe {
            assert!(width <= c_int::max_value() as usize, "VideoFrame width exceeds c_int::max_value()");
            assert!(height <= c_int::max_value() as usize, "VideoFrame height exceeds c_int::max_value()");
            assert!(align <= c_int::max_value() as usize, "VideoFrame align exceeds c_int::max_value()");

            let mut frame = av_frame_alloc();
            if frame.is_null() {
                return Err(format!("Could not allocate video frame"));
            }

            // Fill in required information
            (*frame).format = pixel_format as c_int;
            (*frame).width = width as c_int;
            (*frame).height = height as c_int;

            // Allocate actual frame buffer.
            let res = av_frame_get_buffer(frame, align as c_int);
            if res < 0 {
                av_frame_free(&mut frame);
                return Err(format!("Could not allocate video frame buffer"));
            }

            Ok(Self::from_ptr(frame, pixel_format))
        }
    }
}

impl VideoFrame {
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

    pub fn linesizes(&self) -> [usize; AV_NUM_DATA_POINTERS as usize] {
        let mut linesizes = [0; AV_NUM_DATA_POINTERS as usize];
        for channel in 0..AV_NUM_DATA_POINTERS as usize {
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

    pub fn is_compatible_with_encoder(&self, encoder: &VideoEncoder) -> bool {
           self.pixel_format() == encoder.pixel_format()
        && self.width() == encoder.width()
        && self.height() == encoder.height()
    }

    pub fn channel_mut(&mut self, channel_index: usize) -> &mut [u8] {
        unsafe {
            let buf_len = self.height() * self.linesize(channel_index);
            slice::from_raw_parts_mut(self.as_mut().data[channel_index], buf_len)
        }
    }

    pub fn fill_channel(&mut self, channel_index: usize, source: &[u8]) -> Result<(), String> {
        unsafe {
            use std::cmp::min;
            // when we pass a frame to the encoder, it may keep a reference to it
            // internally; make sure we do not overwrite it here
            let res = ffi::av_frame_make_writable(self.as_mut_ptr());
            if res < 0 {
                return Err(format!("Failed to make frame writeable"));
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

impl VideoFrame {
    pub fn as_ref(&self) -> &AVFrame {
        unsafe { &*self.ptr }
    }

    pub fn as_mut(&mut self) -> &mut AVFrame {
        unsafe { &mut *self.ptr }
    }

    pub unsafe fn from_ptr(ptr: *mut AVFrame, pixel_format: AVPixelFormat) -> Self {
        VideoFrame {
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

impl Drop for VideoFrame {
    fn drop(&mut self) {
        unsafe {
            ffi::av_frame_free(&mut self.ptr);
        }
    }
}
