use std::marker::PhantomData;
use ffi::{self, AVPacket};
use std::mem;

/// A reference to a packet as returned
/// e.g. by Demuxer::read_packet
pub struct RefPacket<'packet> {
    ptr: *mut AVPacket,
    _phantom: PhantomData<&'packet AVPacket>,
}

impl<'packet> RefPacket<'packet> {
    pub unsafe fn from_ptr(ptr: *mut AVPacket) -> RefPacket<'packet> {
        RefPacket {
            ptr: ptr,
            _phantom: PhantomData,
        }
    }
}

impl<'packet> RefPacket<'packet> {
    pub fn as_ptr(&self) -> *const AVPacket {
        self.ptr
    }
    pub fn as_mut_ptr(&mut self) -> *mut AVPacket {
        self.ptr
    }
}

impl<'packet> Drop for RefPacket<'packet> {
    fn drop(&mut self) {
        unsafe {
            ffi::av_packet_unref(self.ptr);
        }
    }
}
