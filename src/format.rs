use ffi;
use std::ptr;
use std::mem;

use io;

pub struct FormatSource {
    ptr: *mut ffi::AVFormatContext,
    io_ctx: Option<io::IOContext>,
}

impl FormatSource {
    #[doc(hidden)]
    pub unsafe fn new(mut io_ctx: io::IOContext) -> Self {
        let format_ctx = ffi::avformat_alloc_context();
        (*format_ctx).pb = io_ctx.as_mut_ptr();

        FormatSource {
            ptr: format_ctx,
            io_ctx: Some(io_ctx),
        }
    }
}

impl Drop for FormatSource {
    fn drop(&mut self) {
        unsafe {
            self.io_ctx.take();
            (*self.ptr).pb = ptr::null_mut();
            ffi::avformat_free_context(mem::replace(&mut self.ptr, ptr::null_mut()));
        }
    }
}
