use crate::{OnceCell, OnceMut};
use core::ops::Deref;

/// Convenience wrapper for static initialization of cells.
///
/// Should be explicitly initialized (by [`init`](LockFreeStatic::init) call) because
/// initialization is fallible and therefore cannot be done automatically on dereference.
pub struct LockFreeStatic<T, D, F: Fn() -> T = fn() -> T> {
    base: D,
    ctor: F,
}

impl<T, D, F: Fn() -> T> Deref for LockFreeStatic<T, D, F> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<T, D, F: Fn() -> T> LockFreeStatic<T, D, F> {
    /// Creates a new wrapper.
    pub const fn new(base: D, ctor: F) -> Self {
        Self { base, ctor }
    }
}

impl<T, F: Fn() -> T> LockFreeStatic<T, OnceCell<T>, F> {
    /// Initializes the underlying cell.
    ///
    /// `true` is returned if the cell was initialized by this call.
    #[must_use]
    pub fn init(&self) -> bool {
        self.base.set((self.ctor)()).is_ok()
    }
}

impl<T, F: Fn() -> T> LockFreeStatic<T, OnceMut<T>, F> {
    /// Initializes the underlying cell.
    ///
    /// `true` is returned if the cell was initialized by this call.
    #[must_use]
    pub fn init(&self) -> bool {
        self.base.set((self.ctor)()).is_ok()
    }
}

/// Convenience macro for creating lock-free statics.
#[macro_export]
macro_rules! lock_free_static {
    ($(#[$attr:meta])* $vis:vis static $ident:ident: $ty:ty = $expr:expr; $($next:tt)*) => {
        $(#[$attr])*
        $vis static $ident: $crate::LockFreeStatic<$ty, $crate::OnceCell<$ty>>
            = $crate::LockFreeStatic::new($crate::OnceCell::new(), || $expr);
        $crate::lock_free_static!($($next)*);
    };
    ($(#[$attr:meta])* $vis:vis static mut $ident:ident: $ty:ty = $expr:expr; $($next:tt)*) => {
        $(#[$attr])*
        $vis static $ident: $crate::LockFreeStatic<$ty, $crate::OnceMut<$ty>>
            = $crate::LockFreeStatic::new($crate::OnceMut::new(), || $expr);
        $crate::lock_free_static!($($next)*);
    };
    () => {};
}

#[cfg(test)]
mod tests {
    use crate::lock_free_static;

    lock_free_static! {
        static CONST: i32 = 123;
    }

    #[test]
    fn const_() {
        assert!(CONST.get().is_none());
        assert!(CONST.init());

        let value = CONST.get().unwrap();
        assert_eq!(*value, 123);
        assert_eq!(*CONST.get().unwrap(), 123);
    }

    lock_free_static! {
        static mut MUT: i32 = 123;
    }

    #[test]
    fn mut_() {
        assert!(MUT.lock().is_none());
        assert!(MUT.init());

        let mut guard = MUT.lock().unwrap();
        assert_eq!(*guard, 123);
        assert!(MUT.lock().is_none());
        *guard = 321;
        drop(guard);

        assert_eq!(*MUT.lock().unwrap(), 321);
    }

    lock_free_static! {
        static ONE: i32 = 1;
        static mut TWO: i32 = 2;
    }

    #[test]
    fn multiple() {
        assert!(ONE.init());
        assert!(TWO.init());
        assert_eq!(*ONE.get().unwrap(), 1);
        assert_eq!(*ONE.get().unwrap(), 1);
        assert_eq!(*TWO.get_mut().unwrap(), 2);
        assert!(TWO.get_mut().is_none());
    }

    mod outer {
        use crate::lock_free_static;

        lock_free_static! {
            pub static ONE: i32 = 1;
            pub static mut TWO: i32 = 2;
        }
    }

    #[test]
    fn visibility() {
        assert!(outer::ONE.init());
        assert!(outer::TWO.init());
        assert_eq!(*outer::ONE.get().unwrap(), 1);
        assert_eq!(*outer::TWO.get_mut().unwrap(), 2);
    }
}
