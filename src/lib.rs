#![no_std]

mod once_cell;
mod once_mut;

pub use once_cell::*;
pub use once_mut::*;

use core::mem::ManuallyDrop;

pub(crate) struct Defer<F: FnOnce()> {
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
