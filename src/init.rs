use crate::{OnceBase, OnceCell, OnceMut};
use core::ops::Deref;

pub struct OnceInit<T, D: Deref<Target = OnceBase<T>>> {
    base: D,
    ctor: fn() -> T,
}

impl<T, D: Deref<Target = OnceBase<T>>> OnceInit<T, D> {
    pub const fn new(base: D, ctor: fn() -> T) -> Self {
        Self { base, ctor }
    }
}

impl<T> OnceInit<T, OnceCell<T>> {
    pub fn get(&self) -> Option<&T> {
        self.base.get_or_init(self.ctor).ok()
    }
}

impl<T> OnceInit<T, OnceMut<T>> {
    pub fn get_mut(&self) -> Option<&mut T> {
        self.base.get_mut_or_init(self.ctor).ok()
    }
}
