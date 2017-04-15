#[macro_use]
extern crate error_chain;
extern crate av;

use std::fs::File;

use av::errors::ResultExt;
use av::format::Demuxer;
use av::generic::Decoder;

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
    let decoders = demuxer.streams()
        .map(|stream| Decoder::from_stream(&stream))
        .collect::<av::Result<Vec<Decoder>>>()?;

    let mut num_packets = 0;
    loop {
        let packet = match demuxer.read_packet().unwrap() {
            Some(packet) => packet,
            None => break,
        };

        num_packets += 1;
    }

    println!("Demuxed {} packets", num_packets);

    Ok(())
}
