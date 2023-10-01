use crate::OnceBase;
use core::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

/// Lock-free thread-safe cell which can mutably borrowed only once.
pub struct OnceMut<T> {
    base: OnceBase<T>,
    borrowed: AtomicBool,
}

impl<T> Deref for OnceMut<T> {
    type Target = OnceBase<T>;
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
            base: OnceBase::new(),
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

#[macro_export]
macro_rules! once_mut {
    ($(#[$attr:meta])* $vis:vis static mut $ident:ident: $ty:ty = $expr:expr; $($next:tt)*) => {
        $(#[$attr])*
        $vis static $ident: $crate::OnceInit<$ty, $crate::OnceMut<$ty>>
            = $crate::OnceInit::new($crate::OnceMut::new(), || $expr);
        $crate::once_mut!($($next)*);
    };
    () => {};
}

#[cfg(test)]
mod tests {
    once_mut! {
        static mut SIMPLE: i32 = 123;
    }

    #[test]
    fn simple() {
        let value_mut = SIMPLE.get_mut().unwrap();
        assert_eq!(*value_mut, 123);
        assert!(SIMPLE.get_mut().is_none());
        *value_mut = 321;
        assert_eq!(*value_mut, 321);
    }

    once_mut! {
        static mut ONE: i32 = 1;
        static mut TWO: i32 = 2;
    }

    #[test]
    fn multiple() {
        assert_eq!(*ONE.get_mut().unwrap(), 1);
        assert!(ONE.get_mut().is_none());
        assert_eq!(*TWO.get_mut().unwrap(), 2);
    }

    mod outer {
        once_mut! {
            pub static mut INNER: i32 = -1;
        }
    }

    #[test]
    fn visibility() {
        assert_eq!(*outer::INNER.get_mut().unwrap(), -1);
        assert!(outer::INNER.get_mut().is_none());
    }
}
