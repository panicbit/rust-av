use std::cmp;
use std::ops;
use std::time::Instant;
use ffi::av_compare_ts;
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

    pub fn calc_index_since(&mut self, stream_start: Instant) {
        let duration = Instant::now().duration_since(stream_start);
        let seconds = duration.as_secs();
        let nanos = duration.subsec_nanos() as u64;
        let duration = seconds * 1_000 + nanos / 1_000_000;
        let index = duration as f64 * self.time_base.as_f64();
        self.index = index.floor() as i64;
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

#[cfg(test)]
mod test {
    use super::Ts;

    #[test]
    fn index_since_instant() {
        use std::thread::sleep;
        use std::time::{Instant, Duration};

        let fps = 30;
        let seconds = 1;
        let num_frames = seconds * fps;
        let mut ts = Ts::new(0, fps);
        let stream_start = Instant::now();

        for expected_index in 0..num_frames {
            ts.calc_index_since(stream_start);
            let distance = ts.index() - expected_index as i64;
            println!("Expected index: {}, got: {}, distance: {}", expected_index, ts.index(), distance);
            assert!(expected_index as i64 <= ts.index());

            sleep(Duration::from_millis(1000 / fps as u64));
        }
    }
}
