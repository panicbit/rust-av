pub extern crate av_sys as ffi;
extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
use std::ffi::CStr;
use std::sync::{Once, ONCE_INIT};

#[macro_use]
mod util;
mod common;

pub mod format;
pub mod video;
pub mod audio;
pub mod generic;

pub mod io;
pub mod codec;
pub mod scaler;

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
            CStr::from_ptr(ffi::av_version_info())
        }
    }

    pub fn build_flags(&self) -> &'static CStr {
        unsafe {
            CStr::from_ptr(ffi::avformat_configuration())
        }
    }
}
