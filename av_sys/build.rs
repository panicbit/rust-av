extern crate libbindgen;

use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let _ = libbindgen::builder()
        .header("ffi.h")
        .clang_arg("-I/usr/include/")
        .no_unstable_rust()
        .ctypes_prefix("::libc")
        .generate().unwrap()
        .write_to_file(Path::new(&out_dir).join("ffi.rs"))
        .unwrap();

    println!("cargo:rustc-link-lib=dylib=avutil");
    println!("cargo:rustc-link-lib=dylib=avformat");
    println!("cargo:rustc-link-lib=dylib=avcodec");
    println!("cargo:rustc-link-lib=dylib=swresample");
    println!("cargo:rustc-link-lib=dylib=swscale");
}
