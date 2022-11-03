//! A wrapper crate to make writing SQL user-defined functions (UDFs) easy
//!
//! This crate provides bindings for easy creation of SQL user-defined functions
//! in Rust. See [the
//! readme](https://github.com/pluots/sql-udf/blob/main/README.md) for more
//! background information.
//!
//! # Usage
//!
//! Using this crate is fairly simple: create a struct that will be used to
//! share data among UDF functions (which can be zero-sized), then implement
//! needed traits for it. [`BasicUdf`] provides function signatures for standard
//! UDFs, and [`AggregateUdf`] provides signatures for aggregate (and window)
//! UDFs. See the documentation there for a step-by-step guide.
//!
//! ```
//! use udf::prelude::*;
//!
//! // Our struct that will produce a UDF of name `my_udf`
//! struct MyUdf {}
//!
//! #[register]
//! impl BasicUdf for MyUdf {
//!     // Specify return type of this UDF to be a nullable integer
//!     type Returns<'a> = Option<i64>;
//!
//!     // Perform initialization steps here
//!     fn init<'a>(
//!         cfg: &UdfCfg<Init>,
//!         args: &'a ArgList<'a, Init>
//!     ) -> Result<Self, String> {
//!         todo!();
//!     }
//!
//!     // Create a result here
//!     fn process<'a>(
//!         &'a mut self,
//!         cfg: &UdfCfg<Process>,
//!         args: &ArgList<Process>,
//!         error: Option<NonZeroU8>,
//!     ) -> Result<Self::Returns<'a>, ProcessError> {
//!         todo!();
//!     }
//! }
//! ```
//!
//! # Version Note
//!
//! Because of reliance on a feature called GATs, this library requires Rust
//! version >= 1.65. This only became stable on 2022-11-03; if you encounter
//! issues compiling, be sure to update your toolchain.
//!
//! ```sh
//! rustup update
//! ```

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

// We re-export this so we can use it in our macro, but don't need it
// to show up in our docs
#[doc(hidden)]
pub extern crate chrono;

extern crate udf_macros;
pub extern crate udf_sys;

pub use udf_macros::register;

pub mod prelude;
pub mod traits;
pub mod types;

// We hide this because it's really only used by our proc macros
#[doc(hidden)]
pub mod wrapper;

#[doc(inline)]
pub use traits::*;
#[doc(inline)]
pub use types::{MYSQL_ERRMSG_SIZE, *};

/// Print a formatted log message to `stderr` to display in server logs
///
/// Performs formatting to match other common SQL error logs, roughly:
///
/// ```text
/// 2022-10-15 13:12:54+00:00 [Warning] Udf: this is the message
/// ```
///
/// ```
/// use udf::udf_log;
///
/// // Prints "2022-10-08 05:27:30+00:00 [Error] UDF: this is an error"
/// // This matches the default entrypoint log format
/// udf_log!(Error: "this is an error");
///
/// udf_log!(Warning: "this is a warning");
///
/// udf_log!(Note: "this is info");
///
/// udf_log!(Debug: "this is a debug message");
///
/// udf_log!("i print without the '[Level] UDF:' formatting");
/// ```
#[macro_export]
macro_rules! udf_log {
    (Error: $msg:expr) => {
        let formatted = format!("[Error] UDF: {}", $msg);
        udf_log!(formatted);
    };
    (Warning: $msg:expr) => {
        let formatted = format!("[Warning] UDF: {}", $msg);
        udf_log!(formatted);
    };
    (Note: $msg:expr) => {
        let formatted = format!("[Note] UDF: {}", $msg);
        udf_log!(formatted);
    };
    (Debug: $msg:expr) => {
        let formatted = format!("[Debug] UDF: {}", $msg);
        udf_log!(formatted);
    };
    ($msg:tt) => {
        eprintln!(
            "{} {}",
            udf::chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%:z"),
            $msg
        );
    };
}
