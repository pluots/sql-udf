
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, Item, DeriveInput};


/// # Arguments
/// 
/// - args: a stream of everything inside `(...)` (e.g.
/// `#[register(bin=false, a=2)]` will give the stream for `bin=false, a=2`
/// 
/// - item: the item contained within the stream
pub(crate) fn register(args: TokenStream, item: TokenStream) -> TokenStream {
    let tmp = item.clone();
    let input = parse_macro_input!(item as Item);

    // We aren't doing a derive macro but DeriveInput helps us easily get the identifier
    let ident = parse_macro_input!(tmp as DeriveInput).ident;

    let init_fn_name = format_ident!("{ident}_init");

    // Safety: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    let init_fn = quote! {
        unsafe extern "C" fn #init_fn_name (
            initid: *mut udf::ffi::bindings::UDF_INIT,
            args: *const udf::ffi::bindings::UDF_ARGS,
            message: *mut core::ffi::c_char,
        ) -> bool
        {
            unsafe { udf::ffi::wrapper::init_wrapper::<#ident::Returns>(initid, args, message) }
        }
    };

    let expanded = quote! {
        #input
        #init_fn
    };

    TokenStream::from(expanded)
}
