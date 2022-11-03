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
    clippy::cast_possible_truncation
)]

mod avg2;
mod avgcost;
mod is_const;
mod lipsum;
mod log_calls;
mod lookup;
mod median;
mod sequence;
mod sum_int;
