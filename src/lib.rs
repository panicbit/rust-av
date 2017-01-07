extern crate av_sys as ffi;
extern crate libc;
use std::ffi::CStr;
use std::sync::{Once, ONCE_INIT};

pub mod format;
pub mod io;
mod util;

static INIT: Once = ONCE_INIT;

pub struct LibAV(());

impl LibAV {
    pub fn init() -> LibAV {
        unsafe {
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

    pub fn open_format_source<R: io::AVRead>(&self, reader: R) -> Result<format::FormatSource, &'static str> {
        unsafe {
            let io_context = io::IOContext::from_reader(reader);
            format::FormatSource::new(io_context)
        }
    }
}
