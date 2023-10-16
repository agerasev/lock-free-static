use core::{
    cell::UnsafeCell,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    panic::{RefUnwindSafe, UnwindSafe},
    sync::atomic::{AtomicBool, Ordering},
};

/// Lock-free mutex.
///
/// Does not provide waiting mechanism.
pub struct Mutex<T: ?Sized> {
    locked: AtomicBool,
    inner: UnsafeCell<T>,
}

unsafe impl<T: Send + ?Sized> Send for Mutex<T> {}
unsafe impl<T: Send + ?Sized> Sync for Mutex<T> {}

impl<T: ?Sized> UnwindSafe for Mutex<T> {}
impl<T: ?Sized> RefUnwindSafe for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    pub const fn new(item: T) -> Self {
        Self {
            inner: UnsafeCell::new(item),
            locked: AtomicBool::new(false),
        }
    }

    /// Consumes this mutex, returning the underlying data.
    pub fn into_inner(self) -> T {
        debug_assert!(!self.locked.load(Ordering::Acquire));
        self.inner.into_inner()
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Returns a pointer to the underlying data.
    pub fn get_ptr(&self) -> *mut T {
        self.inner.get()
    }

    /// Returns a mutable reference to the underlying data.
    pub fn get_mut(&mut self) -> &mut T {
        debug_assert!(!self.locked.load(Ordering::Acquire));
        self.inner.get_mut()
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then `None` is returned.
    /// Otherwise, an RAII guard is returned. The lock will be unlocked when the guard is dropped.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.locked.swap(true, Ordering::AcqRel) {
            None
        } else {
            Some(MutexGuard { owner: self })
        }
    }
}

/// [`Mutex`] lock guard.
///
/// When it is dropped, the lock will be unlocked.
pub struct MutexGuard<'a, T: ?Sized> {
    owner: &'a Mutex<T>,
}

unsafe impl<'a, T: Sync + ?Sized> Send for MutexGuard<'a, T> {}
unsafe impl<'a, T: Sync + ?Sized> Sync for MutexGuard<'a, T> {}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.owner.get_ptr() }
    }
}
impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.owner.get_ptr() }
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.owner.locked.store(false, Ordering::Release);
    }
}

impl<'a, T: ?Sized> MutexGuard<'a, T> {
    /// Returns a mutable reference to the data protected by the mutex and consumes the guard.
    ///
    /// The mutex will remain in a locked state forever after this call.
    pub fn leak(self) -> &'a mut T {
        let this = ManuallyDrop::new(self);
        unsafe { &mut *this.owner.get_ptr() }
    }
}

#[cfg(test)]
mod tests {
    use super::Mutex;

    #[test]
    fn try_lock() {
        let mutex = Mutex::<i32>::new(123);

        let mut guard = mutex.try_lock().unwrap();
        assert_eq!(*guard, 123);
        assert!(mutex.try_lock().is_none());
        *guard = 321;
        assert_eq!(*guard, 321);
        drop(guard);

        assert_eq!(*mutex.try_lock().unwrap(), 321);
    }

    #[test]
    fn leak() {
        let mutex = Mutex::<i32>::new(123);
        let value_mut = mutex.try_lock().unwrap().leak();
        assert_eq!(*value_mut, 123);
        assert!(mutex.try_lock().is_none());
        *value_mut = 321;
        assert_eq!(*value_mut, 321);
    }
}
