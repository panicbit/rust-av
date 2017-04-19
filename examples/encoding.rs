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
use av::generic::Encoder;

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
    let mut timestamps = Vec::<Ts>::new();
    let mut encoders = Vec::<Encoder>::new();

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
    timestamps.push(Ts::new(0, video_encoder.time_base()));
    encoders.push(video_encoder.into());

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
    timestamps.push(Ts::new(0, audio_encoder.time_base()));
    encoders.push(audio_encoder.into());

    // Create format muxer
    let mut muxer = Muxer::new(output_format, file)?;

    for encoder in &encoders {
        muxer.add_stream_from_encoder(&encoder)?;
    }

    let mut muxer = muxer.open()?;

    muxer.dump_info();

    let mut video_frame_buffer = VIDEO_DATA.to_vec();
    let mut video_frame = video::Frame::new(width, height, pixel_format, align)?;
    let mut audio_data = AUDIO_DATA;
    let mut audio_frame = audio::Frame::new(audio_frame_size, sample_rate, sample_format, channel_layout)?;

    loop {
        let (index, ts) = timestamps.iter_mut().enumerate().min_by_key(|&(_, &mut ts)| ts).unwrap();
        let encoder = &mut encoders[index];

        if *ts >= final_ts {
            break;
        }

        match *encoder {
            Encoder::Video(ref mut encoder) => {
                // Render video_frame
                render_demo_bar(&mut video_frame_buffer, width, height, ts.index());
                video_frame.fill_channel(0, &video_frame_buffer)?;
                video_frame.set_pts(ts.index());
                *ts += 1;

                // Encode and mux video frame
                muxer.mux_all(encoder.encode(&mut video_frame)?, index)?;
            },
            Encoder::Audio(ref mut encoder) => {
                // Render audio_frame
                render_audio(&mut audio_frame, &mut audio_data);
                audio_frame.set_pts(ts.index());
                *ts += audio_frame_size as i64;

                // Encode and mux audio frames
                muxer.mux_all(encoder.encode(&mut audio_frame)?, index)?;
            },
        }
    }

    // Flush encoders
    for (index, encoder) in encoders.into_iter().enumerate() {
        muxer.mux_all(encoder.flush()?, index)?;
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
    println!("### Remaining audio bytes: {}", audio_data.len());
}
