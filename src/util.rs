use std::mem;
use std::ptr;
use std::ffi::CStr;
use libc::c_char;

pub trait PtrTakeExt {
    fn take(&mut self) -> Self;
}

impl<T> PtrTakeExt for *const T {
    fn take(&mut self) -> Self {
        mem::replace(self, ptr::null())
    }
}

impl<T> PtrTakeExt for *mut T {
    fn take(&mut self) -> Self {
        mem::replace(self, ptr::null_mut())
    }
}

pub trait AsCStr: Sized {
    unsafe fn as_cstr<'a>(self) -> Option<&'a CStr>;
}

impl AsCStr for *const c_char {
    unsafe fn as_cstr<'a>(self) -> Option<&'a CStr> {
        if self.is_null() {
            None
        } else {
            Some(&CStr::from_ptr(self))
        }
    }
}
