extern crate av;
#[macro_use] extern crate lazy_static;

use std::fs::File;
use av::LibAV;

lazy_static! {
    static ref AV: LibAV = LibAV::init();
}

fn main() {
    println!("{}", AV.version().to_string_lossy());
    println!("{}", AV.build_flags().to_string_lossy());

    let input = File::open("/tmp/input.mp4").expect("input file");
    let format_input = AV.open_format_source(input);

    println!("{:?}", format_input);

    println!("\n=== No crash ===");
}
