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
//! 1.65 is scheduled to become stable on November 03 2022 UTC, so this message
//! may not be relevant not long after time of writing.
//!
//! # Example
//!
//! Your struct type should hold anything that you want to carry between the
//! functions.
//!
//!
//! ```
//!
//!
//! ```
//!
//! # Behind the Scenes
//!
//! Store the struct to the *ptr before exit
//!
//! Define the basic traits here
//!

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate udf_derive;
pub use udf_derive::register;

pub mod ffi;
pub mod mock;
pub mod prelude;
pub mod types;

// Make this inline so we don't show the re-exports
#[doc(inline)]
pub use types::*;
