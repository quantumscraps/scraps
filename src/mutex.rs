use crate::lock::Lock;
use core::borrow::Borrow;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

/// A mutex based on the locking primitives in [lock].
pub struct Mutex<T> {
    lock: Lock,
    value: T,
}

impl<T> Mutex<T> {
    pub const fn new(val: T) -> Self {
        Mutex {
            lock: Lock::new(),
            value: val,
        }
    }

    fn mg(&self) -> MutexGuard<'_, T> {
        MutexGuard {
            ptr: &self.value as *const _ as *mut _,
            lock: &self.lock,
            phantom: PhantomData,
        }
    }

    /// Try to get the value in this mutex.
    pub fn try_get(&self) -> Option<MutexGuard<'_, T>> {
        if self.lock.try_acquire() {
            Some(self.mg())
        } else {
            None
        }
    }

    /// Spin lock to get the value in this mutex.
    pub fn get(&self) -> MutexGuard<'_, T> {
        self.lock.spin_lock();
        self.mg()
    }
}

unsafe impl<T> Sync for Mutex<T> {}

/// A RAII guard for mutex items.
pub struct MutexGuard<'a, T: 'a> {
    ptr: *mut T,
    lock: &'a Lock,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
