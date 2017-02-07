extern crate av;

use std::fs::File;
use av::frame::VideoFrame;
use av::codec::{Codec, VideoEncoder};
use av::format::Muxer;
use av::ffi::AVPixelFormat::*;
use av::ffi;

const STREAM_DURATION: i64 = 10;
const VIDEO_DATA: &'static [u8] = include_bytes!("../rgb-600x400.data");


fn main() {
    demo().unwrap();
}

pub fn demo() -> Result<(), String> {
    av::LibAV::init();

    let width = 600;
    let height = 400;
    let framerate = 30;
    let align = 32;
    let format = "mp4";
    let pixel_format = AV_PIX_FMT_RGB24;
    let codec_id = ffi::AVCodecID::AV_CODEC_ID_H264;
    let file = File::create("/tmp/output_rust.mp4").unwrap();
    let codec = Codec::find_encoder_by_id(codec_id)?;

    let encoder = VideoEncoder::from_codec(codec)
        .width(width)
        .height(height)
        .pixel_format(*codec.pixel_formats().first().expect("VideoEncoder does not support any pixel formats, wtf?"))
        .framerate(framerate)
        .open()?;

    let mut muxer = Muxer::new()
        .format_name(format)
        .add_encoder(encoder)
        .open(file)?;

    let video_stream_id = 0;

    muxer.dump_info();

    let mut frame_buffer = VIDEO_DATA.to_vec();
    let mut frame = VideoFrame::new(width, height, pixel_format, align)?;
    let mut next_video_pts = 0;

    loop {
        unsafe {
            // TODO: Use av_compare_ts for audio when applicable (see muxing example)
            if ffi::av_compare_ts(next_video_pts, muxer.encoders()[video_stream_id].as_ref().time_base, STREAM_DURATION, ffi::AVRational { num: 1, den: 1 }) >= 0 {
                break
            }
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

fn render_demo_bar(frame_buffer: &mut [u8], width: usize, _height: usize, pts: i64) {
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
