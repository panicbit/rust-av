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
use common::stream::Stream;
use common::RcPacket;
use codec::MediaType;
use super::Frame;

pub enum Decoder {
    Video(video::Decoder),
    Audio(audio::Decoder),
}

impl Decoder {
    pub fn from_stream(stream: &Stream) -> Result<Self> {
        Ok(match stream.codec_parameters().media_type() {
            MediaType::Video => video::Decoder::from_stream(stream)?.into(),
            MediaType::Audio => audio::Decoder::from_stream(stream)?.into(),
            other => bail!("Unsupported media type: {:?}", other)

        })
    }

    pub fn into_video_encoder(self) -> Option<video::Decoder> {
        match self {
            Decoder::Video(decoder) => Some(decoder),
            _ => None
        }
    }

    pub fn as_video_encoder(&self) -> Option<&video::Decoder> {
        match *self {
            Decoder::Video(ref decoder) => Some(decoder),
            _ => None
        }
    }

    pub fn as_mut_video_encoder(&mut self) -> Option<&mut video::Decoder> {
        match *self {
            Decoder::Video(ref mut decoder) => Some(decoder),
            _ => None
        }
    }

    pub fn into_audio_encoder(self) -> Option<audio::Decoder> {
        match self {
            Decoder::Audio(decoder) => Some(decoder),
            _ => None
        }
    }

    pub fn as_audio_encoder(&self) -> Option<&audio::Decoder> {
        match *self {
            Decoder::Audio(ref decoder) => Some(decoder),
            _ => None
        }
    }

    pub fn as_mut_audio_encoder(&mut self) -> Option<&mut audio::Decoder> {
        match *self {
            Decoder::Audio(ref mut decoder) => Some(decoder),
            _ => None
        }
    }

    pub fn codec(&self) -> Codec {
        match *self {
            Decoder::Video(ref decoder) => decoder.codec(),
            Decoder::Audio(ref decoder) => decoder.codec(),
        }
    }

    pub fn time_base(&self) -> AVRational {
        match *self {
            Decoder::Video(ref decoder) => decoder.time_base(),
            Decoder::Audio(ref decoder) => decoder.time_base(),
        }
    }
}

impl Decoder {
    pub fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        match *self {
            Decoder::Video(ref mut decoder) => decoder.as_mut_ptr(),
            Decoder::Audio(ref mut decoder) => decoder.as_mut_ptr(),
        }
    }

    pub fn as_ref(&self) -> &AVCodecContext {
        match *self {
            Decoder::Video(ref decoder) => decoder.as_ref(),
            Decoder::Audio(ref decoder) => decoder.as_ref(),
        }
    }

    pub fn as_mut(&mut self) -> &mut AVCodecContext {
        match *self {
            Decoder::Video(ref mut decoder) => decoder.as_mut(),
            Decoder::Audio(ref mut decoder) => decoder.as_mut(),
        }
    }

    pub fn decode<'decoder>(&'decoder mut self, packet: &RcPacket) -> Result<Frames<'decoder>> {
        match *self {
            Decoder::Video(ref mut decoder) => decoder.decode(packet).map(Frames::from),
            Decoder::Audio(ref mut decoder) => decoder.decode(packet).map(Frames::from),
        }
    }

    pub fn flush<'decoder>(&'decoder mut self) -> Result<Frames<'decoder>> {
        match *self {
            Decoder::Video(ref mut decoder) => decoder.flush().map(Frames::from),
            Decoder::Audio(ref mut decoder) => decoder.flush().map(Frames::from),
        }
    }
}

impl From<video::Decoder> for Decoder {
    fn from(decoder: video::Decoder) -> Self {
        Decoder::Video(decoder)
    }
}

impl From<audio::Decoder> for Decoder {
    fn from(decoder: audio::Decoder) -> Self {
        Decoder::Audio(decoder)
    }
}

pub enum Frames<'decoder> {
    Video(video::Frames<'decoder>),
    Audio(audio::Frames<'decoder>),
}

impl<'decoder> Iterator for Frames<'decoder> {
    type Item = Result<Frame>;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Frames::Video(ref mut frames) => frames.next().map(|res| res.map(Frame::from)),
            Frames::Audio(ref mut frames) => frames.next().map(|res| res.map(Frame::from)),
        }
    }
}

impl<'decoder> From<video::Frames<'decoder>> for Frames<'decoder> {
    fn from(frames: video::Frames<'decoder>) -> Self {
        Frames::Video(frames)
    }
}

impl<'decoder> From<audio::Frames<'decoder>> for Frames<'decoder> {
    fn from(frames: audio::Frames<'decoder>) -> Self {
        Frames::Audio(frames)
    }
}
