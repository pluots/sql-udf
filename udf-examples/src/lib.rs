//! Crate for example UDFs
//!
//! See this crate's README for details

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
    clippy::cast_possible_truncation,
    // New users probably like `match` better
    clippy::option_if_let_else,
    clippy::wildcard_imports
)]

mod attribute;
mod avg2;
mod avgcost;
mod empty;
mod is_const;
mod lipsum;
mod log_calls;
mod lookup;
mod median;
mod mishmash;
mod sequence;
mod sum_int;
