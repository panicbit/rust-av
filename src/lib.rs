// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]
pub extern crate av_sys as ffi;
extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate error_chain;
use std::ffi::CStr;
use std::sync::{Once, ONCE_INIT};
use util::AsCStr;

#[macro_use]
mod util;
pub mod common;

pub mod format;
pub mod video;
pub mod audio;
pub mod generic;

pub mod io;
pub mod codec;
pub mod scaler;

pub mod errors;
pub use self::errors::*;

lazy_static! {
    pub static ref AV: LibAV = LibAV::init();
}

pub struct LibAV(());

impl LibAV {
    pub fn init() -> LibAV {
        unsafe {
            static INIT: Once = ONCE_INIT;
            INIT.call_once(|| {
                // Init avformat
                ffi::av_register_all();
            });

            LibAV(())
        }
    }

    pub fn version(&self) -> &'static CStr {
        unsafe {
            ffi::av_version_info().as_cstr().unwrap()
        }
    }

    pub fn build_flags(&self) -> &'static CStr {
        unsafe {
            ffi::avformat_configuration().as_cstr().unwrap()
        }
    }
}
