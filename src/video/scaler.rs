use std::ptr;
use smallvec::SmallVec;
use ffi::{self,AVPixelFormat};
use errors::*;
use super::{Frame, MAX_PLANES};

/// A context for scaling/converting video frames.
///
/// Scaling is most efficiently done by reusing the scaler
/// for a specific combination of source/target width, height and format.
/// It's allowed to use different values for each invocation of the scaling
/// functions, but it will result in reallocation of the internal scaling context,
/// which might not be desireable.
pub struct Scaler {
    context: Option<SwsContext>,
    src_w: usize,
    src_h: usize,
    src_fmt: AVPixelFormat,
    dst_w: usize,
    dst_h: usize,
    dst_fmt: AVPixelFormat,
}

impl Scaler {
    /// Create a new scaling context.
    pub fn new() -> Self {
        Scaler {
            context: None,
            src_w: 0,
            src_h: 0,
            src_fmt: AVPixelFormat::AV_PIX_FMT_RGB24,
            dst_w: 0,
            dst_h: 0,
            dst_fmt: AVPixelFormat::AV_PIX_FMT_RGB24,
        }
    }

    /// Actually initialize the context and
    /// reinitialize it if needed.
    fn init_context(&mut self,
        src_w: usize, src_h: usize, src_fmt: AVPixelFormat,
        dst_w: usize, dst_h: usize, dst_fmt: AVPixelFormat,
    ) -> Result<&mut SwsContext> {
        // (Re)allocate
        if    self.context.is_none()
           || self.src_w   != src_w   || self.dst_w   != dst_w
           || self.src_h   != src_h   || self.dst_h   != dst_h
           || self.src_fmt != src_fmt || self.dst_fmt != dst_fmt
        {
            let flags = ffi::SWS_BICUBIC as i32;
            self.context = Some(SwsContext::new(
                src_w as i32, src_h as i32, src_fmt,
                dst_w as i32, dst_h as i32, dst_fmt,
                flags
            )?);

            self.src_h   = src_h;
            self.src_w   = src_w;
            self.src_fmt = src_fmt;
            self.dst_h   = dst_h;
            self.dst_w   = dst_w;
            self.dst_fmt = dst_fmt;
        }

        Ok(self.context.as_mut().unwrap())
    }

    /// Scale `src_data` to `src_data` by using the given dimensions and formats.
    ///
    /// # Requirements
    ///
    /// - `src_fmt` and `dst_fmt` need to be valid pixel formats.
    /// - `src_w`, `src_h`, `dst_w` and `dst_h` need to be greater than 0.
    /// - The number of planes need to be greater than or equal to the
    ///   number of planes required by the pixel formats.
    /// - The planes need to be big enough to contain the amount of bytes
    ///   described by their linesize and the height (`linesize * height`).
    pub fn scale(&mut self,
        src_data: &    [&    [u8]], src_linesize: &[usize], src_w: usize, src_h: usize, src_fmt: AVPixelFormat,
        dst_data: &mut [&mut [u8]], dst_linesize: &[usize], dst_w: usize, dst_h: usize, dst_fmt: AVPixelFormat,
    ) -> Result<()> {
        unsafe {
            // Get appropriate scaling context
            let context = self.init_context(
                src_w, src_h, src_fmt,
                dst_w, dst_h, dst_fmt,
            )?;

            let src_num_planes = ffi::av_pix_fmt_count_planes(src_fmt);
            let dst_num_planes = ffi::av_pix_fmt_count_planes(dst_fmt);

            if src_num_planes <= 0 || dst_num_planes <= 0 {
                bail!("Invalid pixel format for scale source or target");
            }

            let src_num_planes = src_num_planes as usize;
            let dst_num_planes = dst_num_planes as usize;

            // Check that the required planes are availabe
            {
                if src_data.len() < src_num_planes || src_linesize.len() < src_num_planes {
                    bail!("Scale source has invalid number of planes");
                }

                if dst_data.len() < dst_num_planes || dst_linesize.len() < dst_num_planes {
                    bail!("Scale target has invalid number of planes");
                }
            }

            // Check that the buffers are big enough for the given h/w
            {
                for (plane, linesize) in src_data.iter().zip(src_linesize) {
                    if src_h * linesize > plane.len() {
                        println!("Source plane data too small");
                    }
                }

                for (plane, linesize) in dst_data.iter().zip(dst_linesize) {
                    if dst_h * linesize > plane.len() {
                        println!("Target plane data too small");
                    }
                }
            }

            // Convert the slices to the proper ffmpeg types
            let mut src_data: SmallVec<[*const u8; MAX_PLANES]> = src_data.iter().map(| s| s.as_ptr()).collect();
            let mut dst_data: SmallVec<[*const u8; MAX_PLANES]> = dst_data.iter().map(| s| s.as_ptr()).collect();
            let mut src_linesize: SmallVec<[i32; MAX_PLANES]> = src_linesize.iter().map(|&s| s as i32).collect();
            let mut dst_linesize: SmallVec<[i32; MAX_PLANES]> = dst_linesize.iter().map(|&s| s as i32).collect();

            // Fill arrays up to MAX_PLANES
            while src_data.len() < 4 { src_data.push(ptr::null()) }
            while dst_data.len() < 4 { dst_data.push(ptr::null()) }
            while src_linesize.len() < 4 { src_linesize.push(0) }
            while dst_linesize.len() < 4 { dst_linesize.push(0) }

            let source_y = 0;

            ffi::sws_scale(context.as_mut_ptr(),
                src_data.as_ptr()    , src_linesize.as_ptr(), source_y, src_h as i32,
                dst_data.as_mut_ptr(), dst_linesize.as_ptr(),
            );

            Ok(())
        }
    }

