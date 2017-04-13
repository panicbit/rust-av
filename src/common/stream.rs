use std::marker::PhantomData;
use ffi::{self, AVStream, AVFormatContext};
use std::slice;
use common::codec_parameters::CodecParameters;

pub struct Stream<'fmt_ctx> {
    ptr: *mut AVStream,
    _phantom: PhantomData<&'fmt_ctx AVFormatContext>,
}

impl<'fmt_ctx> Stream<'fmt_ctx> {
    pub unsafe fn from_ptr(ptr: *mut AVStream) -> Stream<'fmt_ctx> {
        Stream {
            ptr: ptr,
            _phantom: PhantomData,
        }
    }

    pub fn index(&self) -> usize {
        self.as_ref().index as usize
    }

    // TODO: investigate what the "Format-specific stream ID" is
    // pub fn id(&self) -> usize {
    //     self.as_ref().id as usize
    // }

    pub fn time_base(&self) -> ffi::AVRational {
        self.as_ref().time_base
    }

    // TODO: start_time

    pub fn duration(&self) -> usize {
        self.as_ref().duration as usize
    }

    /// Returns the number of frames if known
    pub fn num_frames(&self) -> Option<usize> {
        let num_frames = self.as_ref().nb_frames;

        if num_frames > 0 {
            Some(num_frames as usize)
        } else {
            None
        }
    }

    // TODO: disposition

    // TODO: remaining fields

    fn codec_parameters(&self) -> CodecParameters {
        unsafe {
            CodecParameters::from_ptr(self.as_ref().codecpar)
        }
    }
}

impl<'fmt_ctx> Stream<'fmt_ctx> {
    pub fn as_ref(&self) -> &AVStream {
        unsafe { &*self.ptr }
    }
    pub fn as_mut(&self) -> &mut AVStream {
        unsafe { &mut *self.ptr }
    }
    pub fn as_ptr(&self) -> *const AVStream {
        self.ptr
    }
    pub fn as_mut_ptr(&mut self) -> *mut AVStream {
        self.ptr
    }
}

pub struct Streams<'fmt_ctx> {
    iter: slice::Iter<'fmt_ctx, *mut AVStream>,

}

impl<'fmt_ctx> Streams<'fmt_ctx> {
    pub unsafe fn from_slice(slice: &'fmt_ctx [*mut AVStream]) -> Streams<'fmt_ctx> {
        Streams {
            iter: slice.iter()
        }
    }
}

impl<'fmt_ctx> Iterator for Streams<'fmt_ctx> {
    type Item = Stream<'fmt_ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.iter.next().map(|&ptr| Stream::from_ptr(ptr))
        }
    }
}
