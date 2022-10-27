//! Crate for example UDFs
//!
//! To register all functions, execute the following:
//!
//! ```sql
//! CREATE AGGREGATE FUNCTION avg_cost RETURNS real SONAME 'libudf_examples.so';
//! CREATE FUNCTION is_const RETURNS string SONAME 'libudf_examples.so';
//! CREATE FUNCTION lookup6 RETURNS string SONAME 'libudf_examples.so';
//! CREATE AGGREGATE FUNCTION udf_median RETURNS integer SONAME 'libudf_examples.so';
//! CREATE FUNCTION sum_int RETURNS integer SONAME 'libudf_examples.so';
//! ```

#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::str_to_string,
    clippy::missing_inline_in_public_items,
    clippy::expect_used
)]
// Pedantic config
#![allow(
    clippy::missing_const_for_fn,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::cast_possible_truncation
)]

mod avg2;
mod avgcost;
mod is_const;
mod lipsum;
mod lookup;
mod median;
mod sequence;
mod sum_int;
