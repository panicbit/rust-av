use ffi::{AVPacket, AVRational};
use super::*;

/// A copy-on-write packet
pub enum CowPacket<'packet> {
    Ref(RefPacket<'packet>),
    Rc(RcPacket),
}

impl<'packet> CowPacket<'packet> {
    pub unsafe fn from_ptr(ptr: *mut AVPacket, time_base: AVRational) -> CowPacket<'packet> {
        let is_ref = (*ptr).buf.is_null();
        if is_ref {
            CowPacket::Ref(RefPacket::from_ptr(ptr, time_base))
        } else {
            CowPacket::Rc(RcPacket::from_ptr(ptr, time_base))
        }
    }

    pub fn into_ref(self) -> RefPacket<'packet> {
        match self {
            CowPacket::Ref(packet) => packet,
            CowPacket::Rc(packet) => packet.into_ref(),
        }
    }

    pub fn into_rc(self) -> RcPacket {
        unsafe {
            match self {
                CowPacket::Ref(packet) => RcPacket::ref_ptr(packet.as_ptr(), packet.time_base()),
                CowPacket::Rc(packet) => packet,
            }
        }
    }

    pub fn stream_index(&self) -> usize {
        match *self {
            CowPacket::Ref(ref packet) => packet.stream_index(),
            CowPacket::Rc(ref packet) => packet.stream_index(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        match *self {
            CowPacket::Ref(ref packet) => packet.as_slice(),
            CowPacket::Rc(ref packet) => packet.as_slice(),
        }
    }

    pub fn time_base(&self) -> AVRational {
        match *self {
            CowPacket::Ref(ref packet) => packet.time_base(),
            CowPacket::Rc(ref packet) => packet.time_base(),
        }
    }
}

impl<'packet> From<RefPacket<'packet>> for CowPacket<'packet> {
    fn from(packet: RefPacket<'packet>) -> Self {
        CowPacket::Ref(packet)
    }
}

impl From<RcPacket> for CowPacket<'static> {
    fn from(packet: RcPacket) -> Self {
        CowPacket::Rc(packet)
    }
}
