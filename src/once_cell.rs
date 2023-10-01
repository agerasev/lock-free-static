use crate::Defer;
use core::{
    cell::UnsafeCell,
    mem::{forget, MaybeUninit},
    panic::{RefUnwindSafe, UnwindSafe},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

/// Lock-free thread-safe cell which can be written to only once.
pub struct OnceCell<T> {
    slot: UnsafeCell<MaybeUninit<T>>,
    lock: AtomicBool,
    init: AtomicBool,
}

unsafe impl<T: Send> Send for OnceCell<T> {}
unsafe impl<T: Send + Sync> Sync for OnceCell<T> {}

impl<T> OnceCell<T> {
    /// Creates a new empty cell.
    pub const fn new() -> Self {
        Self {
            slot: UnsafeCell::new(MaybeUninit::uninit()),
            lock: AtomicBool::new(false),
            init: AtomicBool::new(false),
        }
    }

    /// Sets the contents of this cell to `value`.
    ///
    /// Returns `Ok(())` if the cell’s value was set by this call.
    ///
    /// The cell is guaranteed to contain a value when `set` returns `Ok(())`.
    pub fn set(&self, value: T) -> Result<(), T> {
        if self.lock.swap(true, Ordering::AcqRel) {
            Err(value)
        } else {
            let slot = unsafe { &mut *self.slot.get() };
            *slot = MaybeUninit::new(value);
            self.init.store(true, Ordering::Release);
            Ok(())
        }
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty, or being initialized.
    pub fn get(&self) -> Option<&T> {
        if self.init.load(Ordering::Acquire) {
            Some(unsafe { (*self.slot.get()).assume_init_ref() })
        } else {
            None
        }
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.init.load(Ordering::Relaxed) {
            Some(unsafe { (*self.slot.get()).assume_init_mut() })
        } else {
            None
        }
    }

    /// Gets the contents of the cell, initializing it with `ctor` if the cell was empty.
    ///
    /// Returns `None` if the cell is being initialized.
    ///
    /// # Panics
    ///
    /// If `ctor` panics, the panic is propagated to the caller, and the cell remains uninitialized.
    pub fn get_or_init<F: FnOnce() -> T>(&self, ctor: F) -> Option<&T> {
        if self.lock.swap(true, Ordering::AcqRel) {
            if self.init.load(Ordering::Acquire) {
                Some(unsafe { (*self.slot.get()).assume_init_ref() })
            } else {
                None
            }
        } else {
            let unlock = Defer::new(|| self.lock.store(false, Ordering::Release));
            let value = ctor();
            forget(unlock);

            let slot = unsafe { &mut *self.slot.get() };
            *slot = MaybeUninit::new(value);
            self.init.store(true, Ordering::Release);

            Some(unsafe { slot.assume_init_ref() })
        }
    }

    /// Takes the value out of this cell, moving it back to an uninitialized state.
    ///
    /// Has no effect and returns `None` if the cell hasn’t been initialized.
    pub fn take(&mut self) -> Option<T> {
        if self.init.swap(false, Ordering::Relaxed) {
            self.lock.store(false, Ordering::Relaxed);
            Some(unsafe { ptr::read(self.slot.get()).assume_init() })
        } else {
            None
        }
    }

    /// Consumes the cell, returning the wrapped value.
    ///
    /// Returns `None` if the cell was empty.
    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }
}

impl<T> Drop for OnceCell<T> {
    fn drop(&mut self) {
        drop(self.take());
    }
}

impl<T: UnwindSafe> UnwindSafe for OnceCell<T> {}
impl<T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for OnceCell<T> {}

#[cfg(test)]
mod tests {
    use super::OnceCell;

    #[test]
    fn set() {
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

        assert_eq!(*cell.get_or_init(|| 123).unwrap(), 123);
        assert_eq!(*cell.get_or_init(|| 321).unwrap(), 123);
    }

    #[test]
    fn get_or_init_panic() {
        extern crate std;
        use std::panic::catch_unwind;

        let cell = OnceCell::<i32>::new();

        assert_eq!(
            *catch_unwind(|| cell.get_or_init(|| panic!("abc")))
                .unwrap_err()
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
