#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::str_to_string,
    clippy::missing_inline_in_public_items
)]
// Pedantic config
#![allow(
    clippy::missing_const_for_fn,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::cast_possible_truncation
)]

mod entry;
mod types;

use proc_macro::TokenStream;

macro_rules! match_variant {
    ($variant:path) => {
        |x| {
            if let $variant(value) = x {
                Some(value)
            } else {
                None
            }
        }
    };
}

pub(crate) use match_variant;

/// # Examples
///
/// ```
/// #[udf::register]
/// struct X{}
///
/// ```
///
/// # Arguments
///
/// This macro accepts the following optional arguments:
///
/// - `#[udf::register(decimals = N)]` set the number of decimals (behind the
///   scenes this sets `initd.decimals`)
/// - `#[udf::register(max_length = N)]` set the max length for strings or
///   decimals (this sets `initid.max_length`)
/// - `#[udf::register(const = true)]` set true if it always returns the same
///   value (this sets `initd.const_item`)
/// - `#[udf::register(name = "new_name")]` will specify a name for your SQL
///   function. If this is not specified, your struct name will be converted to
///   snake case and used (e.g. `AddAllNumbers` would become `add_all_numbers`
///   by default).
///
/// # Behind the scenes
///
/// `initd.maybe_null` is set based on the `Return` type (whether optional or
/// not)
#[proc_macro_attribute]
// #[cfg(not(test))] // Work around for rust-lang/rust#62127
pub fn register(args: TokenStream, item: TokenStream) -> TokenStream {
    // Keep this file clean by keeping the dirty work in entry
    entry::register(&args, item)
}
