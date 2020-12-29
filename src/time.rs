pub use crate::arch::time::*;
use core::time::Duration;

/// Keeps the flow of time.
pub trait TimeCounter {
    /// Returns the smallest representable duration measurable by this counter.
    fn accuracy(&self) -> Duration;
    /// Returns the uptime of the system.
    fn uptime(&self) -> Duration;
    /// Waits for the given duration of time.
    fn wait_for(&self, duration: Duration);
}
