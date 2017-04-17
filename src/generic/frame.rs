use video;
use audio;
use errors::*;

pub enum Frame {
    Video(video::Frame),
    Audio(audio::Frame),
}

impl Frame {
    pub fn into_video_frame(self) -> Option<video::Frame> {
        match self {
            Frame::Video(frame) => Some(frame),
            _ => None,
        }
    }

    pub fn into_audio_frame(self) -> Option<audio::Frame> {
        match self {
            Frame::Audio(frame) => Some(frame),
            _ => None,
        }
    }
}

impl From<video::Frame> for Frame {
    fn from(frame: video::Frame) -> Self {
        Frame::Video(frame)
    }
}

impl From<audio::Frame> for Frame {
    fn from(frame: audio::Frame) -> Self {
        Frame::Audio(frame)
    }
}
