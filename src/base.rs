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

pub struct UnsafeOnceCell<T> {
    slot: UnsafeCell<MaybeUninit<T>>,
    lock: AtomicBool,
    init: AtomicBool,
}

unsafe impl<T: Send> Send for UnsafeOnceCell<T> {}
unsafe impl<T: Send + Sync> Sync for UnsafeOnceCell<T> {}

impl<T> UnsafeOnceCell<T> {
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

    /// Gets the contents of the cell, initializing it with `ctor` if the cell was empty.
    ///
    /// Returns `None` if the cell is being currently initialized.
    ///
    /// # Panics
    ///
    /// If `ctor` panics, the panic is propagated to the caller, and the cell remains uninitialized.
    pub fn get_ptr_or_init<F: FnOnce() -> T>(&self, ctor: F) -> Result<*mut T, F> {
        if self.lock.swap(true, Ordering::AcqRel) {
            if self.init.load(Ordering::Acquire) {
                Ok(self.slot.get() as *mut T)
            } else {
                Err(ctor)
            }
        } else {
            let unlock = Defer::new(|| self.lock.store(false, Ordering::Release));
            let value = ctor();
            forget(unlock);

            let slot = unsafe { &mut *self.slot.get() };
            *slot = MaybeUninit::new(value);
            self.init.store(true, Ordering::Release);

            Ok(slot as *mut _ as *mut T)
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
}

impl<T> Drop for UnsafeOnceCell<T> {
    fn drop(&mut self) {
        drop(self.take());
    }
}

impl<T: UnwindSafe> UnwindSafe for UnsafeOnceCell<T> {}
impl<T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for UnsafeOnceCell<T> {}
