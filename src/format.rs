use ffi;
use std::ptr;
use std::fmt;
use util::PtrTakeExt;

use io;

pub struct FormatSource {
    ptr: *mut ffi::AVFormatContext,
    io_ctx: Option<io::IOContext>,
}

impl FormatSource {
    #[doc(hidden)]
    pub unsafe fn new(mut io_ctx: io::IOContext) -> Self {
        let mut this = ffi::avformat_alloc_context();
        (*this).pb = io_ctx.as_mut_ptr();

        let url = ptr::null();
        let fmt = ptr::null_mut();
        let options = ptr::null_mut();

        ffi::avformat_open_input(&mut this, url, fmt, options);

        // let dict = ptr::null_mut();

        FormatSource {
            ptr: this,
            io_ctx: Some(io_ctx),
        }
    }

    pub fn num_streams(&self) -> usize {
        unsafe { (*self.ptr).nb_streams as usize }
    }

    /// Duration in seconds (floored)
    /// TODO: Return a more exact/fexible representation
    pub fn duration(&self) -> u32 {
        let duration = unsafe { (*self.ptr).duration };
        if duration <= 0 {
            return 0;
        } else {
            duration as u32 / ffi::AV_TIME_BASE
        }
    }
}

impl Drop for FormatSource {
    fn drop(&mut self) {
        unsafe {
            self.io_ctx.take().expect("IOContext").close_with(||
                ffi::avformat_close_input(&mut self.ptr.take())
            );
        }
    }
}

impl fmt::Debug for FormatSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "FormatSource {{")?;
        writeln!(f, "    num_streams: {}", self.num_streams())?;
        writeln!(f, "    duration: {} seconds", self.duration())?;
        write!(f, "}}")
    }
}
