use std::ptr;
use std::os::raw::c_int;
use LibAV;
use codec::{
    Codec,
    MediaType,
};
use ffi::{
    self,
    AVCodecContext,
    AVSampleFormat,
    AVRational,
    avcodec_alloc_context3,
    avcodec_free_context,
    av_get_channel_layout_nb_channels,
};
use ffi::AVSampleFormat::AV_SAMPLE_FMT_S16;
use format::OutputFormat;
use audio::ChannelLayout;
use audio::constants::CHANNEL_LAYOUT_STEREO;
use generic::RefMutFrame;
use common::{self, Packet, Timebase};
use errors::*;
use util::OwnedOrRefMut;

pub struct Encoder {
    ptr: *mut AVCodecContext,
}
unsafe impl Send for Encoder {}
unsafe impl Sync for Encoder {}

impl Encoder {
    pub fn from_codec(codec: Codec) -> Result<EncoderBuilder> {
        EncoderBuilder::from_codec(codec)
    }

    pub fn sample_format(&self) -> AVSampleFormat {
        self.as_ref().sample_fmt
    }

    /// TODO: Check for underflow
    pub fn sample_rate(&self) -> u32 {
        self.as_ref().sample_rate as u32
    }

    pub fn time_base(&self) -> Timebase {
        self.as_ref().time_base.into()
    }

    // Returns the frame size required by the encoder.
    // If the result is `None`, any frame size can be used.
    pub fn frame_size(&self) -> Option<usize> {
        match self.as_ref().frame_size as usize {
            0 => None,
            size => Some(size),
        }
    }

    pub fn codec(&self) -> Codec {
        unsafe {
            Codec::from_ptr(self.as_ref().codec)
        }
    }
}

impl Encoder {
    pub fn encode<'a, F>(&mut self, frame: F) -> Result<Packets> where
        F: Into<RefMutFrame<'a>>,
    {
        unsafe {
            let mut frame = frame.into().into_audio_frame()
                .ok_or("Cannot encode non-audio frame as audio")?;

            // Do scaling if needed
            // if !frame.is_compatible_with_encoder(self) {
            //     self.update_scaler(frame)?;
            //     self.init_tmp_frame()?;

            //     let tmp_frame = self.tmp_frame.as_mut().unwrap();
            //     let scaler = self.scaler.as_mut().unwrap();

            //     scaler.scale_frame(&mut frame, tmp_frame);

            //     // Copy frame data
            //     tmp_frame.set_pts(frame.pts());
            //     frame = tmp_frame;
            // }        

            // Encode the frame
            {
                let mut packet = ::std::mem::zeroed();
                ffi::av_init_packet(&mut packet);

                let res = ffi::avcodec_send_frame(self.ptr, frame.as_mut_ptr());
                if res < 0 {
                    bail!("Could not encode frame: 0x{:X}", res)
                }

            }

            Ok(Packets::from_mut_encoder(self))
        }
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

pub struct EncoderBuilder {
    codec: Codec,
    sample_format: Option<AVSampleFormat>,
    sample_rate: Option<u32>,
    channel_layout: Option<ChannelLayout>,
}

impl EncoderBuilder {
    pub fn from_codec(codec: Codec) -> Result<Self> {
        common::encoder::require_is_encoder(codec)?;
        common::encoder::require_codec_type(MediaType::Audio, codec)?;

        Ok(EncoderBuilder {
            codec: codec,
            sample_format: None,
            sample_rate: None,
            channel_layout: None,
        })
    }

    pub fn sample_format(&mut self, sample_format: AVSampleFormat) -> &mut Self {
        self.sample_format = Some(sample_format); self
    }

    /// TODO: Check for overflow
    pub fn sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        self.sample_rate = Some(sample_rate); self
    }

    pub fn channel_layout(&mut self, channel_layout: ChannelLayout) -> &mut Self {
        self.channel_layout = Some(channel_layout); self
    }

    pub fn open(&self, format: OutputFormat) -> Result<Encoder> {
        unsafe {
            let sample_rate = self.sample_rate.unwrap_or(44100) as c_int;
            let sample_format = self.sample_format.unwrap_or(AV_SAMPLE_FMT_S16);
            let channel_layout = self.channel_layout.unwrap_or(CHANNEL_LAYOUT_STEREO);

            LibAV::init();

            let mut codec_context = avcodec_alloc_context3(self.codec.as_ptr());
            if codec_context.is_null() {
                bail!("Could not allocate an encoding context");
            }

            // Initialize encoder fields
            common::encoder::init(codec_context, format);
            (*codec_context).sample_rate = sample_rate;
            (*codec_context).sample_fmt = sample_format;
            (*codec_context).time_base = AVRational { num: 1, den: sample_rate };
            (*codec_context).channel_layout = channel_layout.bits();
            (*codec_context).channels = av_get_channel_layout_nb_channels(channel_layout.bits());

            common::encoder::open(codec_context, "audio")?;

            Ok(Encoder {
                ptr: codec_context,
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

