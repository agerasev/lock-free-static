use crate::{Mutex, MutexGuard, OnceCell};
use core::cell::Cell;

/// Like [`OnceCell`](crate::OnceCell) but with exclusive mutable access to its content.
pub struct OnceMut<T> {
    cell: OnceCell<Mutex<T>>,
}

impl<T> OnceMut<T> {
    /// Creates a new empty cell.
    pub const fn new() -> Self {
        Self {
            cell: OnceCell::new(),
        }
    }

    /// Sets the contents of this cell to `value`.
    ///
    /// Returns `Ok(())` if the cell’s value was set by this call.
    pub fn set(&self, value: T) -> Result<(), T> {
        self.cell.set(Mutex::new(value)).map_err(Mutex::into_inner)
    }

    /// Sets the contents of this cell to value returned by `ctor` call.
    ///
    /// The `ctor` is called only if the cell’s value is going set by this call. Otherwice `ctor` returned in `Err(..)`.
    ///
    /// # Panics
    ///
    /// If `ctor` panics, the panic is propagated to the caller, and the cell remains uninitialized.
    pub fn set_with<F: FnOnce() -> T>(&self, ctor: F) -> Result<(), F> {
        let cell = Cell::new(Some(ctor));
        self.cell
            .set_with(|| Mutex::new(cell.take().unwrap()()))
            .map_err(|_| cell.take().unwrap())
    }

    /// Takes the value out of this cell, moving it back to an uninitialized state.
    ///
    /// Has no effect and returns `None` if the cell hasn’t been initialized.
    pub fn take(&mut self) -> Option<T> {
        self.cell.take().map(Mutex::into_inner)
    }

    /// Gets the pointer to the underlying value.
    ///
    /// Returns `None` if the cell is empty.
    pub fn get_ptr(&self) -> Option<*mut T> {
        self.cell.get().map(Mutex::get_ptr)
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// The main difference from [`OnceCell::get_mut`](`crate::OnceCell::get_mut`) is that `self` is taken as immutable.
    ///
    /// After this call returns `Some(..)`, all subsequent calls will return `None`, and there is no way to obtain mutable reference again.
    pub fn get_mut(&self) -> Option<&mut T> {
        self.cell
            .get()
            .and_then(Mutex::try_lock)
            .map(MutexGuard::leak)
    }

    /// Gets a guarded mutable reference to the underlying value.
    ///
    /// Only one guard of the same value can exist at the same time.
    pub fn lock(&self) -> Option<LockGuard<'_, T>> {
        self.cell.get().and_then(Mutex::try_lock)
    }

    /// Consumes the cell, returning the wrapped value.
    ///
    /// Returns `None` if the cell was empty.
    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }
}

/// [`OnceMut`] lock guard.
pub type LockGuard<'a, T> = MutexGuard<'a, T>;

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