    /// Copy the `src` pixel data to the `dst` pixel data,
    /// scaling dimensions and converting pixel formats as required.
    pub fn scale_frame(&mut self, src: &Frame, dst: &mut Frame) -> Result<()> {
        let src_data     = &src.data();
        let src_linesize = &src.linesizes();
        let src_h        =  src.height();
        let src_w        =  src.width();
        let src_fmt      =  src.pixel_format();

        let dst_data     = &mut dst.data_mut();
        let dst_linesize = &    dst.linesizes();
        let dst_h        =      dst.height();
        let dst_w        =      dst.width();
        let dst_fmt      =      dst.pixel_format();

        self.scale(
            src_data, src_linesize, src_w, src_h, src_fmt,
            dst_data, dst_linesize, dst_w, dst_h, dst_fmt,
        )
    }
}

unsafe impl Send for Scaler{}
unsafe impl Sync for Scaler{}

struct SwsContext(*mut ffi::SwsContext);

impl SwsContext {
    fn new(
        source_width: i32, source_height: i32, source_pixel_format: AVPixelFormat,
        target_width: i32, target_height: i32, target_pixel_format: AVPixelFormat,
        flags: i32,
    ) -> Result<Self> {
        unsafe {
            let source_filter = ptr::null_mut();
            let target_filter = ptr::null_mut();
            let param = ptr::null_mut();

            // Check that the pixel dimensions are valid
            {
                if source_width <= 0 || source_height <= 0 {
                    bail!("Scale source dimension {} x {} invalid", source_width, source_height)
                }

                if target_width <= 0 || target_height <= 0 {
                    bail!("Scale target dimension {} x {} invalid", target_width, target_height)
                }
            }

            let scaler = ffi::sws_getContext(
                source_width, source_height, source_pixel_format,
                target_width, target_height, target_pixel_format,
                flags,
                source_filter,
                target_filter,
                param
            );

            if scaler.is_null() {
                bail!("Could not create scaler context");
            }

            Ok(SwsContext(scaler))
        }
    }

    unsafe fn free(&mut self) {
        if !self.0.is_null() {
            ffi::sws_freeContext(self.0);
            self.0 = ptr::null_mut();
        }
    }

    fn as_mut_ptr(&mut self) -> *mut ffi::SwsContext {
        self.0
    }
}

impl Drop for SwsContext {
    fn drop(&mut self) {
        unsafe {
            self.free()
        }
    }
}

#[cfg(test)]
mod test {
    use ffi::AVPixelFormat::*;
    use super::Scaler;

    #[test]
    fn reusable() {

        let source = vec![0xFF, 0x00, 0x00];
        let source_data = &[&source[..], &[], &[], &[]];
        let source_width = 1;
        let source_linesize = &[3*source_width, 0, 0, 0];
        let source_height = 1;
        let source_format = AV_PIX_FMT_RGB24;

        let mut target: Vec<u8> = vec![
            1,2,3, 4,5,6, 7,8,9, 10,11,12,
            1,2,3, 4,5,6, 7,8,9, 10,11,12,
            1,2,3, 4,5,6, 7,8,9, 10,11,12,
            1,2,3, 4,5,6, 7,8,9, 10,11,12,
            99,99,99,99,99,99,99,99,99,99,99,99, // Canary
            99,99,99,99,99,99,99,99,99,99,99,99,
            99,99,99,99,99,99,99,99,99,99,99,99,
            99,99,99,99,99,99,99,99,99,99,99,99,
        ];
        let target_data = &mut [&mut target[..], &mut [], &mut [], &mut []];
        let target_width = 4;
        let target_linesize = &[3*target_width, 0, 0, 0];
        let target_height = 4;
        let target_format = AV_PIX_FMT_RGB24;

        let mut scaler = Scaler::new();

        for _ in 0 .. 2000 {
            scaler.scale(
                source_data, source_linesize, source_width, source_height, source_format,
                target_data, target_linesize, target_width, target_height, target_format,
            ).unwrap();
        }

        assert_eq!(target_data[0], &[
            0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00,
            0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00,
            0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00,
            0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00, 0xFF,0x00,0x00,
            99,99,99,99,99,99,99,99,99,99,99,99, // Canary
            99,99,99,99,99,99,99,99,99,99,99,99,
            99,99,99,99,99,99,99,99,99,99,99,99,
            99,99,99,99,99,99,99,99,99,99,99,99,
        ][..]);
    }

    #[test]
    #[should_panic]
    fn not_enough_planes() {
        let mut scaler = Scaler::new();

        scaler.scale(
            &    [], &    [], 1, 1, AV_PIX_FMT_RGB24,
            &mut [], &mut [], 1, 1, AV_PIX_FMT_RGB24,
        ).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_dimensions() {
        let mut scaler = Scaler::new();

        scaler.scale(
            &    [], &    [], 0, 0, AV_PIX_FMT_RGB24,
            &mut [], &mut [], 0, 0, AV_PIX_FMT_RGB24,
        ).unwrap();
    }
}
