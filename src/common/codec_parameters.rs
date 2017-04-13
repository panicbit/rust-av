use std::marker::PhantomData;
use ffi::{self, AVCodecParameters, AVStream};

pub struct CodecParameters<'stream> {
    ptr: *mut AVCodecParameters,
    _phantom: PhantomData<&'stream AVStream>,
}

impl<'stream> CodecParameters<'stream> {
    pub unsafe fn from_ptr(ptr: *mut AVCodecParameters) -> CodecParameters<'stream> {
        CodecParameters {
            ptr: ptr,
            _phantom: PhantomData,
        }
    }

    pub fn codec_type(&self) -> ffi::AVMediaType {
        self.as_ref().codec_type
    }

    pub fn codec_id(&self) -> ffi::AVCodecID {
        self.as_ref().codec_id
    }

    // TODO: remaining fields
}

impl<'stream> CodecParameters<'stream> {
    pub fn as_ref(&self) -> &AVCodecParameters {
        unsafe { &*self.ptr }
    }
    pub fn as_mut(&self) -> &mut AVCodecParameters {
        unsafe { &mut *self.ptr }
    }
    pub fn as_ptr(&self) -> *const AVCodecParameters {
        self.ptr
    }
    pub fn as_mut_ptr(&mut self) -> *mut AVCodecParameters {
        self.ptr
    }
}
