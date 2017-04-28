use std::ptr;
use std::mem;
use std::fmt;
use std::ffi::{CStr,CString};
use std::os::raw::c_uint;
use LibAV;
use io;
use ffi;
use ffi::{
    AVFormatContext,
    AVOutputFormat,
    AVPacket,
    AVStream,
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
    Packets,
};
use format::OutputFormat;
use util::AsCStr;
use common::RcPacket;
use errors::*;

pub struct Muxer {
    ptr: *mut AVFormatContext,
    // The io context is borrowed by the format context
    // and is kept around to be dropped at the right time.
    _io_context: io::IOContext,
    // Whether muxer was closed explicitly
    closed: bool,
}

impl Muxer {
    pub fn new<W: io::AVWrite>(format: OutputFormat, writer: W) -> Result<MuxerBuilder> {
        MuxerBuilder::new(format, writer)
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
            let is_output = 1;
            ffi::av_dump_format(self.as_ptr() as _, stream_index, url, is_output);
        }
    }

    pub fn mux(&mut self, packet: RcPacket, stream_index: usize) -> Result<()> {
        unsafe {
            if stream_index >= self.num_streams() {
                bail!("Invalid stream index {}. Only {} stream(s) exist(s).", stream_index, self.num_streams());
            }

            let packet_time_base = packet.time_base();
            let packet = &mut *packet.as_mut_ptr();
            let stream = *self.as_ref().streams.offset(stream_index as isize);
            let stream_time_base = (*stream).time_base;
            ffi::av_packet_rescale_ts(packet, packet_time_base.into(), stream_time_base);
            packet.stream_index = stream_index as i32;

            // // TODO: log_packet(muxer.as_mut_ptr(), packet);

            let res = ffi::av_interleaved_write_frame(self.ptr, packet);
            if res < 0 {
                bail!("Failed to write packet for stream {}: 0x{:X}", stream_index, res);
            }

            Ok(())
        }
    }

    pub fn mux_all<'a, P: Into<Packets<'a>>>(&mut self, packets: P, stream_index: usize) -> Result<()> {
        for packet in packets.into() {
            self.mux(packet?, stream_index)?;
        }
        Ok(())
    }

    pub fn close(mut self) -> Result<()> {
        self.closed = true;
        self._real_close()
    }

    fn _real_close(&mut self) -> Result<()> {
        unsafe {
            // Write trailer
            {
                let res = ffi::av_write_trailer(self.as_mut_ptr());
                if res < 0 {
                    bail!("Failed to write trailer: 0x{:X}", res);
                }
            }

            Ok(())
        }
    }
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
    ptr: *mut AVFormatContext,
    io_context: Option<io::IOContext>,
}

impl MuxerBuilder {
    pub fn new<W: io::AVWrite>(mut format: OutputFormat, writer: W) -> Result<MuxerBuilder> {
        unsafe {
            LibAV::init();

            let mut muxer = ptr::null_mut();
            let mut io_context = io::IOContext::from_writer(writer);

            // Allocate muxer
            {
                let format_name = ptr::null();
                let name = ptr::null();

                let res = ffi::avformat_alloc_output_context2(&mut muxer, format.as_mut_ptr(), format_name, name);

                if res < 0 || muxer.is_null() {
                    bail!("Failed to allocate output context: 0x{:X}", res);
                }
            }

            // lend the io context to the format context
            (*muxer).pb = io_context.as_mut_ptr();

           Ok(MuxerBuilder {
                ptr: muxer,
                io_context: Some(io_context),
            })
        }
    }

    /// Add a new stream using the settings from an encoder.
    pub fn add_stream_from_encoder<E: AsRef<ffi::AVCodecContext>>(&mut self, encoder: E) -> Result<()> {
        unsafe {
            // Verify that encoder has global header flag set if required
            {
                let format_flags = (*(*self.ptr).oformat).flags as c_uint;
                let encoder_flags = encoder.as_ref().flags;

                if    0 != (format_flags & AVFMT_GLOBALHEADER)
                   && 0 == (encoder_flags & AV_CODEC_FLAG_GLOBAL_HEADER as i32)
                {
                    bail!("Format requires global headers but encoder does not use global headers")
                }
            }

            // Create stream context
            let stream = ffi::avformat_new_stream(self.ptr, encoder.as_ref().codec);
            if stream.is_null() {
                bail!("Could not allocate stream")
            }

            (*stream).id = (*self.ptr).nb_streams as i32 - 1;
            (*stream).time_base = encoder.as_ref().time_base;

            // Copy encoder parameters to stream
            // NOTE: Not advised for remuxing: https://ffmpeg.org/doxygen/3.2/group__lavf__encoding.html#details
            {
                let res = ffi::avcodec_parameters_from_context((*stream).codecpar, encoder.as_ref());
                if res < 0 {
                    bail!("Could not copy stream parameters ({})", res)
                }
            }

            Ok(())
        }
    }

    pub fn open(mut self) -> Result<Muxer> {
        unsafe {
            // Write header 
            {
                let options = ptr::null_mut();
                let res = ffi::avformat_write_header(self.ptr, options);
                if res < 0 {
                    ffi::avformat_free_context(self.ptr);
                    bail!("Could not write header: 0x{:X}", res);
                }
            }

            Ok(Muxer {
                ptr: mem::replace(&mut self.ptr, ptr::null_mut()),
                _io_context: self.io_context.take().unwrap(),
                closed: false,
            })
        }
    }

}

impl Drop for MuxerBuilder {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                ffi::avformat_free_context(self.ptr);
            }
        }
    }
}
