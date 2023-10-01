use crate::Defer;
use core::{
    cell::UnsafeCell,
    mem::{forget, MaybeUninit},
    panic::{RefUnwindSafe, UnwindSafe},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct OnceMut<T> {
    slot: UnsafeCell<MaybeUninit<T>>,
    ctor: fn() -> T,
    init: AtomicBool,
}

unsafe impl<T: Send> Send for OnceMut<T> {}
unsafe impl<T: Send + Sync> Sync for OnceMut<T> {}

impl<T> OnceMut<T> {
    pub const fn new(ctor: fn() -> T) -> Self {
        Self {
            slot: UnsafeCell::new(MaybeUninit::uninit()),
            ctor,
            init: AtomicBool::new(false),
        }
    }

    pub fn take(&self) -> Option<&mut T> {
        if self.init.swap(true, Ordering::AcqRel) {
            None
        } else {
            let defer = Defer::new(|| self.init.store(false, Ordering::Release));
            let value = (self.ctor)();
            forget(defer);

            let slot = unsafe { &mut *self.slot.get() };
            *slot = MaybeUninit::new(value);
            Some(unsafe { slot.assume_init_mut() })
        }
    }
}

impl<T> Drop for OnceMut<T> {
    fn drop(&mut self) {
        if self.init.swap(false, Ordering::Relaxed) {
            drop(unsafe { ptr::read(self.slot.get()).assume_init() });
        }
    }
}

impl<T: UnwindSafe> UnwindSafe for OnceMut<T> {}
impl<T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for OnceMut<T> {}

#[macro_export]
macro_rules! once_mut {
    ($(#[$attr:meta])* $vis:vis static mut $ident:ident: $ty:ty = $expr:expr; $($next:tt)*) => {
        $(#[$attr])* $vis static $ident: $crate::OnceMut<$ty> = $crate::OnceMut::new(|| $expr);
        $crate::once_mut!($($next)*);
    };
    () => {};
}

#[cfg(test)]
mod tests {
    use crate::once_mut;

    once_mut! {
        static mut SIMPLE: i32 = 123;
    }

    #[test]
    fn simple() {
        let value_mut = SIMPLE.take().unwrap();
        assert_eq!(*value_mut, 123);
        assert!(SIMPLE.take().is_none());
        *value_mut = 321;
        assert_eq!(*value_mut, 321);
    }

    once_mut! {
        static mut ONE: i32 = 1;
        static mut TWO: i32 = 2;
    }

    #[test]
    fn multiple() {
        assert_eq!(*ONE.take().unwrap(), 1);
        assert!(ONE.take().is_none());
        assert_eq!(*TWO.take().unwrap(), 2);
    }

    mod outer {
        once_mut! {
            pub static mut INNER: i32 = -1;
        }
    }

    #[test]
    fn visibility() {
        assert_eq!(*outer::INNER.take().unwrap(), -1);
        assert!(outer::INNER.take().is_none());
    }
}
