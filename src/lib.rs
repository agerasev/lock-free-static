//! ## Examples
//!
//! ## Static cell
//!
//! ```
//! use lock_free_static::OnceCell;
//!
//! static VAR: OnceCell<i32> = OnceCell::new();
//!
//! fn main() {
//!     VAR.set(123).unwrap();
//!     assert_eq!(*VAR.get().unwrap(), 123);
//! }
//! ```
//!
//! ## Mutable static cell
//!
//! ```
//! use lock_free_static::OnceMut;
//!
//! static VAR: OnceMut<i32> = OnceMut::new();
//!
//! fn main() {
//!     VAR.set(123).unwrap();
//!
//!     let mut guard = VAR.lock().unwrap();
//!     assert_eq!(*guard, 123);
//!     *guard = 321;
//!     drop(guard);
//!
//!     assert_eq!(*VAR.lock().unwrap(), 321);
//! }
//! ```

#![no_std]

#[cfg(any(test, doc))]
extern crate std;

mod base;
mod cell;
mod mut_;

pub use base::*;
pub use cell::*;
pub use mut_::*;
