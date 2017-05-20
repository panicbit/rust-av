use std::ptr;
use ffi;
use ffi::AVCodecContext;
use codec::{Codec,MediaType};
use common::codec_parameters::CodecParameters;
use common::stream::Stream;
use common::{Packet, Timebase};
use super::Frame;
use errors::*;

pub struct Decoder {
    ptr: *mut AVCodecContext,
}

unsafe impl Send for Decoder{};
unsafe impl Sync for Deocder{};

impl Decoder {
    // TODO: Share code between audio/video
    pub fn from_codec_parameters<'fmt_ctx>(codec_parameters: CodecParameters<'fmt_ctx>) -> Result<Self> {
        unsafe {
            let codec_id = codec_parameters.codec_id();

            // Try to find a suitable codec
            let codec = Codec::find_decoder_by_id(codec_id)?;
            if !codec.media_type().is_video() {
                bail!(ErrorKind::MediaTypeMismatch(MediaType::Video, codec_id))
            }

            // Try to allocate the decoder
            let mut codec_context = ffi::avcodec_alloc_context3(codec.as_ptr());
            if codec_context.is_null() {
                bail!("Could not allocate video decoder");
            }

            // Copy codec parameters to codec_parameters
            {
                let res = ffi::avcodec_parameters_to_context(codec_context, codec_parameters.as_ptr());
                if res < 0 {
                    ffi::avcodec_free_context(&mut codec_context);
                    bail!(ErrorKind::CopyCodecParameters);
                }
            }

            // Try to open the decoder
            {
                let options = ptr::null_mut();
                let res = ffi::avcodec_open2(codec_context, codec.as_ptr(), options);
                if res < 0 {
                    ffi::avcodec_free_context(&mut codec_context);
                    bail!(ErrorKind::OpenDecoder("video"));
                }
            }

            Ok(Decoder {
                ptr: codec_context,
            })
        }
    }

    pub fn from_stream(stream: &Stream) -> Result<Self> {
        Self::from_codec_parameters(stream.codec_parameters())
    }

    pub fn codec(&self) -> Codec {
        unsafe {
            Codec::from_ptr(self.as_ref().codec)
        }
    }

    pub fn time_base(&self) -> Timebase {
        self.as_ref().time_base.into()
    }

    pub fn pixel_format(&self) -> ffi::AVPixelFormat {
        self.as_ref().pix_fmt
    }

    pub fn decode<'decoder>(&'decoder mut self, mut packet: Packet) -> Result<Frames<'decoder>> {
        // TODO: Check that pkt->data is AV_INPUT_BUFFER_PADDING_SIZE larger than packet size

        unsafe {
            let res = ffi::avcodec_send_packet(self.as_mut_ptr(), packet.as_mut_ptr());
            if res < 0 {
                match res {
                    ffi::AVERROR_EAGAIN => bail!("EAGAIN in Decoder::decode. This is not supposed to happen :("),
                    _ => bail!(format!("Failed to decode packet: 0x{:X}", res))
                }
            }

            Ok(Frames::from_decoder(self))
        }
    }

    pub fn flush<'decoder>(&'decoder mut self) -> Result<Frames<'decoder>> {
        // TODO: Check that pkt->data is AV_INPUT_BUFFER_PADDING_SIZE larger than packet size

        unsafe {
            let res = ffi::avcodec_send_packet(self.as_mut_ptr(), ptr::null_mut());

            if res < 0 && res != ffi::AVERROR_EAGAIN {
                bail!(format!("Failed to flush decoder: 0x{:X}", res))
            }

            Ok(Frames::from_decoder(self))
        }
    }
}

impl Decoder {
    pub fn as_ref(&self) -> &AVCodecContext { unsafe { &*self.ptr } }
    pub fn as_mut(&mut self) -> &mut AVCodecContext { unsafe { &mut *self.ptr } }
    pub fn as_ptr(&self) -> *const AVCodecContext { self.ptr }
    pub fn as_mut_ptr(&mut self) -> *mut AVCodecContext { self.ptr }
}

pub struct Frames<'decoder> {
    decoder: &'decoder mut Decoder,
}

impl<'decoder> Frames<'decoder> {
    fn from_decoder(decoder: &'decoder mut Decoder) -> Self {
        Frames {
            decoder: decoder
        }
    }
}

impl<'decoder> Iterator for Frames<'decoder> {
    type Item = Result<Frame>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut frame = ffi::av_frame_alloc();
            let res = ffi::avcodec_receive_frame(self.decoder.as_mut_ptr(), frame);

            if res < 0 {
                ffi::av_frame_free(&mut frame);

                match res {
                    ffi::AVERROR_EAGAIN | ffi::AVERROR_EOF => return None,
                    _ => return Some(Err(format!("Failed to receive frame: 0x{:X}", res).into())),
                }
            }

            let pixel_format = self.decoder.pixel_format();
            let frame = Frame::from_ptr(frame, pixel_format);

            Some(Ok(frame))
        }
    }
}

impl<'decoder> Drop for Frames<'decoder> {
    fn drop(&mut self) {
        // Decode every frame possible
        for _ in self {}
    }
}
