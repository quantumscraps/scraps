pub use crate::arch::time::*;
use core::time::Duration;

pub trait TimeCounter {
    fn accuracy(&self) -> Duration;
    fn uptime(&self) -> Duration;
    fn wait_for(&self, duration: Duration);
}
