use crate::UnsafeOnceCell;
use core::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

/// Lock-free thread-safe cell which can mutably borrowed only once.
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
    pub const fn new() -> Self {
        Self {
            base: UnsafeOnceCell::new(),
            borrowed: AtomicBool::new(false),
        }
    }

    pub fn get_mut_or_init<F: FnOnce() -> T>(&self, ctor: F) -> Result<&mut T, F> {
        if self.borrowed.swap(true, Ordering::AcqRel) {
            Err(ctor)
        } else {
            match self.base.get_ptr_or_init(ctor) {
                Ok(ptr) => Ok(unsafe { &mut *ptr }),
                Err(ctor) => {
                    self.borrowed.store(false, Ordering::Release);
                    Err(ctor)
                }
            }
        }
    }
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
}

#[cfg(test)]
mod tests {
    use super::OnceMut;

    #[test]
    fn set_get_mut() {
        let cell = OnceMut::<i32>::new();
        assert!(cell.get_mut().is_none());

        cell.set(123).unwrap();

        let value_mut = cell.get_mut().unwrap();
        assert_eq!(*value_mut, 123);
        assert!(cell.get_mut().is_none());
        *value_mut = 321;
        assert_eq!(*value_mut, 321);
    }
}
