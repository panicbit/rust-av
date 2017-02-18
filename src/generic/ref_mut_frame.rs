use video;
use audio;

pub enum RefMutFrame<'a> {
    Video(&'a mut video::Frame),
    Audio(&'a mut audio::Frame),
}

impl<'a> RefMutFrame<'a> {
    pub fn into_video_frame(self) -> Option<&'a mut video::Frame> {
        match self {
            RefMutFrame::Video(frame) => Some(frame),
            _ => None,
        }
    }

    pub fn into_audio_frame(self) -> Option<&'a mut audio::Frame> {
        match self {
            RefMutFrame::Audio(frame) => Some(frame),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut video::Frame> for RefMutFrame<'a> {
    fn from(frame: &'a mut video::Frame) -> Self {
        RefMutFrame::Video(frame)
    }
}

impl<'a> From<&'a mut audio::Frame> for RefMutFrame<'a> {
    fn from(frame: &'a mut audio::Frame) -> Self {
        RefMutFrame::Audio(frame)
    }
}
