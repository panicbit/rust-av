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
};
use super::{
    Descriptor,
    DescriptorIter,
};

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

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_ref().name) }
    }

    pub fn long_name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_ref().name) }
    }

    pub fn media_type(&self) -> AVMediaType {
        self.as_ref().type_
    }

    pub fn id(&self) -> AVCodecID {
        self.as_ref().id
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
            .field("name", &self.name())
            .field("long_name", &self.long_name())
            .field("media_type", &self.media_type())
            .field("id", &self.id())
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
