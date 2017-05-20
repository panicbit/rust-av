use std::ptr;
use std::os::raw::c_int;
use LibAV;
use codec::{
    Codec,
    MediaType,
};
use ffi;
use ffi::{
    AVCodecContext,
    AVPixelFormat,
    avcodec_alloc_context3,
    avcodec_free_context,
};
use format::OutputFormat;
use generic::RefMutFrame;
use common::{self, Packet, Timebase};
use errors::*;
use util::OwnedOrRefMut;
use super::{Frame, Scaler};

// TODO: Add align field to encoder
const ALIGN: usize = 32;

pub struct Encoder {
    ptr: *mut AVCodecContext,
    scaler: Scaler,
    tmp_frame: Option<Frame>,
}

unsafe impl Send for Encoder {}
unsafe impl Sync for Encoder {}

impl Encoder {
    pub fn from_codec(codec: Codec) -> Result<EncoderBuilder> {
        EncoderBuilder::from_codec(codec)
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

    pub fn time_base(&self) -> Timebase {
        self.as_ref().time_base.into()
    }
}

impl Encoder {
    pub fn encode<'a, F>(&mut self, frame: F) -> Result<Packets> where
        F: Into<RefMutFrame<'a>>,
    {
        unsafe {
            let mut frame = frame.into().into_video_frame()
                .ok_or("Cannot encode non-video frame as video")?;

            // Do scaling if needed
            if !frame.is_compatible_with_encoder(self) {
                self.init_tmp_frame()?;

                let tmp_frame = self.tmp_frame.as_mut().unwrap();
                let scaler = &mut self.scaler;

                scaler.scale_frame(&mut frame, tmp_frame)?;

                // Copy frame data
                tmp_frame.set_pts(frame.pts());
                frame = tmp_frame;
            }        

            // Encode the frame
            {
                let mut packet = ::std::mem::zeroed();
                ffi::av_init_packet(&mut packet);

                let res = ffi::avcodec_send_frame(self.ptr, frame.as_mut_ptr());
                if res < 0 {
                    bail!("Could not encode frame: 0x{:X}", res)
                }
            }
        }

        Ok(Packets::from_mut_encoder(self))
    }

    pub fn flush(self) -> Result<Packets<'static>> {
        unsafe {
            // Flush encoder
            let res = ffi::avcodec_send_frame(self.ptr, ptr::null_mut());
            if res < 0 {
                bail!("Could not flush encoder: 0x{:X}", res)
            }

            Ok(Packets::from_encoder(self))
        }
    }

    fn init_tmp_frame(&mut self) -> Result<()> {
        if self.tmp_frame.is_none() {
            self.tmp_frame = Some(Frame::new(self.width(), self.height(), self.pixel_format(), ALIGN)?);
        }
        Ok(())
    }
}

impl Encoder {
    pub fn as_mut(&mut self) -> &mut AVCodecContext { unsafe { &mut *self.ptr } }
    pub fn as_ptr(&self) -> *const AVCodecContext { self.ptr }
    pub fn as_mut_ptr(&mut self) -> *mut AVCodecContext { self.ptr }
}

impl AsRef<AVCodecContext> for Encoder {
    fn as_ref(&self) -> &AVCodecContext {
        unsafe { &*self.ptr }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                avcodec_free_context(&mut self.ptr);
            }
        }
    }
}

/// TODO: Check for invalid value ranges
pub struct EncoderBuilder {
    codec: Codec,
    pixel_format: Option<AVPixelFormat>,
    width: Option<c_int>,
    height: Option<c_int>,
    time_base: Option<Timebase>,
    bitrate: Option<i64>,
}

impl EncoderBuilder {
    pub fn from_codec(codec: Codec) -> Result<Self> {
        common::encoder::require_is_encoder(codec)?;
        common::encoder::require_codec_type(MediaType::Video, codec)?;

        Ok(EncoderBuilder {
            codec: codec,
            pixel_format: None,
            width: None,
            height: None,
            time_base: None,
            bitrate: None,
        })
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

    pub fn time_base<TB: Into<Timebase>>(&mut self, time_base: TB) -> &mut Self {
        self.time_base = Some(time_base.into()); self
    }

    pub fn open(&self, format: OutputFormat) -> Result<Encoder> {
        unsafe {
            let width = self.width.ok_or("Video encoder width not set")?;
            let height = self.height.ok_or("Video encoder height not set")?;
            let pixel_format = self.pixel_format.ok_or("Video encoder pixel_format not set")?;
            let time_base = self.time_base.unwrap_or((1, 30).into());

            LibAV::init();

            let mut codec_context = avcodec_alloc_context3(self.codec.as_ptr());
            if codec_context.is_null() {
                bail!("Could not allocate an encoding context");
            }

            // Initialize encoder fields
            common::encoder::init(codec_context, format);
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
            (*codec_context).time_base = time_base.into();

            common::encoder::open(codec_context, "video")?;

            Ok(Encoder {
                ptr: codec_context,
                scaler: Scaler::new(),
                tmp_frame: None,
            })
        }
    }
}

pub struct Packets<'encoder> {
    encoder: OwnedOrRefMut<'encoder, Encoder>,
}

impl<'encoder> Packets<'encoder> {
    fn from_encoder(encoder: Encoder) -> Self {
        Packets {
            encoder: OwnedOrRefMut::Owned(encoder)
        }
    }

    fn from_mut_encoder(encoder: &'encoder mut Encoder) -> Self {
        Packets {
            encoder: OwnedOrRefMut::Borrowed(encoder)
        }
    }
}

impl<'encoder> Iterator for Packets<'encoder> {
    type Item = Result<Packet<'static>>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut packet = ffi::av_packet_alloc();
            let res = ffi::avcodec_receive_packet(self.encoder.as_mut_ptr(), packet);

            if res < 0 {
                ffi::av_packet_free(&mut packet);

                match res {
                    ffi::AVERROR_EAGAIN | ffi::AVERROR_EOF => return None,
                    _ => return Some(Err(format!("Failed to receive packet: 0x{:X}", res).into())),
                }
            }

            let packet = Packet::from_ptr(packet, self.encoder.time_base());

            Some(Ok(packet))
        }
    }
}

impl<'encoder> Drop for Packets<'encoder> {
    fn drop(&mut self) {
        // Receive every packet possible
        for _ in self {}
    }
}
