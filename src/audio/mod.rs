
pub mod constants;
pub use self::constants::ChannelLayout;

mod encoder;
pub use self::encoder::{
    Encoder,
    EncoderBuilder,
};

mod frame;
pub use self::frame::Frame;
