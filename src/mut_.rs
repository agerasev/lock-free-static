use crate::UnsafeOnceCell;
use core::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

/// Like [`OnceCell`](crate::OnceCell) but with exclusive mutable access to its content.
pub struct OnceMut<T> {
    base: UnsafeOnceCell<T>,
    borrowed: AtomicBool,
}

impl<T> Deref for OnceMut<T> {
    type Target = UnsafeOnceCell<T>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl<T> DerefMut for OnceMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<T> OnceMut<T> {
    /// Creates a new empty cell.
    pub const fn new() -> Self {
        Self {
            base: UnsafeOnceCell::new(),
            borrowed: AtomicBool::new(false),
        }
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// The main difference from [`OnceCell::get_mut`](`crate::OnceCell::get_mut`) is that `self` is taken as immutable.
    ///
    /// After this call returns `Some(..)`, all subsequent calls will return `None`, and there is no way to obtain mutable reference again.
    pub fn get_mut(&self) -> Option<&mut T> {
        if self.borrowed.swap(true, Ordering::AcqRel) {
            None
        } else {
            match self.base.get_ptr() {
                Some(ptr) => Some(unsafe { &mut *ptr }),
                None => {
                    self.borrowed.store(false, Ordering::Release);
                    None
                }
            }
        }
    }

    /// Gets a guarded mutable reference to the underlying value.
    ///
    /// Only one guard of the same value can exist at the same time.
    pub fn lock(&self) -> Option<LockGuard<'_, T>> {
        if self.borrowed.swap(true, Ordering::AcqRel) {
            None
        } else {
            match self.base.get_ptr() {
                Some(ptr) => Some(LockGuard {
                    value: ptr,
                    owner: self,
                }),
                None => {
                    self.borrowed.store(false, Ordering::Release);
                    None
                }
            }
        }
    }

    /// Consumes the cell, returning the wrapped value.
    ///
    /// Returns `None` if the cell was empty.
    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }
}

/// [`OnceMut`] lock guard.
pub struct LockGuard<'a, T> {
    owner: &'a OnceMut<T>,
    value: *mut T,
}

unsafe impl<'a, T: Sync> Send for LockGuard<'a, T> {}
unsafe impl<'a, T: Sync> Sync for LockGuard<'a, T> {}

impl<'a, T> Deref for LockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}
impl<'a, T> DerefMut for LockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}

impl<'a, T> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        self.owner.borrowed.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::OnceMut;

    #[test]
    fn get_mut() {
        let cell = OnceMut::<i32>::new();
        assert!(cell.get_mut().is_none());

        cell.set(123).unwrap();

        let value_mut = cell.get_mut().unwrap();
        assert_eq!(*value_mut, 123);
        assert!(cell.get_mut().is_none());
        *value_mut = 321;
        assert_eq!(*value_mut, 321);
    }

    #[test]
    fn lock() {
        let cell = OnceMut::<i32>::new();
        assert!(cell.lock().is_none());

        cell.set(123).unwrap();

        let mut guard = cell.lock().unwrap();
        assert_eq!(*guard, 123);
        assert!(cell.lock().is_none());
        *guard = 321;
        assert_eq!(*guard, 321);
        drop(guard);

        assert_eq!(*cell.lock().unwrap(), 321);
    }
}
