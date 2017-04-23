extern crate bindgen;

use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let prefix = env::var("RUST_FFMPEG_PREFIX").unwrap_or("/usr".to_owned());

    bindgen::builder()
        .header("ffi.h")
        .clang_arg(format!("-I{}/include", prefix))
        .no_unstable_rust()
        .ctypes_prefix("::libc")
        .generate().unwrap()
        .write_to_file(Path::new(&out_dir).join("ffi.rs"))
        .unwrap();

    println!("cargo:rustc-link-search=native={}/lib", prefix);
    println!("cargo:rustc-link-lib=dylib=avutil");
    println!("cargo:rustc-link-lib=dylib=avformat");
    println!("cargo:rustc-link-lib=dylib=avcodec");
    println!("cargo:rustc-link-lib=dylib=avdevice");
    println!("cargo:rustc-link-lib=dylib=avfilter");
    println!("cargo:rustc-link-lib=dylib=swresample");
    println!("cargo:rustc-link-lib=dylib=swscale");
}
