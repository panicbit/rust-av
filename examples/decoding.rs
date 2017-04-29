#[macro_use]
extern crate error_chain;
extern crate av;

use std::fs::File;

use av::errors::ResultExt;
use av::format::Demuxer;
use av::generic::{Decoder,Frame};

quick_main!(decoding);

fn decoding() -> av::Result<()> {
    av::LibAV::init();

    let file = File::open("/tmp/output_rust.mp4")
        .chain_err(|| "Failed to open input file")?;

    let mut demuxer = Demuxer::open(file)?;

    // Dump some info
    demuxer.dump_info();
    println!("{:?}", demuxer);

    // Create decoders
    let mut decoders = demuxer.streams()
        .map(|stream| Decoder::from_stream(&stream))
        .collect::<av::Result<Vec<Decoder>>>()?;

    let mut num_packets = 0;
    let mut num_video_frames = 0;
    let mut num_audio_frames = 0;

    // Demux packets
    while let Some(packet) = demuxer.read_packet()? {
        num_packets += 1;

        // Find the correct decoder for the packet
        let decoder = &mut decoders[packet.stream_index()];

        // Feed the packet to the decoder
        let frames = decoder.decode(packet)?;

        for frame in frames {
            handle_frame(frame?, &mut num_video_frames, &mut num_audio_frames);
        }
    }

    // Flush decoders
    for mut decoder in decoders {
        for frame in decoder.flush()? {
            handle_frame(frame?, &mut num_video_frames, &mut num_audio_frames);
        }
    }

    println!("Demuxed {} packets", num_packets);
    println!("Decoded {} video frames", num_video_frames);
    println!("Decoded {} audio frames", num_audio_frames);

    Ok(())
}

fn handle_frame(frame: Frame, num_video_frames: &mut usize, num_audio_frames: &mut usize) {
    // Handle decoded frame
    match frame {
        Frame::Video(_) => *num_video_frames += 1,
        Frame::Audio(_) => *num_audio_frames += 1,
    }
}
