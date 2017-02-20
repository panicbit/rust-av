use std::ptr;
use libc::int32_t;
use ffi::{
    AVCodecContext,
    avcodec_open2,
    avcodec_free_context,
    AVFMT_GLOBALHEADER,
    AV_CODEC_FLAG_GLOBAL_HEADER,
};
use format::OutputFormat;
use codec::{
    MediaType,
    Codec,
};

pub unsafe fn init(codec_context: *mut AVCodecContext, format: OutputFormat) {
    // Some formats require global headers
    if 0 != (format.as_ref().flags & AVFMT_GLOBALHEADER as int32_t) {
        (*codec_context).flags |= AV_CODEC_FLAG_GLOBAL_HEADER as int32_t;
    }
}

pub unsafe fn open(mut codec_context: *mut AVCodecContext, kind: &'static str) -> Result<(), String> {
    let codec = (*codec_context).codec;
    let options = ptr::null_mut();
    let res = avcodec_open2(codec_context, codec, options);
    if res < 0 {
        avcodec_free_context(&mut codec_context);
        return Err(format!("Failed to open {} encoder ({})", kind, res))
    }

    Ok(())
}

pub fn require_is_encoder(codec: Codec) -> Result<(), String> {
    if !codec.is_encoder() {
        Err(format!("{:?} does not support encoding", codec.id()))
    } else {
        Ok(())
    }
}

pub fn require_codec_type(codec: Codec, required_type: MediaType) -> Result<(), String> {
    if codec.media_type() != required_type {
        Err(format!("{:?} encoder got {:?} codec", required_type, codec.media_type()))
    } else {
        Ok(())
    }
}
