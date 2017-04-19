#[macro_use]
extern crate error_chain;
extern crate av;

use std::fs::File;
use av::codec::Codec;
use av::format::{
    Muxer,
    OutputFormat,
};
use av::ffi::AVPixelFormat::*;
use av::ffi::AVSampleFormat::*;
use av::ffi::AVRational;
use av::ffi;
use av::audio::constants::CHANNEL_LAYOUT_MONO;
use std::cmp::min;
use av::{
    audio,
    video,
};
use av::errors::ResultExt;
use av::common::Ts;

const MOVIE_DURATION: i64 = 10;
const MOVIE_TIMEBASE: AVRational = AVRational { num: 1, den: 1 };
const VIDEO_DATA: &'static [u8] = include_bytes!("../rgb-600x400.data");
const AUDIO_DATA: &'static [u8] = include_bytes!("../music-44100hz-f32-le-mono.raw");


quick_main!(demo);

pub fn demo() -> av::Result<()> {
    av::LibAV::init();

    let file = File::create("/tmp/output_rust.mp4")
        .chain_err(|| "Failed to create output file")?;

    let output_format = OutputFormat::from_name("mp4")
        .ok_or("output format not found")?;
    println!("{:?}", output_format);

    let final_ts = Ts::new(MOVIE_DURATION, MOVIE_TIMEBASE);

    // Create video encoder
    let width = 600;
    let height = 400;
    let framerate = 30;
    let align = 32;
    let pixel_format = AV_PIX_FMT_RGB24;
    let video_codec_id = ffi::AVCodecID::AV_CODEC_ID_H264;
    let video_codec = Codec::find_encoder_by_id(video_codec_id)?;
    let mut video_encoder = video::Encoder::from_codec(video_codec)?
        .width(width)
        .height(height)
        .pixel_format(*video_codec.pixel_formats().first().expect("Video encoder does not support any pixel formats, wtf?"))
        .framerate(framerate)
        .open(output_format)?;
    let video_time_base = video_encoder.time_base();
    let mut video_ts = Ts::new(0, video_time_base);

    // Create audio encoder
    let sample_rate = 44100;
    let sample_format = AV_SAMPLE_FMT_FLTP;
    let channel_layout = CHANNEL_LAYOUT_MONO;
    let audio_codec_id = ffi::AVCodecID::AV_CODEC_ID_AAC;
    let audio_codec = Codec::find_encoder_by_id(audio_codec_id)?;
    let mut audio_encoder = audio::Encoder::from_codec(audio_codec)?
        .sample_rate(sample_rate)
        .sample_format(sample_format)
        .channel_layout(channel_layout)
        .open(output_format)?;
    let audio_time_base = audio_encoder.time_base();
    let mut audio_ts = Ts::new(0, audio_time_base);

    let mut audio_frame_size = audio_encoder.frame_size();
    if audio_frame_size == 0 {
        audio_frame_size = 10000;
    }
    println!("Audio frame size: {} samples", audio_frame_size);

    // Create format muxer
    let mut muxer = Muxer::new(output_format, file)?;

    muxer.add_stream_from_encoder(&video_encoder)?;
    muxer.add_stream_from_encoder(&audio_encoder)?;

    let mut muxer = muxer.open()?;

    let video_stream_id = 0;
    let audio_stream_id = 1;

    muxer.dump_info();

    let mut video_frame_buffer = VIDEO_DATA.to_vec();
    let mut video_frame = video::Frame::new(width, height, pixel_format, align)?;
    let mut audio_data = AUDIO_DATA;
    let mut audio_frame = audio::Frame::new(audio_frame_size, sample_rate, sample_format, channel_layout)?;

    while video_ts < final_ts {
        // Render video_frame
        render_demo_bar(&mut video_frame_buffer, width, height, video_ts.index());
        video_frame.fill_channel(0, &video_frame_buffer)?;
        video_ts += 1;
        video_frame.set_pts(video_ts.index());

        // Catch up audio frames with video frames (if there is enough data left)
        while audio_ts < video_ts && audio_data.len() >= audio_frame_size {
            // Render audio_frame
            render_audio(&mut audio_frame, &mut audio_data);

            // Encode audio frames
            muxer.mux_all(audio_encoder.encode(&mut audio_frame)?, audio_stream_id, audio_time_base)?;

            audio_ts += audio_frame_size as i64;
            audio_frame.set_pts(audio_ts.index());
        }

        // Encode video frame
        muxer.mux_all(video_encoder.encode(&mut video_frame)?, video_stream_id, video_time_base)?;
    }

    // Flush video encoder
    muxer.mux_all(video_encoder.flush()?, video_stream_id, video_time_base)?;
    // Flush audio encoder
    muxer.mux_all(audio_encoder.flush()?, audio_stream_id, audio_time_base)?;

    Ok(())
}

fn render_demo_bar(video_frame_buffer: &mut [u8], width: usize, _height: usize, pts: i64) {
    let max_pts = 300;
    let pixel_per_pts = width / max_pts;
    let bar_pos = pts as usize * pixel_per_pts;
    let bytes_per_pixel = 3;

    video_frame_buffer.copy_from_slice(VIDEO_DATA);
    for line in video_frame_buffer.chunks_mut(width * bytes_per_pixel) {
        for pixel in line.chunks_mut(bytes_per_pixel).take(bar_pos) {
            for component in pixel {
                *component = *component / 3;
            }
        }
    }
}

fn render_audio(audio_frame: &mut audio::Frame, audio_data: &mut &[u8]) {
    println!("### TODO: Do proper audio rendering");
    println!("### frame_size: {}", audio_frame.data_mut()[0].len());
    let buf_len = min(audio_data.len(), audio_frame.data_mut()[0].len());

    audio_frame.data_mut()[0][..buf_len].copy_from_slice(&audio_data[..buf_len]);
    *audio_data = &audio_data[buf_len..];
    println!("### Remaining bytes: {}", audio_data.len());
}
