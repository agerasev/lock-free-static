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

mod mutex;
mod once_cell;
mod once_mut;
mod static_;

pub use mutex::*;
pub use once_cell::*;
pub use once_mut::*;
pub use static_::*;
