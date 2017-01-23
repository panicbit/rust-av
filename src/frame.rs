use std::ops::{Deref, DerefMut};
use libc::c_int;
use ffi::{
    AVFrame,
    AVPixelFormat,
    av_frame_alloc,
    av_frame_free,
    av_frame_get_buffer,
};

const NONE_REF_MSG: &'static str = "Accessed Frame with None FrameRef";

pub struct Frame {
    inner: Option<FrameRef>,
}

impl Frame {
    /// # Panics
    ///
    /// Panics if `width`, `height` or `align` exceed `c_int::max_value()`.
    pub fn new(width: usize, height: usize, pixel_format: AVPixelFormat, align: usize) -> Result<Frame, String> {
        unsafe {
            assert!(width <= c_int::max_value() as usize, "Frame width exceeds c_int::max_value()");
            assert!(height <= c_int::max_value() as usize, "Frame height exceeds c_int::max_value()");
            assert!(align <= c_int::max_value() as usize, "Frame align exceeds c_int::max_value()");

            let frame = FrameRef::allocate()?;

            // Fill in required information
            (*frame.ptr).format = pixel_format as c_int;
            (*frame.ptr).width = width as c_int;
            (*frame.ptr).height = height as c_int;

            // Allocate actual frame buffer.
            let res = av_frame_get_buffer(frame.ptr, align as c_int);
            if res < 0 {
                frame.free();
                return Err(format!("Could not allocate video frame buffer"));
            }

            Ok(Self::from_ref(frame))
        }
    }

    pub unsafe fn from_ref(frame_ref: FrameRef) -> Frame {
        Frame {
            inner: Some(frame_ref)
        }
    }

    /// Leak the inner reference.
    pub fn leak(mut self) -> FrameRef {
        self.inner.take().expect(NONE_REF_MSG)
    }
}

impl Deref for Frame {
    type Target = FrameRef;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect(NONE_REF_MSG)
    }
}

impl DerefMut for Frame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect(NONE_REF_MSG)
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            self.inner.take().map(|f| f.free()); 
        }
    }
}

pub struct FrameRef { ptr: *mut AVFrame }

impl FrameRef {
    pub unsafe fn from_raw(ptr: *mut AVFrame) -> FrameRef {
        FrameRef { ptr: ptr }
    }

    pub fn into_raw(self) -> *mut AVFrame {
        self.ptr
    }

    pub unsafe fn allocate() -> Result<FrameRef, String> {
        let mut picture = av_frame_alloc();
        if picture.is_null() {
            Err(format!("Could not allocate video frame"))
        } else {
            Ok(Self::from_raw(picture))
        }
    }

    pub unsafe fn free(mut self) {
        if !self.ptr.is_null() {
            av_frame_free(&mut self.ptr);
        }
    }
}
