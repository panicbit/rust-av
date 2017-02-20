extern crate av;

use std::fs::File;
use av::codec::Codec;
use av::format::{
    Muxer,
    OutputFormat,
};
use av::ffi::AVPixelFormat::*;
use av::ffi::AVSampleFormat::*;
use av::ffi;
use av::audio::constants::CHANNEL_LAYOUT_MONO;
use std::cmp::min;
use av::{
    audio,
    video,
};

const STREAM_DURATION: i64 = 10;
const VIDEO_DATA: &'static [u8] = include_bytes!("../rgb-600x400.data");
const AUDIO_DATA: &'static [u8] = include_bytes!("../music-44100hz-f32-le-mono.raw");


fn main() {
    demo().unwrap();
}

pub fn demo() -> Result<(), String> {
    av::LibAV::init();

    let file = File::create("/tmp/output_rust.mp4").unwrap();

    let output_format = OutputFormat::from_name("mp4").expect("output format not found");
    println!("{:?}", output_format);

    // Create video encoder
    let width = 600;
    let height = 400;
    let framerate = 30;
    let align = 32;
    let pixel_format = AV_PIX_FMT_RGB24;
    let video_codec_id = ffi::AVCodecID::AV_CODEC_ID_H264;
    let video_codec = Codec::find_encoder_by_id(video_codec_id)?;
    let video_encoder = video::Encoder::from_codec(video_codec)?
        .width(width)
        .height(height)
        .pixel_format(*video_codec.pixel_formats().first().expect("Video encoder does not support any pixel formats, wtf?"))
        .framerate(framerate)
        .open(output_format)?;

    // Create audio encoder
    let sample_rate = 44100;
    let sample_format = AV_SAMPLE_FMT_FLTP;
    let channel_layout = CHANNEL_LAYOUT_MONO;
    let audio_codec_id = ffi::AVCodecID::AV_CODEC_ID_AAC;
    let audio_codec = Codec::find_encoder_by_id(audio_codec_id)?;
    let audio_encoder = audio::Encoder::from_codec(audio_codec)?
        .sample_rate(sample_rate)
        .sample_format(sample_format)
        .channel_layout(channel_layout)
        .open(output_format)?;
    let mut audio_frame_size = audio_encoder.frame_size();
    if audio_frame_size == 0 {
        audio_frame_size = 10000;
    }
    println!("Audio frame size: {} samples", audio_frame_size);

    // Create format muxer
    let mut muxer = Muxer::new()
        .format(output_format)
        .add_encoder(video_encoder)
        .add_encoder(audio_encoder)
        .open(file)?;

    let video_stream_id = 0;
    let audio_stream_id = 1;

    muxer.dump_info();

    let mut video_frame_buffer = VIDEO_DATA.to_vec();
    let mut video_frame = video::Frame::new(width, height, pixel_format, align)?;
    let mut next_video_pts = 0;

    let mut audio_data = AUDIO_DATA;
    let mut audio_frame = audio::Frame::new(audio_frame_size, sample_rate, sample_format, channel_layout)?;
    let mut next_audio_pts = 0;


    loop {
        unsafe {
            // TODO: Use av_compare_ts for audio when applicable (see muxing example)
            if ffi::av_compare_ts(next_video_pts, muxer.encoders()[video_stream_id].as_ref().time_base, STREAM_DURATION, ffi::AVRational { num: 1, den: 1 }) >= 0 {
                break
            }
        }

        // Render video_frame
        render_demo_bar(&mut video_frame_buffer, width, height, next_video_pts);
        video_frame.fill_channel(0, &video_frame_buffer)?;
        video_frame.set_pts(next_video_pts);
        next_video_pts += 1;

        for _ in 0..2 {
            if audio_data.len() < audio_frame_size { break }
            // Render audio_frame
            render_audio(&mut audio_frame, &mut audio_data);
            muxer.send_frame(audio_stream_id, &mut audio_frame)?;
            audio_frame.pts_add(audio_frame_size as i64);
        }

        muxer.send_frame(video_stream_id, &mut video_frame)?
    }

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
