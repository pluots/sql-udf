#![allow(unused_imports)]

use heck::AsSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, parse_quote, DeriveInput, Error, Ident, ImplItem, ImplItemType, Item,
    ItemImpl, Path, PathSegment, Token, Type, TypePath, TypeReference,
};

use crate::match_variant;
use crate::types::{make_type_list, ImplType, RetType, TypeClass};

/// Create an identifier from another identifier, changing the name to snake case
macro_rules! format_ident_str {
    ($formatter:tt, $ident:ident) => {
        Ident::new(
            format!($formatter, AsSnakeCase($ident.to_string())).as_str(),
            Span::call_site(),
        )
    };
}

/// Verify that an `ItemImpl` matches the end of any given path
///
/// implements `BasicUdf` (in any of its pathing options)
fn impls_path(itemimpl: &ItemImpl, expected: ImplType) -> bool {
    let implemented = &itemimpl.trait_.as_ref().unwrap().1.segments;

    let basic_paths: [Punctuated<PathSegment, Token![::]>; 3] = [
        parse_quote! {udf::traits::BasicUdf},
        parse_quote! {udf::BasicUdf},
        parse_quote! {BasicUdf},
    ];
    let arg_paths: [Punctuated<PathSegment, Token![::]>; 3] = [
        parse_quote! {udf::traits::AggregateUdf},
        parse_quote! {udf::AggregateUdf},
        parse_quote! {AggregateUdf},
    ];

    match expected {
        ImplType::Basic => basic_paths.contains(implemented),
        ImplType::Aggregate => arg_paths.contains(implemented),
    }
}

/// Top-level entrypoint
///
/// Creates function signatures for use within the `#[register]` macro
///
/// # Arguments
///
/// - args: a stream of everything inside `(...)` (e.g.
/// `#[register(bin=false, a=2)]` will give the stream for `bin=false, a=2`
/// - item: the item contained within the stream
pub fn register(_args: &TokenStream, input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as ItemImpl);

    let impls_basic = impls_path(&parsed, ImplType::Basic);
    let impls_agg = impls_path(&parsed, ImplType::Aggregate);

    if !(impls_basic || impls_agg) {
        return Error::new_spanned(&parsed, "Expected trait `BasicUdf` or `AggregateUdf`")
            .into_compile_error()
            .into();
    }

    // Extract the last part of the implemented path
    // e.g. crate::mod::MyStruct will return MyStruct
    let impl_for_name = match *parsed.self_ty {
        Type::Path(ref v) => v.path.segments.last().unwrap().clone().ident,
        v => {
            return Error::new_spanned(v, "expected a path")
                .into_compile_error()
                .into()
        }
    };

    let wrapper_ident = Ident::new(&format!("_{impl_for_name}Wrapper"), Span::call_site());

    let content = if impls_basic {
        make_basic_fns(&parsed, &impl_for_name, &wrapper_ident)
    } else {
        make_agg_fns(&parsed, &impl_for_name, &wrapper_ident)
    };

    quote! {
        #parsed

        #content
    }
    .into()
}

/// Create the basic function signatures (`xxx_init`, `xxx_deinit`, `xxx`)
fn make_basic_fns(
    parsed: &ItemImpl,
    impl_for_name: &Ident,
    wrapper_ident: &Ident,
) -> proc_macro2::TokenStream {
    // Get the return type from the macro
    // There is only one type for this trait, which is "Returns"
    let impl_item_type = &parsed
        .items
        .iter()
        .find_map(match_variant!(ImplItem::Type))
        .expect("type expected")
        .ty;

    // Find the matching type in a list
    let content = make_type_list()
        .iter()
        .find(|x| x.type_ == *impl_item_type)
        .map_or_else(
            || {
                Error::new_spanned(
                    impl_item_type,
                    format!(
                        "expected `Returns` to be one of `i64`, `f64`, `&str`, `String`, or their `Option<...>` types, but got {impl_item_type:?}",
                    ),
                )
                .into_compile_error()
            },
            |t| make_basic_fns_content(t, impl_for_name,wrapper_ident),
        );

    content
}

