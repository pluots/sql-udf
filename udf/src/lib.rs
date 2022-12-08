//! A wrapper crate to make writing SQL user-defined functions (UDFs) easy
//!
//! This crate provides bindings for easy creation of SQL user-defined functions
//! in Rust. See [the
//! readme](https://github.com/pluots/sql-udf/blob/main/README.md) for more
//! background information on how UDFs work in general.
//!
//! # Usage
//!
//! Using this crate is fairly simple: create a struct that will be used to
//! share data among UDF function calls (which can be zero-sized), then
//! implement needed traits for it. [`BasicUdf`] provides function signatures
//! for standard UDFs, and [`AggregateUdf`] provides signatures for aggregate
//! (and window) UDFs. See the documentation there for a step-by-step guide.
//!
//! ```
//! use udf::prelude::*;
//!
//! // Our struct that will produce a UDF of name `my_udf`
//! // If there is no data to store between calls, it can be zero sized
//! struct MyUdf;
//!
//! #[register]
//! impl BasicUdf for MyUdf {
//!     // Specify return type of this UDF to be a nullable integer
//!     type Returns<'a> = Option<i64>;
//!
//!     // Perform initialization steps here
//!     fn init(cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
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
//! # Building & Usage
//!
//! The above example will create three C-callable functions: `my_udf`,
//! `my_udf_init`, and `my_udf_deinit`, which is what `MariaDB` and `MySql`
//! expect for UDFs. To create a C dynamic library (as is required for usage),
//! add the following to your `Cargo.toml`
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! The next time you run `cargo build --release`, in `target/release` there
//! will be a shared library `.so` file. Copy this to your `plugin_dir` location
//! (usually `/usr/lib/mysql/plugin/`), and load the function with the
//! following:
//!
//! ```sql
//! CREATE FUNCTION my_udf RETURNS integer SONAME 'libudf_test.so';
//! ```
//!
//! Replace `my_udf` with the function name, `integer` with the return type, and
//! `libudf_test.so` with the correct file name.
//!
//! More details on building are discussed in [the project
//! readme](https://github.com/pluots/sql-udf/blob/main/README.md). See [the
//! `MariaDB` documentation](https://mariadb.com/kb/en/create-function-udf/) for
//! more detailed information on how to load the created libraries.
//!
//! # Version Note
//!
//! Because of reliance on a feature called GATs, this library requires Rust
//! version >= 1.65. This only became stable on 2022-11-03; if you encounter
//! issues compiling, be sure to update your toolchain.

// Strict clippy
#![warn(
    clippy::pedantic,
    // clippy::cargo,
    clippy::nursery,
    clippy::str_to_string,
    clippy::exhaustive_enums,
    clippy::pattern_type_mismatch
)]
// Pedantic config
#![allow(
    clippy::missing_const_for_fn,
    // clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::cast_possible_truncation
)]

// We re-export this so we can use it in our macro, but don't need it
// to show up in our docs
#[doc(hidden)]
pub extern crate chrono;

#[doc(hidden)]
pub extern crate udf_sys;

extern crate udf_macros;

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

// #[cfg(mock)]
pub mod mock;

/// Print a formatted log message to `stderr` to display in server logs
///
/// Performs formatting to match other common SQL error logs, roughly:
///
/// ```text
/// 2022-10-15 13:12:54+00:00 [Warning] Udf: this is the message
/// ```
///
/// ```
/// # #[cfg(not(miri))] // need to skip Miri because. it can't cross FFI
/// # fn test() {
///
/// use udf::udf_log;
///
/// // Prints "2022-10-08 05:27:30+00:00 [Error] UDF: this is an error"
/// // This matches the default entrypoint log format
/// udf_log!(Error: "this is an error");
///
/// udf_log!(Warning: "this is a warning");
///
/// udf_log!(Note: "this is info: value {}", 10 + 10);
///
/// udf_log!(Debug: "this is a debug message");
///
/// udf_log!("i print without the '[Level] UDF:' formatting");
///
/// # }
/// # #[cfg(not(miri))]
/// # test();
/// ```
#[macro_export]
macro_rules! udf_log {
    (Error: $($msg:tt)*) => {{
        let formatted = format!("[Error] UDF: {}", format!($($msg)*));
        udf_log!(formatted);
    }};
    (Warning: $($msg:tt)*) => {{
        let formatted = format!("[Warning] UDF: {}", format!($($msg)*));
        udf_log!(formatted);
    }};
    (Note: $($msg:tt)*) => {{
        let formatted = format!("[Note] UDF: {}", format!($($msg)*));
        udf_log!(formatted);
    }};
    (Debug: $($msg:tt)*) => {{
        let formatted = format!("[Debug] UDF: {}", format!($($msg)*));
        udf_log!(formatted);
    }};
    ($msg:tt) => {
        eprintln!(
            "{} {}",
            $crate::chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%:z"),
            $msg
        );
    };
}
