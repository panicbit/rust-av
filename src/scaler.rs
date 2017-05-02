use std::ptr;
use std::os::raw::c_int;
use ffi;
use errors::*;

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
    ) -> Result<Self> {
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
                bail!("Could not create scaler context");
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
        source: *const *const u8, source_stride: *const c_int, source_y: c_int, source_height: c_int,
        target: *mut   *const u8, target_stride: *const c_int
    ) {
        ffi::sws_scale(self.ptr,
            source, source_stride, source_y, source_height,
            target, target_stride
        );
    }

    /// TODO: Ensure that width, height and pixel format match 
    pub unsafe fn scale_frame(&mut self, source: &video::Frame, target: &mut video::Frame) {
        self.__scale(
            source.as_ref().data.as_ptr() as _, source.as_ref().linesize.as_ptr(), 0, source.height() as i32,
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

#[cfg(test)]
mod test {
    #[test]
    fn reusable() {
        use super::Scaler;
        use std::ptr;
        use ffi::AVPixelFormat::*;

        let source = vec![0xFF, 0x00, 0x00];
        let source = &[source.as_ptr(), ptr::null(), ptr::null(), ptr::null()];
        let target: Vec<u8> = vec![
            1,2,3, 4,5,6, 7,8,9, 10,11,12,
            1,2,3, 4,5,6, 7,8,9, 10,11,12,
            1,2,3, 4,5,6, 7,8,9, 10,11,12,
            1,2,3, 4,5,6, 7,8,9, 10,11,12,
            99,99,99,99,99,99,99,99,99,99,99,99, // Canary
            99,99,99,99,99,99,99,99,99,99,99,99,
            99,99,99,99,99,99,99,99,99,99,99,99,
            99,99,99,99,99,99,99,99,99,99,99,99,
        ];

        {
            let target = &mut [target.as_ptr(), ptr::null(), ptr::null(), ptr::null()];

            let source_width = 1;
            let source_linesize: &[i32] = &[3*source_width, 0, 0, 0];
            let source_height = 1;
            let source_pixel_format = AV_PIX_FMT_RGB24;

            let target_width = 4;
            let target_linesize: &[i32] = &[3*target_width, 0, 0, 0];
            let target_height = 4;
            let target_pixel_format = AV_PIX_FMT_RGB24;

            let mut scaler = Scaler::new(
                source_width as usize, source_height, source_pixel_format,
                target_width as usize, target_height, target_pixel_format,
            ).unwrap();

            unsafe {
                for _ in 0 .. 2000 {
                    scaler.__scale(
                        source.as_ptr(),     source_linesize.as_ptr(), 0, source_height as i32,
                        target.as_mut_ptr(), target_linesize.as_ptr(),
                    );
                }
            }
        }

        assert_eq!(target, vec![
            0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00,
            0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00,
            0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00,
            0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00,
            99,99,99,99,99,99,99,99,99,99,99,99, // Canary
            99,99,99,99,99,99,99,99,99,99,99,99,
            99,99,99,99,99,99,99,99,99,99,99,99,
            99,99,99,99,99,99,99,99,99,99,99,99,
        ]);
    }
}
