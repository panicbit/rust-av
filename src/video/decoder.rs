use std::ptr;
use ffi;
use ffi::AVCodecContext;
use codec::{Codec,MediaType};
use common::codec_parameters::CodecParameters;
use common::stream::Stream;
use errors::*;

pub struct Decoder {
    ptr: *mut AVCodecContext,
}

impl Decoder {
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

    pub fn time_base(&self) -> ffi::AVRational {
        self.as_ref().time_base
    }
}

impl Decoder {
    pub fn as_ref(&self) -> &AVCodecContext { unsafe { &*self.ptr } }
    pub fn as_mut(&mut self) -> &mut AVCodecContext { unsafe { &mut *self.ptr } }
    pub fn as_ptr(&self) -> *const AVCodecContext { self.ptr }
    pub fn as_mut_ptr(&mut self) -> *mut AVCodecContext { self.ptr }
}
