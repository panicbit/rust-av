use ffi::av_compare_ts;
use std::cmp;
use std::ops;
use common::Timebase;

#[derive(Copy,Clone)]
pub struct Ts {
    index: i64,
    time_base: Timebase,
}

impl Ts {
    pub fn new<TB: Into<Timebase>>(index: i64, time_base: TB) -> Self {
        Ts {
            index: index,
            time_base: time_base.into(),
        }
    }

    pub fn index(&self) -> i64 {
        self.index
    }

    pub fn time_base(&self) -> Timebase {
        self.time_base
    }
}

impl cmp::PartialEq for Ts {
    fn eq(&self, other: &Ts) -> bool {
        unsafe {
            av_compare_ts(self.index, self.time_base.into(), other.index, other.time_base.into()) == 0
        }
    }
}

impl cmp::Eq for Ts {}

impl cmp::PartialOrd for Ts {
    fn partial_cmp(&self, other: &Ts) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Ts {
    fn cmp(&self, other: &Ts) -> cmp::Ordering {
        unsafe {
            use std::cmp::Ordering::*;
            match av_compare_ts(self.index, self.time_base.into(), other.index, other.time_base.into()) {
                -1 => Less,
                 0 => Equal,
                 1 => Greater,
                 _ => unreachable!("av_compare_ts BUG")
            }
        }
    }
}

impl ops::AddAssign<i64> for Ts {
    fn add_assign(&mut self, rhs: i64) {
        self.index += rhs;
    }
}
