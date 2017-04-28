use std::ptr;
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
use errors::*;

pub unsafe fn init(codec_context: *mut AVCodecContext, format: OutputFormat) {
    // Some formats require global headers
    if 0 != (format.as_ref().flags & AVFMT_GLOBALHEADER as i32) {
        (*codec_context).flags |= AV_CODEC_FLAG_GLOBAL_HEADER as i32;
    }
}

pub unsafe fn open(mut codec_context: *mut AVCodecContext, kind: &'static str) -> Result<()> {
    let codec = (*codec_context).codec;
    let options = ptr::null_mut();
    let res = avcodec_open2(codec_context, codec, options);
    if res < 0 {
        avcodec_free_context(&mut codec_context);
        bail!(ErrorKind::OpenEncoder(kind));
    }

    Ok(())
}

pub fn require_is_encoder(codec: Codec) -> Result<()> {
    if !codec.is_encoder() {
        Err(ErrorKind::EncodingUnsupported(codec.id()).into())
    } else {
        Ok(())
    }
}

pub fn require_codec_type(encoder_type: MediaType, codec: Codec) -> Result<()> {
    if encoder_type != codec.media_type() {
        Err(ErrorKind::MediaTypeMismatch(encoder_type, codec.id()).into())
    } else {
        Ok(())
    }
}
