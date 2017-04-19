use std::slice;
use ffi::{self, AVPacket, AVRational};
use super::*;

/// A reference-counted packet
pub struct RcPacket {
    ptr: *mut AVPacket,
    time_base: AVRational,
}

impl RcPacket {
    pub unsafe fn from_ptr(ptr: *mut AVPacket, time_base: AVRational) -> RcPacket {
        RcPacket {
            ptr: ptr,
            time_base: time_base,
        }
    }

    pub unsafe fn ref_ptr(ptr: *const AVPacket, time_base: AVRational) -> RcPacket {
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

        RcPacket::from_ptr(packet, time_base)
    }

    pub fn into_ref(self) -> RefPacket<'static> {
        unsafe {
            RefPacket::from_ptr(self.ptr, self.time_base)
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

    pub fn time_base(&self) -> AVRational {
        self.time_base
    }
}

impl RcPacket {
    pub fn as_raw(&self) -> &AVPacket {
        unsafe { &*self.ptr }
    }

    pub fn as_mut_ptr(&self) -> *mut AVPacket {
        self.ptr
    }
}

impl Clone for RcPacket {
    fn clone(&self) -> Self {
        unsafe {
            RcPacket::ref_ptr(self.ptr, self.time_base())
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
