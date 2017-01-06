use std::mem;
use std::ptr;

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
