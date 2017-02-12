use libc::{c_int, int64_t};
use LibAV;
use super::Codec;
use ffi;
use ffi::{
    AVCodecContext,
    AVPixelFormat,
    AVRational,
    avcodec_alloc_context3,
    avcodec_free_context,
};
use frame::{RefMutFrame, VideoFrame};
use scaler::Scaler;

// TODO: Add align field to encoder
const ALIGN: usize = 32;

pub struct VideoEncoder {
    ptr: *mut AVCodecContext,
    scaler: Option<Scaler>,
    tmp_frame: Option<VideoFrame>,
}

impl VideoEncoder {
    pub fn from_codec(codec: Codec) -> VideoEncoderBuilder {
        VideoEncoderBuilder::from_codec(codec)
    }

    pub fn pixel_format(&self) -> ffi::AVPixelFormat {
        self.as_ref().pix_fmt
    }

    pub fn width(&self) -> usize {
        self.as_ref().width as usize
    }

    pub fn height(&self) -> usize {
        self.as_ref().height as usize
    }

    pub fn codec(&self) -> Codec {
        unsafe {
            Codec::from_ptr(self.as_ref().codec)
        }
    }

    pub fn time_base(&self) -> AVRational {
        self.as_ref().time_base
    }
}

impl VideoEncoder {
    pub unsafe fn send_frame<'a, F, H>(&mut self, frame: F, mut packet_handler: H) -> Result<(), String> where
        F: Into<RefMutFrame<'a>>,
        H: FnMut(&mut ffi::AVPacket) -> Result<(), String>,
    {
        let mut frame = frame.into().into_video_frame()
            .ok_or_else(|| format!("Cannot encode non-video frame as video"))?;

        // Do scaling if needed
        if !frame.is_compatible_with_encoder(self) {
            self.update_scaler(frame)?;
            self.init_tmp_frame()?;

            let tmp_frame = self.tmp_frame.as_mut().unwrap();
            let scaler = self.scaler.as_mut().unwrap();

            scaler.scale_frame(&mut frame, tmp_frame);

            // Copy frame data
            tmp_frame.set_pts(frame.pts());
            frame = tmp_frame;
        }        

        // Do the encoding business
        let mut packet = ::std::mem::zeroed();

        ffi::av_init_packet(&mut packet);

        // TODO: Check errors on send_frame too?
        ffi::avcodec_send_frame(self.ptr, frame.as_mut_ptr());
        loop {
            match ffi::avcodec_receive_packet(self.ptr, &mut packet) {
                0 => {
                    let handler_success = packet_handler(&mut packet);
                    ffi::av_packet_unref(&mut packet);
                    handler_success?
                },
                ffi::RUST_AVERROR_EAGAIN | ffi::RUST_AVERROR_EOF => return Ok(()),
                _ => return Err(format!("Error encoding packet")),
            }
        }
    }

    fn scaler_needs_update(&self, source: &VideoFrame) -> bool {
        if let Some(ref scaler) = self.scaler {
               source.pixel_format() != scaler.source_pixel_format()
            || source.width() != scaler.source_width()
            || source.height() != scaler.source_height()
        } else {
            true
        }
    }

    fn update_scaler(&mut self, frame: &VideoFrame) -> Result<(), String> {
        if self.scaler_needs_update(frame) {
            self.scaler = Some(Scaler::new(
                frame.width(), frame.height(), frame.pixel_format(),
                self.width(), self.height(), self.pixel_format()
            )?);
        }
        Ok(())
    }

    fn init_tmp_frame(&mut self) -> Result<(), String> {
        if self.tmp_frame.is_none() {
            self.tmp_frame = Some(VideoFrame::new(self.width(), self.height(), self.pixel_format(), ALIGN)?);
        }
        Ok(())
    }
}

impl VideoEncoder {
    pub fn as_ref(&self) -> &AVCodecContext { unsafe { &*self.ptr } }
    pub fn as_mut(&mut self) -> &mut AVCodecContext { unsafe { &mut *self.ptr } }
    pub fn as_ptr(&self) -> *const AVCodecContext { self.ptr }
    pub fn as_mut_ptr(&mut self) -> *mut AVCodecContext { self.ptr }
}

impl Drop for VideoEncoder {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                avcodec_free_context(&mut self.ptr);
            }
        }
    }
}

/// TODO: Check for invalid value ranges
pub struct VideoEncoderBuilder {
    codec: Codec,
    pixel_format: Option<AVPixelFormat>,
    width: Option<c_int>,
    height: Option<c_int>,
    time_base: Option<AVRational>,
    bitrate: Option<int64_t>,
}

impl VideoEncoderBuilder {
    pub fn from_codec(codec: Codec) -> Self {
        VideoEncoderBuilder {
            codec: codec,
            pixel_format: None,
            width: None,
            height: None,
            time_base: None,
            bitrate: None,
        }
    }

    /// TODO: Check for overflow
    pub fn width(&mut self, width: usize) -> &mut Self {
        self.width = Some(width as i32); self
    }

    /// TODO: Check for overflow
    pub fn height(&mut self, height: usize) -> &mut Self {
        self.height = Some(height as i32); self
    }

    pub fn pixel_format(&mut self, pixel_format: AVPixelFormat) -> &mut Self {
        self.pixel_format = Some(pixel_format); self
    }

    pub fn framerate(&mut self, framerate: usize) -> &mut Self {
        self.time_base = Some(AVRational { num: 1, den: framerate as i32 }); self
    }

    pub fn open(&self) -> Result<VideoEncoder, String> {
        unsafe {
            let width = self.width.ok_or_else(|| format!("VideoEncoder width not set"))?;
            let height = self.height.ok_or_else(|| format!("VideoEncoder height not set"))?;
            let pixel_format = self.pixel_format.ok_or_else(|| format!("VideoEncoder pixel_format not set"))?;
            let time_base = self.time_base.unwrap_or(AVRational { num: 1, den: 30 });

            LibAV::init();

            let mut codec_context = avcodec_alloc_context3(self.codec.as_ptr());
            if codec_context.is_null() {
                return Err(format!("Could not allocate an encoding context"));
            }

            (*codec_context).codec_id = self.codec.id();
            (*codec_context).width = width;
            (*codec_context).height = height;
            (*codec_context).pix_fmt = pixel_format;
            if let Some(bitrate) = self.bitrate {
                (*codec_context).bit_rate = bitrate;
            }
            // time_base: This is the fundamental unit of time (in seconds) in terms
            // of which frame timestamps are represented. For fixed-fps content,
            // time_base should be 1/framerate and timestamp increments should be
            // identical to 1.
            (*codec_context).time_base = time_base;

            Ok(VideoEncoder {
                ptr: codec_context,
                scaler: None,
                tmp_frame: None,
            })
        }
    }
}
