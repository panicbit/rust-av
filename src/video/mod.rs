
mod encoder;
pub use self::encoder::{
    Encoder,
    EncoderBuilder,
    Packets,
};

mod decoder;
pub use self::decoder::{
    Decoder,
    Frames,
};

mod frame;
pub use self::frame::Frame;
pub const MAX_PLANES: usize = 4;
