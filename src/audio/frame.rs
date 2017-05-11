use std::slice;
use std::os::raw::c_int;
use audio::ChannelLayout;
use ffi::{
    self,
    AVFrame,
    AVSampleFormat,
    av_frame_alloc,
    av_frame_get_buffer,
    av_frame_free,
    av_get_channel_layout_nb_channels,
    av_sample_fmt_is_planar,
    AV_NUM_DATA_POINTERS,
};
use errors::*;

pub struct Frame {
    ptr: *mut AVFrame,
    sample_format: AVSampleFormat,
}

// See https://github.com/panicbit/rust-av/issues/28
unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Frame {
    /// TODO: Check for overflows
    pub fn new(num_samples: usize, sample_rate: u32, sample_format: AVSampleFormat, channel_layout: ChannelLayout) -> Result<Self> {
        unsafe {

            let mut frame = av_frame_alloc();
            if frame.is_null() {
                bail!("Could not allocate frame");
            }

            (*frame).pts = 0;
            (*frame).format = sample_format as c_int;
            (*frame).channel_layout = channel_layout.bits();
            (*frame).sample_rate = sample_rate as i32;
            (*frame).nb_samples = num_samples as i32;

            if num_samples > 0 {
                let align = 0;
                let res = av_frame_get_buffer(frame, align);
                if res < 0 {
                    av_frame_free(&mut frame);
                    bail!("Could not allocate audio frame buffer: 0x{:X}", res);
                }
            }

            Ok(Frame {
                ptr: frame,
                sample_format: sample_format,
            })
        }
    }

    pub fn num_channels(&self) -> usize {
        unsafe {
            av_get_channel_layout_nb_channels(self.as_ref().channel_layout) as usize
        }
    }

    pub fn is_planar(&self) -> bool {
        unsafe {
            av_sample_fmt_is_planar(self.sample_format) != 0
        }
    }

    pub fn data_mut(&mut self) -> [&mut [u8]; AV_NUM_DATA_POINTERS as usize] {
        unsafe {
            if ffi::av_frame_make_writable(self.ptr) < 0 {
                panic!("av_frame_make_writable failed (OOM?)");
            }

            // For audio only linesize[0] is set. Every channel needs to have the same size.
            let buf_len = self.as_ref().linesize[0] as usize;
            let mut channels: [&mut [u8]; AV_NUM_DATA_POINTERS as usize] = Default::default();
            let mut num_channels = self.num_channels();
            // interleaved formats maximally have one data channel
            if !self.is_planar() && num_channels > 1 {
                num_channels = 1;
            }

            for i in 0..num_channels {
                channels[i] = slice::from_raw_parts_mut(self.as_ref().data[i], buf_len);
            }

            channels
        }
    }

    pub fn num_samples(&self) -> usize {
        self.as_ref().nb_samples as usize
    }

    pub fn set_pts(&mut self, pts: i64) {
        self.as_mut().pts = pts;
    }
}

impl Frame {
    pub unsafe fn from_ptr(ptr: *mut AVFrame, sample_format: AVSampleFormat) -> Self {
        Frame {
            ptr: ptr,
            sample_format: sample_format,
        }
    }

    pub fn as_ref(&self) -> &AVFrame {
        unsafe { &*self.ptr }
    }

    pub fn as_mut(&mut self) -> &mut AVFrame {
        unsafe { &mut *self.ptr }
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVFrame {
        self.ptr
    }
}
