use std::slice;
use ffi::{self, AVPacket};
use super::*;

/// A reference-counted packet
pub struct RcPacket {
    ptr: *mut AVPacket,
}

impl RcPacket {
    pub unsafe fn from_ptr(ptr: *mut AVPacket) -> RcPacket {
        RcPacket {
            ptr: ptr,
        }
    }

    pub unsafe fn ref_ptr(ptr: *const AVPacket) -> RcPacket {
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

        RcPacket::from_ptr(packet)
    }

    pub fn into_ref(self) -> RefPacket<'static> {
        unsafe {
            RefPacket::from_ptr(self.ptr)
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
}

impl RcPacket {
    pub fn as_raw(&self) -> &AVPacket {
        unsafe { &*self.ptr }
    }
}

impl Clone for RcPacket {
    fn clone(&self) -> Self {
        unsafe {
            RcPacket::ref_ptr(self.ptr)
        }
    }
}

impl Drop for RcPacket {
    fn drop(&mut self) {
        unsafe {
            ffi::av_packet_free(&mut self.ptr);
        }
    }
}
