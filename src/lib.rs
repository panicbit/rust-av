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

    pub fn set_log_level(&self, level: LogLevel) {
        unsafe {
            ffi::av_log_set_level(level as i32);
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

#[repr(i32)]
pub enum LogLevel {
    /// Print no output.
    Quiet = ffi::AV_LOG_QUIET as i32,
    /// Something went really wrong and we will crash now.
    Panic = ffi::AV_LOG_PANIC as i32,
    /// Something went wrong and recovery is not possible.
    /// For example, no header was found for a format which
    /// depends on headers or an illegal combination of parameters
    /// is used.
    Fatal = ffi::AV_LOG_FATAL as i32,
    /// Something went wrong and cannot losslessly be recovered.
    /// However, not all future data is affected.
    Error = ffi::AV_LOG_ERROR as i32,
    /// Something somehow does not look correct.
    /// This may or may not lead to problems.
    /// An example would be the use of '-vstrict -2'.
    Warning = ffi::AV_LOG_WARNING as i32,
    /// Standard information.
    Info = ffi::AV_LOG_INFO as i32,
    /// Detailed information.
    Verbose = ffi::AV_LOG_VERBOSE as i32,
    /// Stuff which is only useful for libav* developers.
    Debug = ffi::AV_LOG_DEBUG as i32,
    /// Extremely verbose debugging, useful for libav* development.
    Trace = ffi::AV_LOG_TRACE as i32,
}
