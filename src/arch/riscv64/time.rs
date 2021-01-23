use crate::bsp::{HAS_RDTIME, NANOS_PER_TICK, TICKS_PER_NANO};
use crate::time;
use core::time::Duration;

// TODO: support fractional nanos/tick or ticks/nano
pub struct RISCVTimer;

static TIME_COUNTER: RISCVTimer = RISCVTimer;

pub fn time_counter() -> &'static impl time::TimeCounter {
    &TIME_COUNTER
}

impl RISCVTimer {
    #[inline(always)]
    fn raw_time(&self) -> usize {
        let mut time: usize;
        unsafe {
            if HAS_RDTIME {
                asm!(
                    "rdtime {time}",
                    time = out(reg) time,
                );
            } else {
                asm!(
                    "rdcycle {time}",
                    time = out(reg) time,
                );
            }
        }
        time
    }
}

impl time::TimeCounter for RISCVTimer {
    fn accuracy(&self) -> Duration {
        // empirically measure timer accuracy
        const SAMPLE_SIZE: usize = 1_000_000;
        let mut diff_total: usize = 0;
        for _ in 0..SAMPLE_SIZE {
            let d1 = self.raw_time();
            let d2 = self.raw_time();
            diff_total += d2 - d1;
        }
        Duration::from_nanos((diff_total / SAMPLE_SIZE * NANOS_PER_TICK / TICKS_PER_NANO) as u64)
    }
    fn uptime(&self) -> Duration {
        Duration::from_nanos((self.raw_time() * NANOS_PER_TICK / TICKS_PER_NANO) as u64)
    }
    fn wait_for(&self, duration: Duration) {
        let begin = self.uptime();
        while self.uptime() - begin < duration { /* spin */ }
    }
}