/// Create the aggregate function signatures (`xxx_add`, `xxx_clear`, `xxx_remove`)
fn make_agg_fns(
    parsed: &ItemImpl,
    dstruct_ident: &Ident,
    wrapper_ident: &Ident,
) -> proc_macro2::TokenStream {
    let clear_fn_name = format_ident_str!("{}_clear", dstruct_ident);
    let add_fn_name = format_ident_str!("{}_add", dstruct_ident);
    let remove_fn_name = format_ident_str!("{}_remove", dstruct_ident);

    // Determine whether this re-implements `remove`
    let impls_remove = &parsed
        .items
        .iter()
        .filter_map(match_variant!(ImplItem::Fn))
        .map(|m| &m.sig.ident)
        .any(|id| *id == "remove");

    let clear_fn = make_clear_fn(dstruct_ident, wrapper_ident, &clear_fn_name);
    let add_fn = make_add_fn(dstruct_ident, wrapper_ident, &add_fn_name);
    let remove_fn_impl = make_remove_fn(dstruct_ident, wrapper_ident, &remove_fn_name);

    // If we implement remove, add a remove function. Otherwise, we don't need
    // anything.
    let remove_fn = if *impls_remove {
        remove_fn_impl
    } else {
        proc_macro2::TokenStream::default()
    };

    quote! {
        #clear_fn

        #add_fn

        #remove_fn
    }
}

fn make_basic_fns_content(
    rt: &RetType,
    dstruct_ident: &Ident,
    wrapper_ident: &Ident,
) -> proc_macro2::TokenStream {
    let init_fn_name = format_ident_str!("{}_init", dstruct_ident);
    let deinit_fn_name = format_ident_str!("{}_deinit", dstruct_ident);
    let process_fn_name = format_ident_str!("{}", dstruct_ident);

    let init_fn = make_init_fn(dstruct_ident, wrapper_ident, &init_fn_name);
    let deinit_fn = make_deinit_fn(dstruct_ident, wrapper_ident, &deinit_fn_name);
    let process_fn = match rt.type_cls {
        TypeClass::Bytes => make_proc_buf_fn(
            dstruct_ident,
            wrapper_ident,
            &process_fn_name,
            rt.is_optional,
            false,
        ),
        TypeClass::BytesRef => make_proc_buf_fn(
            dstruct_ident,
            wrapper_ident,
            &process_fn_name,
            rt.is_optional,
            true,
        ),
        TypeClass::Int => make_proc_fn(
            dstruct_ident,
            wrapper_ident,
            &process_fn_name,
            &quote!(::std::ffi::c_longlong),
            rt.is_optional,
        ),
        TypeClass::Float => make_proc_fn(
            dstruct_ident,
            wrapper_ident,
            &process_fn_name,
            &quote!(::std::ffi::c_double),
            rt.is_optional,
        ),
    };

    let ret_type = &rt.type_;
    let wrapper_struct = if rt.type_cls == TypeClass::Bytes {
        quote! {
            type #wrapper_ident = udf::wrapper::BufConverter<#dstruct_ident, #ret_type>;
        }
    } else {
        quote! { type #wrapper_ident = #dstruct_ident; }
    };
    // let process_fn = make_str_proc_fn(&dstruct_ident, deinit_fn_name, rt.is_optional);

    quote! {
        #wrapper_struct

        #init_fn

        #deinit_fn

        #process_fn
    }
}

/// Given the name of a type or struct, create a function that will be evaluated (`xxx`)
fn make_init_fn(
    dstruct_ident: &Ident,
    wrapper_ident: &Ident,
    fn_name: &Ident,
) -> proc_macro2::TokenStream {
    // SAFETY: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::udf_sys::UDF_INIT,
            args: *mut udf::udf_sys::UDF_ARGS,
            message: *mut std::ffi::c_char,
        ) -> bool
        {
            udf::wrapper::wrap_init::<#wrapper_ident, #dstruct_ident>(initid, args, message)
        }
    }
}

