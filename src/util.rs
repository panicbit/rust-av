use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ops;

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

pub enum OwnedOrRefMut<'a, T: 'a> {
    Owned(T),
    Borrowed(&'a mut T),
}

impl<'a, T: 'a> ops::Deref for OwnedOrRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match *self {
            OwnedOrRefMut::Owned(ref t) => t,
            OwnedOrRefMut::Borrowed(ref t) => t,
        }
    } 
}

impl<'a, T: 'a> ops::DerefMut for OwnedOrRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        match *self {
            OwnedOrRefMut::Owned(ref mut t) => t,
            OwnedOrRefMut::Borrowed(ref mut t) => t,
        }
    } 
}
