use codec::Codec;
use generic::RefMutFrame;
use ffi::{
    AVCodecContext,
    AVPacket,
    AVRational,
};
use video;
use audio;

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

    pub unsafe fn send_frame<'a, F, H>(&mut self, frame: F, packet_handler: H) -> Result<(), String> where
        F: Into<RefMutFrame<'a>>,
        H: FnMut(&mut AVPacket) -> Result<(), String>,
    {
        match *self {
            Encoder::Video(ref mut encoder) => encoder.send_frame(frame, packet_handler),
            Encoder::Audio(ref mut encoder) => encoder.send_frame(frame, packet_handler),
        }
    }
}

impl Encoder {
    pub fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        match *self {
            Encoder::Video(ref mut encoder) => encoder.as_mut_ptr(),
            Encoder::Audio(ref mut encoder) => encoder.as_mut_ptr(),
        }
    }

    pub fn as_ref(&self) -> &AVCodecContext {
        match *self {
            Encoder::Video(ref encoder) => encoder.as_ref(),
            Encoder::Audio(ref encoder) => encoder.as_ref(),
        }
    }

    pub fn as_mut(&mut self) -> &mut AVCodecContext {
        match *self {
            Encoder::Video(ref mut encoder) => encoder.as_mut(),
            Encoder::Audio(ref mut encoder) => encoder.as_mut(),
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
