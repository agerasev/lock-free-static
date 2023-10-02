use crate::{OnceCell, OnceMut, UnsafeOnceCell};
use core::ops::Deref;

pub struct LockFreeStatic<T, D: Deref<Target = UnsafeOnceCell<T>>> {
    base: D,
    ctor: fn() -> T,
}

impl<T, D: Deref<Target = UnsafeOnceCell<T>>> LockFreeStatic<T, D> {
    pub const fn new(base: D, ctor: fn() -> T) -> Self {
        Self { base, ctor }
    }
    pub fn init(&self) -> bool {
        self.base.set((self.ctor)()).is_ok()
    }
}

impl<T> LockFreeStatic<T, OnceCell<T>> {
    pub fn get_or_init(&self) -> Option<&T> {
        self.base.get_or_init(self.ctor).ok()
    }
    pub fn get(&self) -> Option<&T> {
        self.base.get()
    }
}

impl<T> LockFreeStatic<T, OnceMut<T>> {
    pub fn get_mut_or_init(&self) -> Option<&mut T> {
        self.base.get_mut_or_init(self.ctor).ok()
    }
    pub fn get_mut(&self) -> Option<&mut T> {
        self.base.get_mut()
    }
}

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
        assert!(MUT.get_mut().is_none());
        assert!(MUT.init());

        let value_mut = MUT.get_mut().unwrap();
        assert_eq!(*value_mut, 123);
        assert!(MUT.get_mut().is_none());
        *value_mut = 321;
        assert_eq!(*value_mut, 321);
    }

    lock_free_static! {
        static ONE: i32 = 1;
        static mut TWO: i32 = 2;
    }

    #[test]
    fn multiple() {
        assert_eq!(*ONE.get_or_init().unwrap(), 1);
        assert_eq!(*ONE.get_or_init().unwrap(), 1);
        assert_eq!(*TWO.get_mut_or_init().unwrap(), 2);
        assert!(TWO.get_mut_or_init().is_none());
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
        assert_eq!(*outer::ONE.get_or_init().unwrap(), 1);
        assert_eq!(*outer::TWO.get_mut_or_init().unwrap(), 2);
    }
}
