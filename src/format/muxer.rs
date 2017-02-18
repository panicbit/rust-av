use std::ptr;
use std::mem;
use std::fmt;
use std::ffi::{CStr,CString};
use libc::{c_uint, int32_t};
use LibAV;
use io;
use ffi;
use ffi::{
    AVFormatContext,
    AVOutputFormat,
    AVPacket,
    AVStream,
    AVRational,
    AV_TIME_BASE,
    AVFMT_GLOBALHEADER,
    AV_CODEC_FLAG_GLOBAL_HEADER,
    AV_CODEC_CAP_DELAY,
    AVERROR_EAGAIN,
    AVERROR_EOF,
};
use generic::{
    Encoder,
    RefMutFrame,
};
use format::OutputFormat;
use util::AsCStr;

pub struct Muxer {
    ptr: *mut AVFormatContext,
    // The io context is borrowed by the format context
    // and is kept around to be dropped at the right time.
    _io_context: io::IOContext,
    encoders: Vec<Encoder>,
    closed: bool,
}

impl Muxer {
    pub fn new() -> MuxerBuilder {
        MuxerBuilder::new()
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

    pub fn encoders(&self) -> &[Encoder] {
        &self.encoders
    }

    pub fn dump_info(&self) {
        unsafe {
            let stream_index = 0;
            let url = ptr::null();
            let is_output = 1;
            ffi::av_dump_format(self.as_ptr() as _, stream_index, url, is_output);
        }
    }

    pub fn send_frame<'a, F>(&mut self, stream_id: usize, frame: F) -> Result<(), String> where
        F: Into<RefMutFrame<'a>>
    {
        unsafe {
            let frame = frame.into();
            let format_context = self.ptr;
            let stream = *self.as_mut().streams.offset(stream_id as isize);
            let encoder = &mut self.encoders[stream_id];
            let time_base = encoder.time_base();
            
            encoder.send_frame(frame, |packet|
                write_frame(format_context, time_base, stream, packet)
            )
        }
    }

    pub fn close(mut self) -> Result<(), String> {
        self.closed = true;
        self._real_close()
    }

    fn _real_close(&mut self) -> Result<(), String> {
        unsafe {
            self._flush()?;

            let res = ffi::av_write_trailer(self.as_mut_ptr());
            if res < 0 {
                return Err(format!("Failed to write trailer"));
            } else {
                 Ok(())
            }
        }
    }

    // TODO: Improve flushing?
    fn _flush(&mut self) -> Result<(), String> {
        unsafe {
            let streams = self.as_mut().streams;
            let mut packet = ::std::mem::zeroed();

            let mut continue_flushing = true;
            while continue_flushing {
                continue_flushing = false;
                for stream_id in 0..self.num_streams() {
                    // Only try to flush codecs that support it
                    if self.encoders_mut()[stream_id].codec().as_ref().capabilities as c_uint & AV_CODEC_CAP_DELAY == 0 {
                        continue;
                    }

                    let encoder = self.encoders_mut()[stream_id].as_mut_ptr();
                    let null_frame = ptr::null_mut();

                    ffi::av_init_packet(&mut packet);

                    // TODO: Check errors on send_frame too?
                    ffi::avcodec_send_frame(encoder, null_frame);
                    match ffi::avcodec_receive_packet(encoder, &mut packet) {
                        0 => {
                            let stream = *streams.offset(stream_id as isize);
                            let time_base = (*encoder).time_base;
                            let write_succeeded = write_frame(self.ptr, time_base, stream, &mut packet);
                            // Free the packet
                            ffi::av_packet_unref(&mut packet);
                            write_succeeded?;
                            continue_flushing = true;
                        },
                        AVERROR_EAGAIN | AVERROR_EOF => {},
                        _ => return Err(format!("Error encoding packet")),
                    }
                }
            }

            Ok(())
        }
    }
}

unsafe fn write_frame(format_context: *mut AVFormatContext, time_base: AVRational, stream: *mut AVStream, packet: &mut AVPacket) -> Result<(), String> {
    ffi::av_packet_rescale_ts(packet, time_base, (*stream).time_base);
    (*packet).stream_index = (*stream).index;

    // TODO: log_packet(muxer.as_mut_ptr(), packet);

    let res = ffi::av_interleaved_write_frame(format_context, packet);
    if res < 0 {
        return Err(format!("Failed to write frame"));
    }

    Ok(())
}

impl Muxer {
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
    pub fn encoders_mut(&mut self) -> &mut [Encoder] {
        &mut self.encoders
    }
}

impl Drop for Muxer {
    fn drop(&mut self) {
        unsafe {
            if !self.closed {
                self._real_close().ok();
            }
            ffi::avformat_free_context(self.ptr)
            // The associated io context will be implicitly dropped here.
            // It may not be dropped before the format context because
            // `avformat_free_context` might still write some data.
        }
    }
}

impl fmt::Debug for Muxer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Muxer")
            .field("num_streams", &self.num_streams())
            .field("duration", &format!("{} seconds", self.duration()))
            .field("format_name", &self.format_name())
            .field("format_long_name", &self.format_long_name())
            .finish()
    }
}