/// Make the `xxx_deinit` function signature
fn make_deinit_fn(
    dstruct_ident: &Ident,
    wrapper_ident: &Ident,
    fn_name: &Ident,
) -> proc_macro2::TokenStream {
    // SAFETY: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::udf_sys::UDF_INIT,
        ) {
            udf::wrapper::wrap_deinit::<#wrapper_ident, #dstruct_ident>(initid)
        }
    }
}

fn make_proc_fn(
    dstruct_ident: &Ident,
    wrapper_ident: &Ident,
    fn_name: &Ident,
    ret_type: &proc_macro2::TokenStream,
    is_optional: bool,
) -> proc_macro2::TokenStream {
    let wrap_fn_name = if is_optional {
        quote!(udf::wrapper::wrap_process_basic_option::<#wrapper_ident, #dstruct_ident, _>)
    } else {
        quote!(udf::wrapper::wrap_process_basic::<#wrapper_ident, #dstruct_ident, _>)
    };

    // SAFETY: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::udf_sys::UDF_INIT,
            args: *mut udf::udf_sys::UDF_ARGS,
            is_null: *mut ::std::ffi::c_uchar,
            error: *mut ::std::ffi::c_uchar,
        ) -> #ret_type {
            #wrap_fn_name(initid, args, is_null, error)
        }
    }
}

fn make_proc_buf_fn(
    dstruct_ident: &Ident,
    wrapper_ident: &Ident,
    fn_name: &Ident,
    is_optional: bool,
    is_ref: bool,
) -> proc_macro2::TokenStream {
    let wrap_fn_name = if is_optional && is_ref {
        quote!(udf::wrapper::wrap_process_buf_option_ref::<#wrapper_ident, #dstruct_ident, _>)
    } else if is_optional {
        quote!(udf::wrapper::wrap_process_buf_option::<#wrapper_ident, #dstruct_ident, _>)
    } else {
        quote!(udf::wrapper::wrap_process_buf::<#wrapper_ident, #dstruct_ident>)
    };

    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::udf_sys::UDF_INIT,
            args: *mut udf::udf_sys::UDF_ARGS,
            result: *mut ::std::ffi::c_char,
            length: *mut ::std::ffi::c_ulong,
            is_null: *mut ::std::ffi::c_uchar,
            error: *mut ::std::ffi::c_uchar,
        ) -> *const ::std::ffi::c_char {
            #wrap_fn_name(
                initid,
                args,
                result,
                length,
                is_null,
                error,
            )
        }
    }
}

/// Create the function signature for aggregate `xxx_add`
fn make_add_fn(
    dstruct_ident: &Ident,
    wrapper_ident: &Ident,
    fn_name: &Ident,
) -> proc_macro2::TokenStream {
    // SAFETY: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::udf_sys::UDF_INIT,
            args: *mut udf::udf_sys::UDF_ARGS,
            is_null: *mut ::std::ffi::c_uchar,
            error: *mut ::std::ffi::c_uchar,
        ) {
            udf::wrapper::wrap_add::<#wrapper_ident, #dstruct_ident>(initid, args, is_null, error)
        }
    }
}

/// Create the function signature for aggregate `xxx_clear`
fn make_clear_fn(
    dstruct_ident: &Ident,
    wrapper_ident: &Ident,
    fn_name: &Ident,
) -> proc_macro2::TokenStream {
    // SAFETY: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::udf_sys::UDF_INIT,
            is_null: *mut ::std::ffi::c_uchar,
            error: *mut ::std::ffi::c_uchar,
        ) {
            udf::wrapper::wrap_clear::<#wrapper_ident, #dstruct_ident>(initid, is_null, error)
        }
    }
}

/// Create the function signature for aggregate `xxx_remove`
fn make_remove_fn(
    dstruct_ident: &Ident,
    wrapper_ident: &Ident,
    fn_name: &Ident,
) -> proc_macro2::TokenStream {
    // SAFETY: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::udf_sys::UDF_INIT,
            args: *mut udf::udf_sys::UDF_ARGS,
            is_null: *mut ::std::ffi::c_uchar,
            error: *mut ::std::ffi::c_uchar,
        ) {
            udf::wrapper::wrap_remove::<#wrapper_ident, #dstruct_ident>
                (initid, args, is_null, error)
        }
    }
}
