use codec::{Codec, VideoEncoder};
use frame::RefMutFrame;
use ffi::{
    AVCodecContext,
    AVPacket,
    AVRational,
};

pub enum Encoder {
    Video(VideoEncoder),
    // Audio(AudioEncoder),
}

impl Encoder {
    pub fn into_video_encoder(self) -> Option<VideoEncoder> {
        match self {
            Encoder::Video(encoder) => Some(encoder),
            // _ => None
        }
    }

    pub fn as_video_encoder(&self) -> Option<&VideoEncoder> {
        match *self {
            Encoder::Video(ref encoder) => Some(encoder),
            // _ => None
        }
    }

    pub fn codec(&self) -> Codec {
        match *self {
            Encoder::Video(ref encoder) => encoder.codec(),
            // Encoder::Audio(ref mut encoder) => encoder.as_mut_ptr(),
        }
    }

    pub fn time_base(&self) -> AVRational {
        match *self {
            Encoder::Video(ref encoder) => encoder.time_base(),
            // Encoder::Audio(ref mut encoder) => encoder.time_base(),
        }

    }

    pub unsafe fn send_frame<'a, F, H>(&mut self, frame: F, packet_handler: H) -> Result<(), String> where
        F: Into<RefMutFrame<'a>>,
        H: FnMut(&mut AVPacket) -> Result<(), String>,
    {
        match *self {
            Encoder::Video(ref mut encoder) => encoder.send_frame(frame, packet_handler),
            // Encoder::Audio(ref mut encoder) => encoder.send_frame(frame, packet_handler),
        }
    }
}

impl Encoder {
    pub fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        match *self {
            Encoder::Video(ref mut encoder) => encoder.as_mut_ptr(),
            // Encoder::Audio(ref mut encoder) => encoder.as_mut_ptr(),
        }
    }

    pub fn as_ref(&self) -> &AVCodecContext {
        match *self {
            Encoder::Video(ref encoder) => encoder.as_ref(),
            // Encoder::Audio(ref encoder) => encoder.as_ref(),
        }
    }

    pub fn as_mut(&mut self) -> &mut AVCodecContext {
        match *self {
            Encoder::Video(ref mut encoder) => encoder.as_mut(),
            // Encoder::Audio(ref mut encoder) => encoder.as_mut(),
        }
    }
}

impl From<VideoEncoder> for Encoder {
    fn from(encoder: VideoEncoder) -> Self {
        Encoder::Video(encoder)
    }
}

// impl From<AudioEncoder> for Encoder {
//     fn from(encoder: AudioEncoder) -> Self {
//         Encoder::Audio(encoder)
//     }
// }
