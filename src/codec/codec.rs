use std::fmt;
use std::slice;
use std::ffi::CStr;
use LibAV;
use ffi::{
    AVCodec,
    AVCodecID,
    AVMediaType,
    AVPixelFormat,
    avcodec_find_encoder,
    avcodec_find_decoder,
    av_codec_is_encoder,
    av_codec_is_decoder,
};
use super::{
    Descriptor,
    DescriptorIter,
};
use util::AsCStr;

#[derive(Copy,Clone)]
pub struct Codec {
    ptr: *const AVCodec
}

impl Codec {
    pub fn find_encoder_by_id(codec_id: AVCodecID) -> Result<Self, String> {
        unsafe {
            LibAV::init();
            let codec = avcodec_find_encoder(codec_id);
            if codec.is_null() {
                // maybe use avcodec_get_name(codec_id)
                return Err(format!("Could not find encoder for {:?}", codec_id))
            }
            Ok(Self::from_ptr(codec))
        }
    }

    pub fn find_decoder_by_id(codec_id: AVCodecID) -> Result<Self, String> {
        unsafe {
            LibAV::init();
            let codec = avcodec_find_decoder(codec_id);
            if codec.is_null() {
                // maybe use avcodec_get_name(codec_id)
                return Err(format!("Could not find decoder for {:?}", codec_id))
            }
            Ok(Self::from_ptr(codec))
        }
    }

    pub fn is_encoder(&self) -> bool {
        unsafe { av_codec_is_encoder(self.ptr) != 0 }
    }

    pub fn is_decoder(&self) -> bool {
        unsafe { av_codec_is_decoder(self.ptr) != 0 }
    }

    pub fn id(&self) -> AVCodecID {
        self.as_ref().id
    }

    pub fn name(&self) -> &CStr {
        unsafe { self.as_ref().name.as_cstr().unwrap() }
    }

    pub fn long_name(&self) -> &CStr {
        unsafe { self.as_ref().name.as_cstr().unwrap() }
    }

    pub fn media_type(&self) -> AVMediaType {
        self.as_ref().type_
    }

    pub fn pixel_formats(&self) -> &[AVPixelFormat] {
        unsafe {
            use ffi::AVPixelFormat::AV_PIX_FMT_NONE;

            let pix_fmts = (*self.ptr).pix_fmts;
            let mut len = 0;

            while *pix_fmts.offset(len) != AV_PIX_FMT_NONE {
                len += 1;
            }

            slice::from_raw_parts(pix_fmts, len as usize)
        }
    }

    pub fn descriptors() -> DescriptorIter {
        LibAV::init();
        DescriptorIter::new()
    }
}

impl Codec {
    pub unsafe fn from_ptr(ptr: *const AVCodec) -> Self {
        Codec { ptr: ptr }
    }

    pub fn as_ref(&self) -> &AVCodec {
        unsafe { &*self.ptr }
    }
    
    pub fn as_ptr(&self) -> *const AVCodec {
        self.ptr
    }
}

impl fmt::Debug for Codec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Codec")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("long_name", &self.long_name())
            .field("is_encoder", &self.is_encoder())
            .field("is_decoder", &self.is_decoder())
            .field("media_type", &self.media_type())
            .field("pixel_formats", &self.pixel_formats())
            .finish()
    }
}

pub trait AVCodecIDExt {
    fn descriptor(self) -> Option<Descriptor>;
}

impl AVCodecIDExt for AVCodecID {
    fn descriptor(self) -> Option<Descriptor> {
        Descriptor::from_codec_id(self)
    }
}
