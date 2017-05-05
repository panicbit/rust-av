use std::os::raw::c_int;
use ffi::AVRational;

#[derive(Copy,Clone,Debug)]
pub struct Timebase(AVRational);

impl Timebase {
    pub fn new(num: c_int, den: c_int) -> Self{
        Timebase(AVRational {
            num: num,
            den: den,
        })
    }

    pub fn num(&self) -> c_int {
        self.0.num
    }

    pub fn den(&self) -> c_int {
        self.0.den
    }

    pub fn as_f32(&self) -> f32 {
        self.num() as f32 / self.den() as f32
    }

    pub fn as_f64(&self) -> f64 {
        self.num() as f64 / self.den() as f64
    }
}

impl From<AVRational> for Timebase {
    fn from(rat: AVRational) -> Self {
        Timebase(rat)
    }
}

impl From<(c_int, c_int)> for Timebase {
    fn from((num, den): (c_int, c_int)) -> Self {
        Timebase::new(num, den)
    }
}

impl From<c_int> for Timebase {
    fn from(den: c_int) -> Self {
        Timebase::new(1, den)
    }
}

impl Into<AVRational> for Timebase {
    fn into(self) -> AVRational {
        self.0
    }
}
