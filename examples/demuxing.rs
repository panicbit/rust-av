#[macro_use]
extern crate error_chain;
extern crate av;

use std::fs::File;

use av::errors::ResultExt;
use av::format::Demuxer;

quick_main!(decoding);

fn decoding() -> av::Result<()> {
    av::LibAV::init();

    let file = File::open("/tmp/output_rust.mp4")
        .chain_err(|| "Failed to open input file")?;

    let mut demuxer = Demuxer::open(file)?;

    // Dump some info
    demuxer.dump_info();
    println!("{:?}", demuxer);

    let mut num_packets = 0;

    while demuxer.read_packet()?.is_some() {
        num_packets += 1;
    }

    println!("Demuxed {} packets", num_packets);

    Ok(())
}
