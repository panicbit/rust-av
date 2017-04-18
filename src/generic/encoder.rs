use codec::Codec;
use generic::RefMutFrame;
use ffi::{
    AVCodecContext,
    AVPacket,
    AVRational,
};
use video;
use audio;
use errors::*;
use common::RcPacket;

pub enum Encoder {
    Video(video::Encoder),
    Audio(audio::Encoder),
}

impl Encoder {
    pub fn into_video_encoder(self) -> Option<video::Encoder> {
        match self {
            Encoder::Video(encoder) => Some(encoder),
            _ => None
        }
    }

    pub fn as_video_encoder(&self) -> Option<&video::Encoder> {
        match *self {
            Encoder::Video(ref encoder) => Some(encoder),
            _ => None
        }
    }

    pub fn as_mut_video_encoder(&mut self) -> Option<&mut video::Encoder> {
        match *self {
            Encoder::Video(ref mut encoder) => Some(encoder),
            _ => None
        }
    }

    pub fn into_audio_encoder(self) -> Option<audio::Encoder> {
        match self {
            Encoder::Audio(encoder) => Some(encoder),
            _ => None
        }
    }

    pub fn as_audio_encoder(&self) -> Option<&audio::Encoder> {
        match *self {
            Encoder::Audio(ref encoder) => Some(encoder),
            _ => None
        }
    }

    pub fn as_mut_audio_encoder(&mut self) -> Option<&mut audio::Encoder> {
        match *self {
            Encoder::Audio(ref mut encoder) => Some(encoder),
            _ => None
        }
    }

    pub fn codec(&self) -> Codec {
        match *self {
            Encoder::Video(ref encoder) => encoder.codec(),
            Encoder::Audio(ref encoder) => encoder.codec(),
        }
    }

    pub fn time_base(&self) -> AVRational {
        match *self {
            Encoder::Video(ref encoder) => encoder.time_base(),
            Encoder::Audio(ref encoder) => encoder.time_base(),
        }
    }

    pub fn encode<'a, F>(&mut self, frame: F) -> Result<Packets> where
        F: Into<RefMutFrame<'a>>,
    {
        match *self {
            Encoder::Video(ref mut encoder) => encoder.encode(frame).map(Packets::from),
            Encoder::Audio(ref mut encoder) => encoder.encode(frame).map(Packets::from),
        }
    }

    pub fn flush<'a, F>(self) -> Result<Packets<'static>> where
        F: Into<RefMutFrame<'a>>,
    {
        match self {
            Encoder::Video(encoder) => encoder.flush().map(Packets::from),
            Encoder::Audio(encoder) => encoder.flush().map(Packets::from),
        }
    }
}

impl Encoder {
    pub fn as_ptr(&self) -> *const AVCodecContext {
        match *self {
            Encoder::Video(ref encoder) => encoder.as_ptr(),
            Encoder::Audio(ref encoder) => encoder.as_ptr(),
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        match *self {
            Encoder::Video(ref mut encoder) => encoder.as_mut_ptr(),
            Encoder::Audio(ref mut encoder) => encoder.as_mut_ptr(),
        }
    }

    pub fn as_mut(&mut self) -> &mut AVCodecContext {
        match *self {
            Encoder::Video(ref mut encoder) => encoder.as_mut(),
            Encoder::Audio(ref mut encoder) => encoder.as_mut(),
        }
    }
}

impl AsRef<AVCodecContext> for Encoder {
    fn as_ref(&self) -> &AVCodecContext {
        match *self {
            Encoder::Video(ref encoder) => encoder.as_ref(),
            Encoder::Audio(ref encoder) => encoder.as_ref(),
        }
    }
}

impl From<video::Encoder> for Encoder {
    fn from(encoder: video::Encoder) -> Self {
        Encoder::Video(encoder)
    }
}

impl From<audio::Encoder> for Encoder {
    fn from(encoder: audio::Encoder) -> Self {
        Encoder::Audio(encoder)
    }
}

pub enum Packets<'encoder> {
    Video(video::Packets<'encoder>),
    Audio(audio::Packets<'encoder>),
}

impl<'encoder> Iterator for Packets<'encoder> {
    type Item = Result<RcPacket>;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Packets::Video(ref mut packets) => packets.next(),
            Packets::Audio(ref mut packets) => packets.next(),
        }
    }
}

impl<'encoder> From<video::Packets<'encoder>> for Packets<'encoder> {
    fn from(packets: video::Packets<'encoder>) -> Self {
        Packets::Video(packets)
    }
}

impl<'decoder> From<audio::Packets<'decoder>> for Packets<'decoder> {
    fn from(packets: audio::Packets<'decoder>) -> Self {
        Packets::Audio(packets)
    }
}
