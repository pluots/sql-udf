
mod entry;

use proc_macro::TokenStream;


/// # Examples
/// 
/// ```
/// #[udf::register]
/// struct X{}
/// 
/// ```
#[proc_macro_attribute]
// #[cfg(not(test))] // Work around for rust-lang/rust#62127
pub fn register(args: TokenStream, item: TokenStream) -> TokenStream {
    // Keep this file clean by keeping the dirty work in entry
    entry::register(args, item)
}
