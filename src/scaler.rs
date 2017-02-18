use std::ptr;
use libc::c_int;
use ffi;

pub struct Scaler {
    ptr: *mut ffi::SwsContext,
    source_pixel_format: ffi::AVPixelFormat,
    target_pixel_format: ffi::AVPixelFormat,
    source_width: usize,
    source_height: usize,
    target_width: usize,
    target_height: usize,
}
use video;

impl Scaler {
    pub fn new(
        source_width: usize, source_height: usize, source_pixel_format: ffi::AVPixelFormat,
        target_width: usize, target_height: usize, target_pixel_format: ffi::AVPixelFormat
    ) -> Result<Self, String> {
        unsafe {
            let source_filter = ptr::null_mut();
            let target_filter = ptr::null_mut();
            let param = ptr::null_mut();
            let flags = ffi::SWS_BICUBIC as i32;

            let scaler = ffi::sws_getContext(
                source_width as i32, source_height as i32, source_pixel_format,
                target_width as i32, target_height as i32, target_pixel_format,
                flags,
                source_filter,
                target_filter,
                param
            );

            if scaler.is_null() {
                return Err(format!("Could not create scaler context"));
            }

            Ok(Scaler {
                ptr: scaler,
                source_pixel_format: source_pixel_format,
                target_pixel_format: target_pixel_format,
                source_width: source_width,
                source_height: source_height,
                target_width: target_width,
                target_height: target_height,
            })
        }
    }

    // `&mut self` is required to avoid corrupting the SwsContext
    pub unsafe fn __scale(&mut self,
        source: *mut *const u8, source_stride: *mut c_int, source_y: c_int, source_height: c_int,
        target: *mut *const u8, target_stride: *mut c_int
    ) {
        ffi::sws_scale(self.ptr,
            source, source_stride, source_y, source_height,
            target, target_stride
        );
    }

    /// TODO: Ensure that width, height and pixel format match 
    pub unsafe fn scale_frame(&mut self, source: &mut video::Frame, target: &mut video::Frame) {
        self.__scale(
            source.as_ref().data.as_ptr() as _, source.as_mut().linesize.as_mut_ptr(), 0, source.height() as i32,
            target.as_mut().data.as_mut_ptr() as _, target.as_mut().linesize.as_mut_ptr()
        );
    }

    pub fn source_pixel_format(&self) -> ffi::AVPixelFormat {
        self.source_pixel_format
    }

    pub fn source_width(&self) -> usize {
        self.source_width
    }
    
    pub fn source_height(&self) -> usize {
        self.source_height
    }

    pub fn target_pixel_format(&self) -> ffi::AVPixelFormat {
        self.target_pixel_format
    }

    pub fn target_width(&self) -> usize {
        self.target_width
    }

    pub fn target_height(&self) -> usize {
        self.target_height
    }
}

impl Drop for Scaler {
    fn drop(&mut self) {
        unsafe {
            ffi::sws_freeContext(self.ptr);
        }
    }
}
