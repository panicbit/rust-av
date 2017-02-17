use frame::{
    VideoFrame,
    AudioFrame,
};

pub enum RefMutFrame<'a> {
    Video(&'a mut VideoFrame),
    Audio(&'a mut AudioFrame),
}

impl<'a> RefMutFrame<'a> {
    pub fn into_video_frame(self) -> Option<&'a mut VideoFrame> {
        match self {
            RefMutFrame::Video(frame) => Some(frame),
            _ => None,
        }
    }

    pub fn into_audio_frame(self) -> Option<&'a mut AudioFrame> {
        match self {
            RefMutFrame::Audio(frame) => Some(frame),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut VideoFrame> for RefMutFrame<'a> {
    fn from(frame: &'a mut VideoFrame) -> Self {
        RefMutFrame::Video(frame)
    }
}

impl<'a> From<&'a mut AudioFrame> for RefMutFrame<'a> {
    fn from(frame: &'a mut AudioFrame) -> Self {
        RefMutFrame::Audio(frame)
    }
}
