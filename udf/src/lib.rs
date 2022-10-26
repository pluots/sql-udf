//! A wrapper crate to make writing SQL UDFs easy
//!
//! Version note: Because of reliance on a feature called GATs, this library
//! requires Rust version >= 1.65 which is currently in beta. If `rustup show`
//! does not show 1.65 or greater under active toolchain, you will need to
//! update:
//!
//! ```sh
//! # nightly can also be used instead of beta
//! rustup default beta
//! rustup update beta
//! ```
//!
//! 1.65 is scheduled to become stable on 2022-11-03, so this message
//! may become irrelevant not long after time of writing.
//!
//! # Example
//!
//! Your struct type should hold anything that you want to carry between the
//! functions.
//!
//!
//! ```
//! struct MyFunction {
//!     intermediate: i64
//! }
//!
//! ```

#![deny(unsafe_op_in_unsafe_fn)]
// Strict clippy
#![warn(
    clippy::pedantic,
    // clippy::cargo,
    clippy::nursery,
    clippy::str_to_string,
    clippy::missing_inline_in_public_items,
    clippy::exhaustive_enums,
    clippy::pattern_type_mismatch
)]
// Pedantic config
#![allow(
    clippy::missing_const_for_fn,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::cast_possible_truncation
)]

pub extern crate udf_sys;
// #[doc(hidden)]
// pub use udf_sys;

extern crate udf_macros;
pub use udf_macros::register;

pub mod prelude;
pub mod traits;
pub mod types;

// We hide this because it's really only used by our proc macros
#[doc(hidden)]
pub mod wrapper;

pub use traits::*;
pub use types::{MYSQL_ERRMSG_SIZE, *};
