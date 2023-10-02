use crate::UnsafeOnceCell;
use core::ops::{Deref, DerefMut};

/// Lock-free thread-safe cell which can be written to only once.
pub struct OnceCell<T> {
    base: UnsafeOnceCell<T>,
}

impl<T> Deref for OnceCell<T> {
    type Target = UnsafeOnceCell<T>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl<T> DerefMut for OnceCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<T> OnceCell<T> {
    /// Creates a new empty cell.
    pub const fn new() -> Self {
        Self {
            base: UnsafeOnceCell::new(),
        }
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty, or being initialized.
    pub fn get(&self) -> Option<&T> {
        self.base.get_ptr().map(|p| unsafe { &*p })
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.base.get_ptr().map(|p| unsafe { &mut *p })
    }

    /// Consumes the cell, returning the wrapped value.
    ///
    /// Returns `None` if the cell was empty.
    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }
}

#[cfg(test)]
mod tests {
    use super::OnceCell;

    #[test]
    fn get() {
        let mut cell = OnceCell::<i32>::new();
        assert!(cell.get().is_none());

        cell.set(123).unwrap();
        assert_eq!(cell.set(321), Err(321));
        assert_eq!(*cell.get().unwrap(), 123);

        {
            let value_mut = cell.get_mut().unwrap();
            assert_eq!(*value_mut, 123);
            *value_mut = 321;
            assert_eq!(*value_mut, 321);
        }
        assert_eq!(*cell.get().unwrap(), 321);
    }

    #[test]
    fn take() {
        let mut cell = OnceCell::<i32>::new();
        assert!(cell.get().is_none());

        cell.set(123).unwrap();
        assert_eq!(cell.set(321), Err(321));
        assert_eq!(*cell.get().unwrap(), 123);

        assert_eq!(cell.take().unwrap(), 123);
        assert!(cell.get().is_none());
        assert!(cell.take().is_none());

        cell.set(321).unwrap();
        assert_eq!(*cell.get().unwrap(), 321);
        assert_eq!(cell.into_inner().unwrap(), 321);
    }

    #[test]
    fn set_with() {
        let cell = OnceCell::<i32>::new();

        assert!(cell.set_with(|| 123).is_ok());
        assert_eq!(*cell.get().unwrap(), 123);
        assert!(cell.set_with(|| 321).is_err());
        assert_eq!(*cell.get().unwrap(), 123);
    }

    #[test]
    fn set_with_panic() {
        extern crate std;
        use std::panic::catch_unwind;

        let cell = OnceCell::<i32>::new();

        assert_eq!(
            *catch_unwind(|| cell.set_with(|| panic!("abc")))
                .err()
                .unwrap()
                .downcast::<&'static str>()
                .unwrap(),
            "abc"
        );
        assert!(cell.get().is_none());

        cell.set(321).unwrap();
        assert_eq!(*cell.get().unwrap(), 321);
    }

    static CELL: OnceCell<i32> = OnceCell::new();

    #[test]
    fn static_() {
        assert!(CELL.get().is_none());

        CELL.set(123).unwrap();
        assert_eq!(CELL.set(321), Err(321));
        assert_eq!(*CELL.get().unwrap(), 123);
    }
}
