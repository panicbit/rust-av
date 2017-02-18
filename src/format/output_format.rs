use LibAV;
use std::ffi::{
    CString,
    CStr,
};
use std::ptr;
use std::fmt;
use libc::c_char;
use ffi::{
    AVOutputFormat,
    av_guess_format,
};

#[derive(Copy,Clone)]
pub struct OutputFormat {
    ptr: *mut AVOutputFormat
}

impl OutputFormat {
    /// Get format from short name like `mp4`, `avi`, `ogg` etc.
    pub fn from_name(name: &str) -> Option<Self> {
        unsafe {
            LibAV::init();
            let name = CString::new(name).unwrap();
            let format = av_guess_format(name.as_ptr(), ptr::null(), ptr::null());
            if format.is_null() {
                None
            } else {
                Some(OutputFormat { ptr: format })
            }
        }
    }

    /// Get format from filename extension
    pub fn from_filename(filename: &str) -> Option<Self> {
        unsafe {
            LibAV::init();
            let filename = CString::new(filename).unwrap();
            let format = av_guess_format(ptr::null(), filename.as_ptr(), ptr::null());
            if format.is_null() {
                None
            } else {
                Some(OutputFormat { ptr: format })
            }
        }
    }

    // TODO: implement `guess_from_mime`
}

impl OutputFormat {
    pub fn as_ptr(&self) -> *const AVOutputFormat { self.ptr }
    pub fn as_mut_ptr(&mut self) -> *mut AVOutputFormat { self.ptr }
    pub fn as_ref(&self) -> &AVOutputFormat { unsafe { &*self.ptr } }
}

impl fmt::Debug for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            // println!("{:?}", CStr::from_ptr(self.as_ref().mime_type).to_str());
            f.debug_struct("OutputFormat")
            .field("name", &safe_cstr(self.as_ref().name))
            .field("long_name", &safe_cstr(self.as_ref().long_name))
            .field("mime_type", &safe_cstr(self.as_ref().mime_type))
            .field("extensions", &safe_cstr(self.as_ref().extensions))
            .finish()
        }
    }
}

unsafe fn safe_cstr<'a>(ptr: *const c_char) -> Option<&'a CStr> {
    if ptr.is_null() {
        None
    } else {
        Some(&CStr::from_ptr(ptr))
    }
}