pub struct MuxerBuilder {
    name: Option<CString>,
    format_name: Option<CString>,
    format: Option<OutputFormat>,
    encoders: Vec<Encoder>,
}

impl MuxerBuilder {
    pub fn new() -> MuxerBuilder {
        LibAV::init();
        MuxerBuilder {
            name: None,
            format_name: None,
            format: None,
            encoders: Vec::new(),
        }
    }

    /// Set the name of the stream. If no format is set explicitcy,
    /// this name will be used to infer it.
    ///
    /// # Panics
    /// 
    /// Panics if `name` contains `\0` characters.
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(CString::new(name).unwrap()); self
    }

    /// Set the name of the format. If unset the name of the stream will be used
    /// to infer the format.
    ///
    /// # Panics
    /// 
    /// Panics if `name` contains `\0` characters.
    pub fn format_name(&mut self, format: &str) -> &mut Self {
        self.format_name = Some(CString::new(format).unwrap()); self
    }

    /// Set the output format.
    pub fn format(&mut self, format: OutputFormat) -> &mut Self {
        self.format = Some(format); self
    }

    pub fn add_encoder<E: Into<Encoder>>(&mut self, encoder: E) -> &mut Self {
        self.encoders.push(encoder.into()); self
    }

    pub fn open<W: io::AVWrite>(&mut self, writer: W) -> Result<Muxer, String> {
        unsafe {
            let name = self.name.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null());
            let format_name = self.format_name.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null());
            let mut format = self.format.ok_or(format!("No format set"))?;
            let mut encoders = mem::replace(&mut self.encoders, Vec::new());
            let mut io_context = io::IOContext::from_writer(writer);
            let mut format_context = ptr::null_mut();

            ffi::avformat_alloc_output_context2(&mut format_context, format.as_mut_ptr(), format_name, name);
            if format_context.is_null() {
                return Err(format!("Failed to allocate output context (unknown format?)"));
            }

            // lend the io context to the format context
            (*format_context).pb = io_context.as_mut_ptr();

            for encoder in &mut encoders {
                // Create stream context
                let stream = ffi::avformat_new_stream(format_context, encoder.codec().as_ptr());
                if stream.is_null() {
                    ffi::avformat_free_context(format_context);
                    return Err(format!("Could not allocate stream"))
                }

                (*stream).id = (*format_context).nb_streams as i32 - 1;
                (*stream).time_base = encoder.time_base();

                // Verify that encoder has global header flag set if required
                let format_flags = (*(*format_context).oformat).flags as c_uint;
                let encoder_flags = encoder.as_mut().flags;
                if    0 != (format_flags & AVFMT_GLOBALHEADER)
                   && 0 == (encoder_flags & AV_CODEC_FLAG_GLOBAL_HEADER as int32_t)
                {
                    ffi::avformat_free_context(format_context);
                    return Err(format!("Format requires global headers but encoder for stream {} does not use global headers", (*stream).id))
                }

                // Copy encoder parameters to stream
                let res = ffi::avcodec_parameters_from_context((*stream).codecpar, encoder.as_mut_ptr());
                if res < 0 {
                    ffi::avformat_free_context(format_context);
                    return Err(format!("Could not copy stream parameters ({})", res))
                }
            }

            let options = ptr::null_mut();
            let res = ffi::avformat_write_header(format_context, options);
            if res < 0 {
                ffi::avformat_free_context(format_context);
                return Err(format!("Could not write header"));
            }

            Ok(Muxer {
                ptr: format_context,
                _io_context: io_context,
                encoders: encoders,
                closed: false,
            })
        }
    }
}
