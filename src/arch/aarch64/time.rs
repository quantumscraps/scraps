use crate::time;
use core::time::Duration;
use cortex_a::regs::*;

pub struct ARMv8Timer;

static TIME_COUNTER: ARMv8Timer = ARMv8Timer;

pub fn time_counter() -> &'static impl time::TimeCounter {
    &TIME_COUNTER
}
impl time::TimeCounter for ARMv8Timer {
    fn accuracy(&self) -> Duration {
        Duration::from_nanos(1_000_000_000 / (CNTFRQ_EL0.get() as u64))
    }
    fn uptime(&self) -> Duration {
        Duration::from_nanos(
            (CNTPCT_EL0.get() as u64) * 1_000_000_000 / (CNTFRQ_EL0.get() as u64)
        )
    }

    fn wait_for(&self, duration: Duration) {
        if duration.as_nanos() == 0 {
            return;
        }
        let freq: u64 = CNTFRQ_EL0.get();
        let ticks = match freq.checked_mul(duration.as_nanos() as u64) {
            None => {
                // spin is too long because it overflowed
                return;
            }
            Some(val) => val,
        };
        let tval = ticks / 1_000_000_000;
        if tval == 0 || tval > u32::max_value().into() {
            return;
        }
        CNTP_TVAL_EL0.set(tval); // load the timer value
        CNTP_CTL_EL0.set(0b11); // enable time counting and mask the interrupt for now.
        loop {
            // keep checking the ISTATUS bit in a spin loop.
            // in the future we will be using interrupts and not masking the interrupt LUL
            if CNTP_CTL_EL0.matches_all(CNTP_CTL_EL0::ISTATUS::SET) {
                break;
            }
        }
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::CLEAR); // disable time counting
    }
}
