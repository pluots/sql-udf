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
//! // Specifying a name is optional; `#[register]` uses a snake case version of
//! // the struct name by default (`my_udf` in this case)
//! #[register(name = "my_shiny_udf")]
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
//! # Crate Features
//!
//! This crate includes some optional features. They can be enabled in your
//! `Cargo.toml`.
//!
//! - `mock`: enable this feature to add the [mock] module for easier unit
//!   testing. _(note: this feature will become unneeded in the future and
//!   `mock` will be available by default. It is currently feature-gated because
//!   it is considered unstable.)_
//! - `logging-debug`: enable this feature to turn on debug level logging for
//!   this crate. This uses the `udf_log!` macro and includes information about
//!   memory management and function calls. These will show up with your SQL
//!   server logs, like:
//!
//!   ```text
//!   2023-03-23 00:45:53+00:00 [Debug] UDF: ENTER init for 'udf_examples::lookup::Lookup6'
//!   2023-03-23 00:45:53+00:00 [Debug] UDF: 0x7fdea4022220 24 bytes udf->server control transfer
//!                                          (BufConverter<Lookup6, Option<String>>)
//!   2023-03-23 00:45:53+00:00 [Debug] UDF: EXIT init for 'udf_examples::lookup::Lookup6'
//!   2023-03-23 00:45:53+00:00 [Debug] UDF: ENTER process for 'udf_examples::lookup::Lookup6'
//!   2023-03-23 00:45:53+00:00 [Debug] UDF: 0x7fdea4022220 24 bytes server->udf control transfer
//!                                          (BufConverter<Lookup6, Option<String>>)
//!   2023-03-23 00:45:53+00:00 [Debug] UDF: 0x7fdea4022220 24 bytes udf->server control transfer
//!                                          (BufConverter<Lookup6, Option<String>>)
//!   2023-03-23 00:45:53+00:00 [Debug] UDF: EXIT process for 'udf_examples::lookup::Lookup6'
//!   2023-03-23 00:45:53+00:00 [Debug] UDF: ENTER deinit for 'udf_examples::lookup::Lookup6'
//!   2023-03-23 00:45:53+00:00 [Debug] UDF: 0x7fdea4022220 24 bytes server->udf control transfer
//!                                          (BufConverter<Lookup6, Option<String>>)
//!   ```
//!
//!   This output can be helpful to understand the exact data flow between
//!   the library and the server. They are enabled by default in the `udf-examples`
//!   library.
//!
//! - `logging-debug-calls` full debugging printing of the structs passed
//!   between this library and the SQL server. Implies `logging-debug`. This
//!   output can be noisy, but can help to debug issues related to the lower
//!   level interfaces (i.e. problems with this library or with the server
//!   itself).
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

#[macro_use]
mod macros;
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

pub mod mock;
