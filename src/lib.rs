extern crate av_sys as ffi;
extern crate libc;
#[macro_use]
extern crate lazy_static;
use std::ffi::CStr;
use std::sync::{Once, ONCE_INIT};

#[macro_use]
mod util;

pub mod format;
pub mod io;
pub mod frame;
pub mod codec;
pub mod scaler;

lazy_static! {
    pub static ref AV: LibAV = LibAV::init();
}

pub struct LibAV(());

impl LibAV {
    pub fn init() -> LibAV {
        unsafe {
            static INIT: Once = ONCE_INIT;
            INIT.call_once(|| {
                // Init avformat
                ffi::av_register_all();
            });

            LibAV(())
        }
    }

    pub fn version(&self) -> &'static CStr {
        unsafe {
            CStr::from_ptr(ffi::av_version_info())
        }
    }

    pub fn build_flags(&self) -> &'static CStr {
        unsafe {
            CStr::from_ptr(ffi::avformat_configuration())
        }
    }
}

// #####################

use std::fs::File;
use std::ptr;
use std::fmt;
use std::slice;
use std::ffi::CString;
use libc::{c_int, c_uint, int64_t};
use frame::VideoFrame;
pub use codec::{Codec, VideoEncoder};
use format::Muxer;
use ffi::float_t;
use ffi::AVPixelFormat::*;

const STREAM_DURATION: int64_t = 10;
const STREAM_FRAME_RATE: c_int = 30;
const STREAM_PIXEL_FORMAT: ffi::AVPixelFormat = AV_PIX_FMT_RGB24;

const VIDEO_DATA: &'static [u8] = include_bytes!("../rgb-600x400.data");

pub unsafe fn encode_demo() -> Result<(), String> {
    use std::ptr;
    use std::ffi::CString;

    LibAV::init();

    let width = 600;
    let height = 400;
    let align = 32;
    let format = "mp4";
    let pixel_format = AV_PIX_FMT_RGB24;
    let codec_id = ffi::AVCodecID::AV_CODEC_ID_H264;
    let file = File::create("/tmp/output_rust.mp4").unwrap();
    let codec = Codec::find_encoder_by_id(codec_id)?;

    let mut encoder = VideoEncoder::from_codec(codec)
        .width(width)
        .height(height)
        .pixel_format(*codec.pixel_formats().first().expect("VideoEncoder does not support any pixel formats, wtf?"))
        .open()?;

    let mut muxer = Muxer::new()
        .format_name(format)
        .add_encoder(encoder)
        .open(file)?;

    let video_stream_id = 0;

    muxer.dump_info();

    let mut frame_buffer = VIDEO_DATA.to_vec();
    let mut frame = VideoFrame::new(width, height, STREAM_PIXEL_FORMAT, align)?;
    let mut next_video_pts = 0;

    loop {
        // TODO: Use av_compare_ts for audio when applicable (see muxing example)
        if ffi::av_compare_ts(next_video_pts, muxer.encoders()[video_stream_id].as_ref().time_base, STREAM_DURATION, ffi::AVRational { num: 1, den: 1 }) >= 0 {
            break
        }

        // Render frame
        render_demo_bar(&mut frame_buffer, width, height, next_video_pts);
        frame.fill_channel(0, &frame_buffer)?;
        frame.set_pts(next_video_pts);
        next_video_pts += 1;

        muxer.send_frame(video_stream_id, &mut frame)?
    }

    Ok(())
}

fn render_demo_bar(frame_buffer: &mut [u8], width: usize, height: usize, pts: i64) {
    let max_pts = 300;
    let pixel_per_pts = width / max_pts;
    let bar_pos = pts as usize * pixel_per_pts;
    let bytes_per_pixel = 3;

    frame_buffer.copy_from_slice(VIDEO_DATA);
    for line in frame_buffer.chunks_mut(width * bytes_per_pixel) {
        for pixel in line.chunks_mut(bytes_per_pixel).take(bar_pos) {
            for component in pixel {
                *component = *component / 3;
            }
        }
    }
}
