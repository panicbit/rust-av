use std::ffi::CStr;
use std::fmt;
use std::ptr;
use ffi::{
    AVCodecID,
    AVCodecDescriptor,
    AVMediaType,
    avcodec_descriptor_get,
    avcodec_descriptor_next,
};
use super::ProfileIter;
use super::MimeTypeIter;

#[derive(PartialEq)]
pub struct Descriptor {
    ptr: *const AVCodecDescriptor
}

impl Descriptor {
    pub fn from_codec_id(codec_id: AVCodecID) -> Option<Self> {
        unsafe {
            let descriptor = avcodec_descriptor_get(codec_id);;
            if descriptor.is_null() {
                None
            } else {
                Some(Self::from_ptr(descriptor))
            }
        }
    }

    pub fn as_ref(&self) -> &AVCodecDescriptor {
        unsafe { &*self.ptr }
    }

    pub fn id(&self) -> AVCodecID {
        self.as_ref().id
    }

    pub fn media_type(&self) -> AVMediaType {
        self.as_ref().type_
    }

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_ref().name) }
    }

    pub fn long_name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_ref().long_name) }
    }

    pub fn mime_types(&self) -> MimeTypeIter {
        unsafe { MimeTypeIter::from_ptr(self.as_ref().mime_types) }
    }

    pub fn profiles(&self) -> ProfileIter {
        unsafe { ProfileIter::from_ptr(self.as_ref().profiles) }
    }
}

impl Descriptor {
    pub unsafe fn from_ptr(descriptor: *const AVCodecDescriptor) -> Self {
        Descriptor { ptr: descriptor }
    }

}

impl fmt::Debug for Descriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CodecDescriptorRef")
            .field("id", &self.id())
            .field("media_type", &self.media_type())
            .field("name", &self.name())
            .field("long_name", &self.long_name())
            .field("profiles", &self.profiles().collect::<Vec<_>>())
            .finish()
    }
}

pub struct DescriptorIter {
    prev: *const AVCodecDescriptor
}

impl DescriptorIter {
    pub fn new() -> Self {
        ::LibAV::init();
        DescriptorIter { prev: ptr::null() }
    }
}

impl Iterator for DescriptorIter {
    type Item = Descriptor;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = avcodec_descriptor_next(self.prev);
            if next.is_null() {
                None
            } else {
                self.prev = next;
                Some(Descriptor::from_ptr(next))
            }
        }
    }
}
