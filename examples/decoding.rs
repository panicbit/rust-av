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

    let decoder = Demuxer::open(file)?;

    decoder.dump_info();
    
    Ok(())
}