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
// Strict clippy
#![warn(
    clippy::pedantic,
    // clippy::cargo,
    // clippy::nursery,
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
    clippy::cast_possible_truncation,
    // Below items are from "restriction"
    clippy::missing_docs_in_private_items,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::implicit_return,
    clippy::integer_arithmetic,
    clippy::exhaustive_structs,
    clippy::shadow_unrelated,
)]

extern crate udf_macros;
pub use udf_macros::register;

pub mod ffi;
pub mod prelude;
pub mod traits;
pub mod types;
pub use traits::*;
// Make this inline so we don't show the re-exports
// #[doc(inline)]
pub use types::*;
