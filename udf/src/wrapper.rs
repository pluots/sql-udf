//! Non-public module to assist macro with wrapping functions
//!
//! Warning: This module should be considered unstable and generally not for
//! public use

#[macro_use]
mod const_helpers;
mod functions;
mod helpers;
mod modded_types;
mod process;

use std::str;

use const_helpers::{const_slice_eq, const_slice_to_str, const_str_eq};
pub use functions::{wrap_add, wrap_clear, wrap_deinit, wrap_init, wrap_remove, BufConverter};
pub(crate) use helpers::*;
pub use modded_types::UDF_ARGSx;
pub use process::{
    wrap_process_basic, wrap_process_basic_option, wrap_process_buf, wrap_process_buf_option,
    wrap_process_buf_option_ref,
};

/// A trait implemented by the proc macro
// FIXME: on unimplemented
pub trait RegisteredBasicUdf {
    /// The main function name
    const NAME: &'static str;
    /// Aliases, if any
    const ALIASES: &'static [&'static str];
    /// True if `NAME` comes from the default value for the struct
    const DEFAULT_NAME_USED: bool;
}

/// Implemented by the proc macro. This is used to enforce that the basic UDF and aggregate
/// UDF have the same name and aliases.
pub trait RegisteredAggregateUdf: RegisteredBasicUdf {
    /// The main function name
    const NAME: &'static str;
    /// Aliases, if any
    const ALIASES: &'static [&'static str];
    /// True if `NAME` comes from the default value for the struct
    const DEFAULT_NAME_USED: bool;
}

const NAME_MSG: &str = "`#[register]` on `BasicUdf` and `AggregateUdf` must have the same ";

/// Enforce that a struct has the same basic and aggregate UDF names.
pub const fn verify_aggregate_attributes<T: RegisteredAggregateUdf>() {
    verify_aggregate_attributes_name::<T>();
    verify_aggregate_attribute_aliases::<T>();
}

const fn verify_aggregate_attributes_name<T: RegisteredAggregateUdf>() {
    let basic_name = <T as RegisteredBasicUdf>::NAME;
    let agg_name = <T as RegisteredAggregateUdf>::NAME;
    let basic_default_name = <T as RegisteredBasicUdf>::DEFAULT_NAME_USED;
    let agg_default_name = <T as RegisteredAggregateUdf>::DEFAULT_NAME_USED;

    if const_str_eq(basic_name, agg_name) {
        return;
    }

    let mut msg_buf = [0u8; 512];
    let mut curs = 0;
    curs += const_write_all!(
        msg_buf,
        [NAME_MSG, "`name` argument; got `", basic_name, "`",],
        curs
    );

    if basic_default_name {
        curs += const_write_all!(msg_buf, [" (default from struct name)"], curs);
    }

    curs += const_write_all!(msg_buf, [" and `", agg_name, "`"], curs);

    if agg_default_name {
        curs += const_write_all!(msg_buf, [" (default from struct name)"], curs);
    }

    let msg = const_slice_to_str(msg_buf.as_slice(), curs);
    panic!("{}", msg);
}

#[allow(clippy::cognitive_complexity)]
const fn verify_aggregate_attribute_aliases<T: RegisteredAggregateUdf>() {
    let basic_aliases = <T as RegisteredBasicUdf>::ALIASES;
    let agg_aliases = <T as RegisteredAggregateUdf>::ALIASES;

    if const_slice_eq(basic_aliases, agg_aliases) {
        return;
    }

    let mut msg_buf = [0u8; 512];
    let mut curs = 0;

    curs += const_write_all!(msg_buf, [NAME_MSG, "`alias` arguments; got [",], 0);

    let mut i = 0;
    while i < basic_aliases.len() {
        if i > 0 {
            curs += const_write_all!(msg_buf, [", "], curs);
        }
        curs += const_write_all!(msg_buf, ["`", basic_aliases[i], "`",], curs);
        i += 1;
    }

    curs += const_write_all!(msg_buf, ["] and ["], curs);

    let mut i = 0;
    while i < agg_aliases.len() {
        if i > 0 {
            curs += const_write_all!(msg_buf, [", "], curs);
        }
        curs += const_write_all!(msg_buf, ["`", agg_aliases[i], "`",], curs);
        i += 1;
    }

    curs += const_write_all!(msg_buf, ["]"], curs);

    let msg = const_slice_to_str(msg_buf.as_slice(), curs);
    panic!("{}", msg);
}

#[cfg(test)]
mod tests;
