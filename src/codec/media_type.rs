use ffi::AVMediaType;

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum MediaType {
    /// Usually treated as `Data`
    Unknown,
    Video,
    Audio,
    /// Opaque data information usually continuous
    Data,
    Subtitle,
    /// Opaque data information usually sparse
    Attachment,
    NB,
}

impl MediaType {
    pub fn from_raw(kind: AVMediaType) -> Self {
        match kind {
            AVMediaType::AVMEDIA_TYPE_UNKNOWN    => MediaType::Unknown,
            AVMediaType::AVMEDIA_TYPE_VIDEO      => MediaType::Video,
            AVMediaType::AVMEDIA_TYPE_AUDIO      => MediaType::Audio,
            AVMediaType::AVMEDIA_TYPE_DATA       => MediaType::Data,
            AVMediaType::AVMEDIA_TYPE_SUBTITLE   => MediaType::Subtitle,
            AVMediaType::AVMEDIA_TYPE_ATTACHMENT => MediaType::Attachment,
            AVMediaType::AVMEDIA_TYPE_NB         => MediaType::NB,
        }
    }

    pub fn as_raw(self) -> AVMediaType {
        match self {
            MediaType::Unknown    => AVMediaType::AVMEDIA_TYPE_UNKNOWN,
            MediaType::Video      => AVMediaType::AVMEDIA_TYPE_VIDEO,
            MediaType::Audio      => AVMediaType::AVMEDIA_TYPE_AUDIO,
            MediaType::Data       => AVMediaType::AVMEDIA_TYPE_DATA,
            MediaType::Subtitle   => AVMediaType::AVMEDIA_TYPE_SUBTITLE,
            MediaType::Attachment => AVMediaType::AVMEDIA_TYPE_ATTACHMENT,
            MediaType::NB         => AVMediaType::AVMEDIA_TYPE_NB,
        }
    }

    pub fn is_unknown(self) -> bool {
        match self {
            MediaType::Unknown => true,
            _ => false,
        }
    }
    
    pub fn is_video(self) -> bool {
        match self {
            MediaType::Video => true,
            _ => false,
        }
    }
    
    pub fn is_audio(self) -> bool {
        match self {
            MediaType::Audio => true,
            _ => false,
        }
    }
    
    pub fn is_data(self) -> bool {
        match self {
            MediaType::Data => true,
            _ => false,
        }
    }
    
    pub fn is_subtitle(self) -> bool {
        match self {
            MediaType::Subtitle => true,
            _ => false,
        }
    }
    
    pub fn is_attachment(self) -> bool {
        match self {
            MediaType::Attachment => true,
            _ => false,
        }
    }
    
    pub fn is_nb(self) -> bool {
        match self {
            MediaType::NB => true,
            _ => false,
        }
    }
    
}
