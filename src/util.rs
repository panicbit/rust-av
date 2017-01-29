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

pub struct Guard<'a> {
    droppers: Vec<Box<FnMut() + 'a>>,
    armed: bool,
}

pub fn with_guard<R, F: FnOnce(&mut Guard) -> R>(f: F) -> R {
    let mut guard = Guard::new();
    let result = f(&mut guard);
    guard.disarm();
    result
}

impl<'a> Guard<'a> {
    pub fn new() -> Guard<'a> {
        Guard {
            droppers: Vec::new(),
            armed: true,
        }
    }

    pub fn add<F: FnMut() + 'a>(&mut self, f: F) {
        self.droppers.push(Box::new(f));
    }

    pub fn disarm(mut self) {
        self.armed = false;
    }
}

impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        if self.armed {
            for dropper in &mut self.droppers {
                dropper();
            }
        }
    }
}
