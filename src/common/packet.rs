use std::marker::PhantomData;
use std::slice;
use std::mem;
use ffi::{self, AVPacket};
use common::Timebase;

/// A reference to a packet as returned
/// e.g. by Demuxer::read_packet
pub struct Packet<'buf> {
    ptr: *mut AVPacket,
    time_base: Timebase,
    _phantom: PhantomData<&'buf AVPacket>,
}

// See https://github.com/panicbit/rust-av/issues/28
unsafe impl<'buf> Send for Packet<'buf> {}
unsafe impl<'buf> Sync for Packet<'buf> {}

impl<'buf> Packet<'buf> {
    pub unsafe fn from_ptr(ptr: *mut AVPacket, time_base: Timebase) -> Packet<'buf> {
        Packet {
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

    pub fn is_rc(&self) -> bool {
        !self.as_raw().buf.is_null()
    }

    pub fn into_rc(mut self) -> Packet<'static> {
        unsafe {
            // Replace internal packet with a refcounted copy
            if !self.is_rc() {
                let rc_packet = Self::ref_packet(self.ptr);
                ffi::av_packet_unref(self.ptr);
                self.ptr = rc_packet;
            }
            
            // Transmute self to fix the lifetime
            mem::transmute(self)
        }
    }
}

// Private helpers
impl<'buf> Packet<'buf> {
    unsafe fn ref_packet(ptr: *const AVPacket) -> *mut AVPacket {
        let packet = ffi::av_packet_alloc();
        if packet.is_null() {
            panic!("av_packet_alloc: out of memory!");
        }

        // bump ref
        {
            let res = ffi::av_packet_ref(packet, ptr);
            if res < 0 {
                panic!("av_packet_ref: 0x{:X}", res);
            }
        }

        packet
    }

    unsafe fn copy_packet(ptr: *const AVPacket) -> *mut AVPacket {
        let packet = ffi::av_packet_alloc();
        if packet.is_null() {
            panic!("av_packet_alloc: out of memory!");
        }

        *packet = *ptr;

        packet
    }
}

impl<'buf> Clone for Packet<'buf> {
    fn clone(&self) -> Self {
        unsafe {
            let packet = if self.is_rc() {
                Self::ref_packet(self.ptr)
            } else {
                Self::copy_packet(self.ptr)
            };
            
            Self::from_ptr(packet, self.time_base)
        }
    }
}

impl<'buf> Packet<'buf> {
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

impl<'buf> Drop for Packet<'buf> {
    fn drop(&mut self) {
        unsafe {
            ffi::av_packet_unref(self.ptr);
        }
    }
}
