use core::cell::UnsafeCell;

/// A RISC-V atomic instruction based lock.
/// All of the methods only take &self instead of &mut self
/// since shared access is required for a lock to be useful in the
/// first place.
// .0 = whether the lock is held
pub struct Lock(UnsafeCell<u8>);

impl Lock {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(0))
    }

    /// Try to acquire this lock.
    pub fn try_acquire(&self) -> bool {
        let mut previous_value: u8 = 1;
        unsafe {
            asm!("amoswap.w.aq {0}, {0}, ({1})", inout(reg) previous_value, in(reg) self.0.get());
        }
        previous_value == 0
    }

    /// Spin until the lock is acquired.
    pub fn spin_lock(&self) {
        loop {
            if self.try_acquire() {
                break;
            }
        }
    }

    /// Unlock the lock, regardless of its previous state.
    pub fn unlock(&self) {
        let mut _value: u8 = 0;
        unsafe {
            asm!("amoswap.w.rl {0}, {0}, ({1})", inout(reg) _value, in(reg) self.0.get());
        }
    }

    /// Tells whether the lock is held.
    /// This is probably not safe with respect to atomic ordering.
    pub fn is_held(&self) -> bool {
        unsafe { *self.0.get() != 0 }
    }
}

unsafe impl Sync for Lock {}
