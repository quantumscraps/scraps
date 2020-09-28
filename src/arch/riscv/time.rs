use crate::time;
use crate::bsp::{NANOS_PER_TICK, HAS_RDTIME};
use core::time::Duration;

pub struct RISCVTimer;

static TIME_COUNTER: RISCVTimer = RISCVTimer;

pub fn time_counter() -> &'static impl time::TimeCounter {
    &TIME_COUNTER
}

impl time::TimeCounter for RISCVTimer {
    fn accuracy(&self) -> Duration {
        // empirically measure timer accuracy
        const SAMPLE_SIZE: usize = 10_000;
        let mut durations: [Duration; SAMPLE_SIZE] = [Duration::zero(); SAMPLE_SIZE];
        let mut durations2: [Duration; SAMPLE_SIZE] = [Duration::zero(); SAMPLE_SIZE];
        for i in 0..SAMPLE_SIZE {
            durations[i] = self.uptime();
            durations2[i] = self.uptime();
        }
        let mut diff_total: u64 = 0;
        for (d1, d2) in durations.iter().zip(durations2.iter()) {
            diff_total += (*d2 - *d1).as_nanos() as u64;
        }
        Duration::from_nanos(diff_total / (SAMPLE_SIZE as u64))
    }
    fn uptime(&self) -> Duration {
        let mut time: u64;
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
        Duration::from_nanos(time * NANOS_PER_TICK)
    }
    fn wait_for(&self, duration: Duration) {
        let begin = self.uptime();
        while self.uptime() - begin < duration {
            /* spin */
        }
    }
}