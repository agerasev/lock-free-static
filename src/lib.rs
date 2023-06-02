#![no_std]

use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct OnceMut<T> {
    cell: UnsafeCell<MaybeUninit<T>>,
    init: fn() -> T,
    taken: AtomicBool,
}

unsafe impl<T> Sync for OnceMut<T> {}

impl<T> OnceMut<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            init,
            cell: UnsafeCell::new(MaybeUninit::uninit()),
            taken: AtomicBool::new(false),
        }
    }

    pub fn take(&self) -> Option<&mut T> {
        if self.taken.swap(true, Ordering::AcqRel) {
            return None;
        }
        let value = (self.init)();
        let slot = unsafe { &mut *self.cell.get() };
        *slot = MaybeUninit::new(value);
        Some(unsafe { slot.assume_init_mut() })
    }
}

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
        static mut ONE: i32 = 1;
        static mut TWO: i32 = 2;
    }

    #[test]
    fn it_works() {
        assert_eq!(*ONE.take().unwrap(), 1);
        assert!(ONE.take().is_none());
        assert_eq!(*TWO.take().unwrap(), 2);
    }
}
