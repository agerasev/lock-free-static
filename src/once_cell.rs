use core::{
    cell::UnsafeCell,
    mem::{forget, ManuallyDrop, MaybeUninit},
    panic::{RefUnwindSafe, UnwindSafe},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

struct Defer<F: FnOnce()> {
    f: ManuallyDrop<F>,
}
impl<F: FnOnce()> Defer<F> {
    pub fn new(f: F) -> Self {
        Self {
            f: ManuallyDrop::new(f),
        }
    }
}
impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        (unsafe { ManuallyDrop::take(&mut self.f) })();
    }
}

/// Lock-free thread-safe cell which can be written to only once.
pub struct OnceCell<T> {
    slot: UnsafeCell<MaybeUninit<T>>,
    lock: AtomicBool,
    init: AtomicBool,
}

unsafe impl<T: Send> Send for OnceCell<T> {}
unsafe impl<T: Send + Sync> Sync for OnceCell<T> {}

impl<T: UnwindSafe> UnwindSafe for OnceCell<T> {}
impl<T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for OnceCell<T> {}

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

    /// Sets the contents of this cell to value returned by `ctor` call.
    ///
    /// The `ctor` is called only if the cell’s value is going set by this call. Otherwice `ctor` returned in `Err(..)`.
    ///
    /// # Panics
    ///
    /// If `ctor` panics, the panic is propagated to the caller, and the cell remains uninitialized.
    pub fn set_with<F: FnOnce() -> T>(&self, ctor: F) -> Result<(), F> {
        if self.lock.swap(true, Ordering::AcqRel) {
            Err(ctor)
        } else {
            let unlock = Defer::new(|| self.lock.store(false, Ordering::Release));
            let value = ctor();
            forget(unlock);

            let slot = unsafe { &mut *self.slot.get() };
            *slot = MaybeUninit::new(value);
            self.init.store(true, Ordering::Release);
            Ok(())
        }
    }

    /// Gets the pointer to the underlying value.
    ///
    /// Returns `None` if the cell is empty.
    pub fn get_ptr(&self) -> Option<*mut T> {
        if self.init.load(Ordering::Relaxed) {
            Some(self.slot.get() as *mut T)
        } else {
            None
        }
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty, or being initialized.
    pub fn get(&self) -> Option<&T> {
        self.get_ptr().map(|p| unsafe { &*p })
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.get_ptr().map(|p| unsafe { &mut *p })
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
