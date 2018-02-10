use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::UnsafeCell;
use std::ops::{DerefMut, Deref, Drop};

#[repr(align(32))]
pub struct Mutex<T> {
    data: UnsafeCell<T>,
    #[allow(unused)]
    lock: AtomicBool,
}

unsafe impl<T: Send> Send for Mutex<T> { }
unsafe impl<T: Send> Sync for Mutex<T> { }

pub struct MutexGuard<'a, T: 'a> {
    lock: &'a Mutex<T>
}

impl<'a, T> !Send for MutexGuard<'a, T> { }
unsafe impl<'a, T: Sync> Sync for MutexGuard<'a, T> { }

impl<T> Mutex<T> {
    pub const fn new(val: T) -> Mutex<T> {
        Mutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(val)
        }
    }
}

impl<T> Mutex<T> {
    // Once MMU/cache is enabled, do the right thing here. For now, we don't
    // need any real synchronization.
    #[inline(never)]
    pub fn lock(&self) -> MutexGuard<T> {
        // Wait until we can "aquire" the lock, then "acquire" it.
        while self.lock.load(Ordering::Relaxed) { }
        self.lock.store(true, Ordering::Relaxed);

        MutexGuard { lock: &self }
    }

    fn unlock(&self) {
        self.lock.store(false, Ordering::Relaxed);
    }
}

impl<'a, T: 'a> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { & *self.lock.data.get() }
    }
}

impl<'a, T: 'a> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T: 'a> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock()
    }
}
