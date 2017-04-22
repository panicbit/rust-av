use std::marker::PhantomData;
use std::mem;
use std::slice;
use ffi::{self, AVPacket};
use common::Timebase;

/// A reference to a packet as returned
/// e.g. by Demuxer::read_packet
pub struct RefPacket<'packet> {
    ptr: *mut AVPacket,
    time_base: Timebase,
    _phantom: PhantomData<&'packet AVPacket>,
}

impl<'packet> RefPacket<'packet> {
    pub unsafe fn from_ptr(ptr: *mut AVPacket, time_base: Timebase) -> RefPacket<'packet> {
        RefPacket {
            ptr: ptr,
            time_base: time_base,
            _phantom: PhantomData,
        }
    }

    pub fn stream_index(&self) -> usize {
        self.as_raw().stream_index as usize
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let packet = self.as_raw();
            slice::from_raw_parts(packet.data, packet.size as usize)
        }
    }

    pub fn time_base(&self) -> Timebase {
        self.time_base
    }
}

impl<'packet> RefPacket<'packet> {
    pub fn as_raw(&self) -> &AVPacket {
        unsafe { &*self.ptr }
    }
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
