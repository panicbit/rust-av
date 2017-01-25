use std::ffi::CStr;
use libc::c_char;

pub struct MimeTypeIter {
    ptr: *const *const c_char
}

impl MimeTypeIter {
    pub unsafe fn from_ptr(ptr: *const *const c_char) -> Self {
        MimeTypeIter { ptr: ptr }
    }
}

impl Iterator for MimeTypeIter {
    type Item = &'static CStr;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.ptr.is_null() || (*self.ptr).is_null() {
                None
            } else {
                let next = self.ptr;
                self.ptr = self.ptr.offset(1);
                Some(CStr::from_ptr(*next))
            }
        }
    }
}
