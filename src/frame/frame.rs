use frame::VideoFrame;

pub enum RefMutFrame<'a> {
    Video(&'a mut VideoFrame),
    // Audio(AudioFrame),
}

impl<'a> RefMutFrame<'a> {
    pub fn into_video_frame(self) -> Option<&'a mut VideoFrame> {
        match self {
            RefMutFrame::Video(frame) => Some(frame),
            // _ => None,
        }
    }
}

impl<'a> From<&'a mut VideoFrame> for RefMutFrame<'a> {
    fn from(frame: &'a mut VideoFrame) -> Self {
        RefMutFrame::Video(frame)
    }
}
