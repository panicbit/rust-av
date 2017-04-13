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

    let demuxer = Demuxer::open(file)?;

    // Dump some info
    demuxer.dump_info();
    println!("{:?}", demuxer);

    // Create decoders
    let decoders = demuxer.streams()
        .map(|stream| Decoder::from_stream(&stream))
        .collect::<av::Result<Vec<Decoder>>>()?;

    Ok(())
}