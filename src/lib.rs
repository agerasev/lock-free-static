//! ## Examples
//!
//! ### Static variable
//!
//! ```
//! # use lock_free_static::lock_free_static;
//! #
//! lock_free_static!{
//!     static VAR: i32 = 123;
//! }
//!
//! fn main() {
//!     assert!(VAR.init());
//!     assert_eq!(*VAR.get().unwrap(), 123);
//! }
//! ```
//!
//! ### Mutable static variable
//!
//! ```
//! # use lock_free_static::lock_free_static;
//! #
//! lock_free_static!{
//!     static mut VAR: i32 = 123;
//! }
//!
//! fn main() {
//!     assert!(VAR.init());
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
mod static_;

pub use base::*;
pub use cell::*;
pub use mut_::*;
pub use static_::*;
