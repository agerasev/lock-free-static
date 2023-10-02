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

    /// Gets the contents of the cell, initializing it with `ctor` if the cell was empty.
    ///
    /// Returns `None` if the cell is being currently initialized.
    ///
    /// # Panics
    ///
    /// If `ctor` panics, the panic is propagated to the caller, and the cell remains uninitialized.
    pub fn get_or_init<F: FnOnce() -> T>(&self, ctor: F) -> Result<&T, F> {
        self.base.get_ptr_or_init(ctor).map(|p| unsafe { &*p })
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
    fn get_or_init() {
        let cell = OnceCell::<i32>::new();

        assert_eq!(*cell.get_or_init(|| 123).ok().unwrap(), 123);
        assert_eq!(*cell.get_or_init(|| 321).ok().unwrap(), 123);
    }

    #[test]
    fn get_or_init_panic() {
        extern crate std;
        use std::panic::catch_unwind;

        let cell = OnceCell::<i32>::new();

        assert_eq!(
            *catch_unwind(|| cell.get_or_init(|| panic!("abc")))
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
