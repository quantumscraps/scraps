use crate::time;
use core::time::Duration;

pub struct ARMv8Timer;

static TIME_COUNTER: ARMv8Timer = ARMv8Timer;

pub fn time_counter() -> &'static impl time::TimeCounter {
    &TIME_COUNTER
}
impl time::TimeCounter for ARMv8Timer {
    fn accuracy(&self) -> Duration {
        let freq: u64;
        unsafe {
            asm!(
                "mrs {fq}, cntfrq_el0",
                fq = out(reg) freq,
            );
        }
        Duration::from_nanos(1_000_000_000 / freq)
    }
    fn uptime(&self) -> Duration {
        let freq: u64;
        let mut count: u64;
        unsafe {
            asm!(
                "mrs {fq}, cntfrq_el0",
                "mrs {pct}, cntpct_el0",
                fq = out(reg) freq,
                pct = out(reg) count,
            );
        }
        count *= 1_000_000_000;
        Duration::from_nanos(count / freq)
    }

    fn wait_for(&self, duration: Duration) {
        if duration.as_nanos() == 0 {
            return;
        }
        let freq: u64;
        unsafe {
            asm!(
                "mrs {fq}, cntfrq_el0",
                fq = out(reg) freq,
            );
        }
        let x = match freq.checked_mul(duration.as_nanos() as u64) {
            None => {
                // spin is too long because it overflowed
                return;
            }
            Some(val) => val,
        };
        let tval = x / 1_000_000_000;
        if tval == 0 || tval > u32::max_value().into() {
            return;
        }
        let mut res: u64;
        let mut _clob: u64 = 0;
        unsafe {
            asm!(
                "msr cntp_tval_el0, {timeval}", // load the timer value
                "mov {scratch}, 0b11",
                "msr cntp_ctl_el0, {scratch}", // enable counting but mask the interrupt since we can't handle it just yet
                timeval = in(reg) tval as u32,
                scratch = inout(reg) _clob,
            );
        }
        loop {
            unsafe {
                asm!(
                    "mrs {result}, cntp_ctl_el0",
                    result = out(reg) res,
                );
            }
            if res & 0b100 == 0b100 {
                // if the status met bit is set
                break;
            }
        }
        unsafe {
            asm!(
                "mov {scratch}, 0",
                "msr cntp_ctl_el0, {scratch}",
                scratch = inout(reg) _clob,
            ); // disable counting again
        }
    }
}
