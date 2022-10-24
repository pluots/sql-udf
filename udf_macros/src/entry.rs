// #![allow(unused_imports)]
// #![allow(unused_variables)]
#![allow(unused)]
// use lazy_static;
use heck::AsSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Colon2;
use syn::{
    parse_macro_input, parse_quote, DeriveInput, Error, Ident, ImplItem, ImplItemType, Item,
    ItemImpl, Path, PathSegment, Token, Type, TypePath, TypeReference,
};

use crate::match_variant;
use crate::types::{make_type_list, FnSigType, ImplType, RetType};

macro_rules! format_ident_str {
    ($formatter: tt, $ident: ident) => {
        Ident::new(
            format!($formatter, AsSnakeCase($ident.to_string())).as_str(),
            Span::call_site(),
        )
    };
}

/// Verify that an ItemImpl matches the end of any given path
///
/// implements BasicUdf (in any of its pathing options)
fn impls_path(itemimpl: &ItemImpl, expected: ImplType) -> bool {
    let implemented = &itemimpl.trait_.as_ref().unwrap().1.segments;

    let basic_paths: [Punctuated<PathSegment, Colon2>; 3] = [
        parse_quote! {udf::traits::BasicUdf},
        parse_quote! {udf::BasicUdf},
        parse_quote! {BasicUdf},
    ];
    let arg_paths: [Punctuated<PathSegment, Colon2>; 3] = [
        parse_quote! {udf::traits::AggregateUdf},
        parse_quote! {udf::AggregateUdf},
        parse_quote! {AggregateUdf},
    ];

    match expected {
        ImplType::Basic => basic_paths.contains(&implemented),
        ImplType::Aggregate => arg_paths.contains(&implemented),
    }
}

/// # Arguments
///
/// - args: a stream of everything inside `(...)` (e.g.
/// `#[register(bin=false, a=2)]` will give the stream for `bin=false, a=2`
/// - item: the item contained within the stream
pub(crate) fn register(_args: TokenStream, input: TokenStream) -> TokenStream {
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

    // Get the return type from the macro
    // There is only one type for this trait, which is "Returns"
    let impl_item_type = &parsed
        .items
        .iter()
        .find_map(match_variant!(ImplItem::Type))
        .expect("type expected")
        .ty;

    // Find the matching type in a list
    let content = match make_type_list().iter().find(|x| x.type_ == *impl_item_type) {
        Some(t) => make_basic_fns(t, impl_for_name),
        None => {
            let emsg = format!(
                "expected `Result` to be one of `i64`, `f64`, `&str`, `String`, \
                or their `Option<...>` types, but got {impl_item_type:?}",
            );
            Error::new_spanned(impl_item_type, emsg)
                .into_compile_error()
                .into()
        }
    };

    quote! {
        #parsed

        #content
    }
    .into()
}

fn make_basic_fns(rt: &RetType, dstruct_ident: Ident) -> proc_macro2::TokenStream {
    let init_fn_name = format_ident_str!("{}_init", dstruct_ident);
    let deinit_fn_name = format_ident_str!("{}_deinit", dstruct_ident);
    let process_fn_name = format_ident_str!("{}", dstruct_ident);

    let init_fn = make_init_fn(&dstruct_ident, init_fn_name);
    let deinit_fn = make_deinit_fn(&dstruct_ident, deinit_fn_name);
    let process_fn = match rt.fn_sig {
        FnSigType::String => todo!(),
        FnSigType::Int => make_proc_int_fn(&dstruct_ident, process_fn_name, rt.is_optional),
        FnSigType::Float => make_proc_float_fn(&dstruct_ident, process_fn_name, rt.is_optional),
    };
    // let process_fn = make_str_proc_fn(&dstruct_ident, deinit_fn_name, rt.is_optional);

    quote! {
        #init_fn

        #deinit_fn

        #process_fn
    }
}

/// Given the name of a type or struct, create a function that will be evaluated
fn make_init_fn(dstruct_ident: &Ident, fn_name: Ident) -> proc_macro2::TokenStream {
    // Safety: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::ffi::bindings::UDF_INIT,
            args: *mut udf::ffi::bindings::UDF_ARGS,
            message: *mut std::ffi::c_char,
        ) -> bool
        {
            unsafe {
                udf::ffi::wrapper::wrap_init::<#dstruct_ident>(initid, args, message)
            }
        }
    }
}

fn make_deinit_fn(dstruct_ident: &Ident, fn_name: Ident) -> proc_macro2::TokenStream {
    // Safety: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::ffi::bindings::UDF_INIT,
        ) {
            unsafe { udf::ffi::wrapper::wrap_deinit::<#dstruct_ident>(initid) }
        }
    }
}

fn make_proc_int_fn(
    dstruct_ident: &Ident,
    fn_name: Ident,
    nullable: bool,
) -> proc_macro2::TokenStream {
    // Safety: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    let fn_title = if nullable {
        quote! { udf::ffi::wrapper::wrap_process_int_null::<#dstruct_ident> }
    } else {
        quote! { udf::ffi::wrapper::wrap_process_int::<#dstruct_ident> }
    };

    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::ffi::bindings::UDF_INIT,
            args: *mut udf::ffi::bindings::UDF_ARGS,
            is_null: *mut ::std::ffi::c_uchar,
            error: *mut ::std::ffi::c_uchar,
        ) -> ::std::ffi::c_longlong {
            unsafe {
                #fn_title(
                    initid,
                    args,
                    is_null,
                    error
                )
            }
        }
    }
}

fn make_proc_float_fn(
    dstruct_ident: &Ident,
    fn_name: Ident,
    nullable: bool,
) -> proc_macro2::TokenStream {
    // Safety: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    let fn_title = if nullable {
        quote! { udf::ffi::wrapper::wrap_process_float_null::<#dstruct_ident> }
    } else {
        quote! { udf::ffi::wrapper::wrap_process_float::<#dstruct_ident> }
    };

    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::ffi::bindings::UDF_INIT,
            args: *mut udf::ffi::bindings::UDF_ARGS,
            is_null: *mut ::std::ffi::c_uchar,
            error: *mut ::std::ffi::c_uchar,
        ) -> f64 {
            unsafe {
                #fn_title(
                    initid,
                    args,
                    is_null,
                    error
                )
            }
        }
    }
}
