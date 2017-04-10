use std::ptr;
use std::fmt;
use std::ffi::CStr;
use LibAV;
use io;
use ffi;
use ffi::{
    AVFormatContext,
    AVOutputFormat,
    AV_TIME_BASE,
};
use util::AsCStr;
use errors::*;

pub struct Demuxer {
    ptr: *mut AVFormatContext,
    // The io context is borrowed by the format context
    // and is kept around to be dropped at the right time.
    _io_context: io::IOContext,
}

impl Demuxer {
    pub fn open<W: io::AVRead>(reader: W) -> Result<Demuxer> {
        unsafe {
            LibAV::init();

            // Allocate IOContext and AVFormatContext
            let mut io_context = io::IOContext::from_reader(reader);
            let mut format_context = ffi::avformat_alloc_context();

            if format_context.is_null() {
                bail!("Failed to allocate input context");
            }

            // Lend the io context to the format context
            (*format_context).pb = io_context.as_mut_ptr();

            // Open the decoder
            {
                let url = ptr::null_mut();
                let format = ptr::null_mut();
                let options = ptr::null_mut();
                let res = ffi::avformat_open_input(&mut format_context, url, format, options);

                if res < 0 {
                    // No need to fre format_context here.
                    // avformat_open_input already has freed the format context at this point.
                    bail!("Failed to open input context");
                }
            }

            // Decode some stream info
            {
                let options = ptr::null_mut();
                let res = ffi::avformat_find_stream_info(format_context, options);

                if res < 0 {
                    ffi::avformat_close_input(&mut format_context);
                    bail!("Failed to find stream info");
                }
            }


            Ok(Demuxer {
                ptr: format_context,
                _io_context: io_context,
            })
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
            duration as u32 / AV_TIME_BASE
        }
    }

    pub fn format_name(&self) -> &CStr {
        unsafe {
            self.output_format().name.as_cstr().unwrap()
        }
    }

    pub fn format_long_name(&self) -> &CStr {
        unsafe {
            self.output_format().long_name.as_cstr().unwrap()
        }
    }

    pub fn dump_info(&self) {
        unsafe {
            let stream_index = 0;
            let url = ptr::null();
            let is_output = 0;
            ffi::av_dump_format(self.as_ptr() as _, stream_index, url, is_output);
        }
    }
}

impl Demuxer {
    pub fn as_ref(&self) -> &AVFormatContext {
        unsafe { &*self.ptr }
    }
    pub fn as_mut(&self) -> &mut AVFormatContext {
        unsafe { &mut *self.ptr }
    }
    pub fn as_ptr(&self) -> *const AVFormatContext {
        self.ptr
    }
    pub fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
        self.ptr
    }
    unsafe fn output_format(&self) -> &AVOutputFormat {
        &*self.as_ref().oformat
    }
    // pub fn encoders_mut(&mut self) -> &mut [Encoder] {
    //     &mut self.encoders
    // }
}

impl Drop for Demuxer {
    fn drop(&mut self) {
        unsafe {
            ffi::avformat_close_input(&mut self.ptr)
        }
    }
}

impl fmt::Debug for Demuxer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Demuxer")
            .field("num_streams", &self.num_streams())
            .field("duration", &format!("{} seconds", self.duration()))
            .field("format_name", &self.format_name())
            .field("format_long_name", &self.format_long_name())
            .finish()
    }
}
