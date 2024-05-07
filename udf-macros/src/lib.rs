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

mod register;
mod simplified;
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

/// # Register exposed function names required for a UDF
///
/// This macro is applied to an `impl BasicUdf` block (and an `AggregateUdf`
/// block, if applicable) and exposed the C-callable functions that
/// `MariaDB`/`MySQL` expect.
///
/// Usage:
///
/// ```ignore
/// #[register]
/// impl BasicUdf for MyStruct {
///     ...
/// }
///
/// #[register]
/// impl AggregateUdf for MyStruct {
///     ...
/// }
/// ```
///
/// Its process is as follows:
///
/// - Convert the implemented struct's name to snake case to create the function
///   name (unless a name is specified)
/// - Obtain the return type from the `Returns` type in `BasicUdf`
/// - Create functions `fn_name`, `fn_name_init`, and `fn_name_deinit` with
///   correct signatures and interfaces
/// - If applied on an `impl AggregateUdf` block, create `fn_name_clear` and
///   `fn_name_add`. `fn_name_remove` is also included if it is redefined
///
/// # Arguments
///
/// - `#[udf::register(name = "new_name")]` will specify a name for your SQL
///   function. If this is not specified, your struct name will be converted to
///   snake case and used (e.g. `AddAllNumbers` would become `add_all_numbers`
///   by default).
/// - `#[udf::register(alias = "alias")]` will specify an alias for this function.
///   More than one alias can be specified, and it can be combined with a `name` attribute.
///
/// **IMPORTANT**: if using aggregate UDFs, the exact same renaming must be applied to
/// both the `impl BasicUdf` and the `impl AggregateUdf` blocks! If this is not followed,
/// your function will not act as an aggregate (there may also be a compile error).
#[proc_macro_attribute]
pub fn register(args: TokenStream, item: TokenStream) -> TokenStream {
    // Keep this file clean by keeping the dirty work in entry
    register::register(&args, item)
}

/// Apply `#[udf::udf]` to any function to make it a UDF
#[proc_macro_attribute]
pub fn udf(args: TokenStream, item: TokenStream) -> TokenStream {
    // Keep this file clean by keeping the dirty work in entry
    simplified::simple_udf(&args, item)
}
