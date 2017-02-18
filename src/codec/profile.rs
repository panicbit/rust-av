use std::fmt;
use std::ffi::CStr;
use ffi::{
    AVProfile,
    FF_PROFILE_UNKNOWN,
};
use util::AsCStr;

pub struct Profile {
    ptr: *const AVProfile
}

impl Profile {
    pub fn name(&self) -> &CStr {
        unsafe {
            (*self.ptr).name.as_cstr().unwrap()
        }
    }
}

impl fmt::Debug for Profile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Profile")
            .field("name", &self.name())
            .finish()
    }
}

pub struct ProfileIter {
    next: *const AVProfile
}

impl ProfileIter {
    pub unsafe fn from_ptr(ptr: *const AVProfile) -> Self {
        ProfileIter { next: ptr }
    }
}

impl Iterator for ProfileIter {
    type Item = Profile;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.next.is_null() || (*self.next).profile == FF_PROFILE_UNKNOWN {
                None
            } else {
                let next = self.next;
                self.next = next.offset(1);
                Some(Profile { ptr: next })
            }
        }
    }
}
