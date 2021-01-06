use core::{cell::UnsafeCell, ops::Deref};

use spin::Mutex;

//pub mod mmio;

/// A Mutex wrapper that allows it
/// (in some cases) to be borrowed without locking.
pub struct UnsafeMutex<T>(UnsafeCell<Mutex<T>>);

impl<T> Deref for UnsafeMutex<T> {
    type Target = Mutex<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

impl<T> UnsafeMutex<T> {
    pub const fn new(val: T) -> Self {
        Self(UnsafeCell::new(Mutex::new(val)))
    }

    /// Borrows this Mutex's value, mutably, without locking.
    ///
    /// # Safety
    /// You **must ensure** there are no other references
    /// before using this function. Since it does not lock,
    /// it cannot check whether it is already in use.
    pub unsafe fn get_mut(&self) -> &mut T {
        (&mut *self.0.get()).get_mut()
    }
}

unsafe impl<T> Sync for UnsafeMutex<T> {}
unsafe impl<T> Send for UnsafeMutex<T> {}
