#![allow(unused_imports)]

use heck::AsSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, parse_quote, DeriveInput, Error, Expr, ExprLit, Ident, ImplItem,
    ImplItemType, Item, ItemImpl, Lit, Meta, Path, PathSegment, Token, Type, TypePath,
    TypeReference,
};

use crate::match_variant;
use crate::types::{make_type_list, ImplType, RetType, TypeClass};

/// Create an identifier from another identifier, changing the name to snake case
macro_rules! format_ident_str {
    ($formatter:tt, $ident:ident) => {
        Ident::new(format!($formatter, $ident).as_str(), Span::call_site())
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
pub fn register(args: &TokenStream, input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as ItemImpl);

    let impls_basic = impls_path(&parsed, ImplType::Basic);
    let impls_agg = impls_path(&parsed, ImplType::Aggregate);

    if !(impls_basic || impls_agg) {
        return Error::new_spanned(&parsed, "Expected trait `BasicUdf` or `AggregateUdf`")
            .into_compile_error()
            .into();
    }

    // Full type path of our data struct
    let Type::Path(dstruct_path) = parsed.self_ty.as_ref() else {
        return Error::new_spanned(parsed.self_ty, "expected a path")
            .into_compile_error()
            .into();
    };

    let base_fn_names = match parse_args(args, dstruct_path) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };

    let dstruct_path_as_ident: String = dstruct_path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect();
    let wrapper_ident = Ident::new(
        &format!("_{dstruct_path_as_ident}Wrapper",),
        Span::call_site(),
    );

    let (ret_ty, wrapper_def) = if impls_basic {
        match get_rt_and_wrapper(&parsed, dstruct_path, &wrapper_ident) {
            Ok((r, w)) => (Some(r), w),
            Err(e) => return e.into_compile_error().into(),
        }
    } else {
        (None, TokenStream2::new())
    };

    let content_iter = base_fn_names.iter().map(|base_fn_name| {
        if impls_basic {
            make_basic_fns(
                ret_ty.as_ref().unwrap(),
                base_fn_name,
                dstruct_path,
                &wrapper_ident,
            )
        } else {
            make_agg_fns(&parsed, base_fn_name, dstruct_path, &wrapper_ident)
        }
    });

    quote! {
        #parsed

        #wrapper_def

        #( #content_iter )*
    }
    .into()
}

/// Parse attribute arguments. Returns an iterator of names
fn parse_args(args: &TokenStream, dstruct_path: &TypePath) -> syn::Result<Vec<String>> {
    let meta = Punctuated::<Meta, Token![,]>::parse_terminated.parse(args.clone())?;
    let mut base_fn_names: Vec<String> = vec![];
    let mut primary_name_specified = false;

    for m in meta {
        let Meta::NameValue(mval) = m else {
            return Err(Error::new_spanned(m, "expected `a = b atributes`"));
        };

        if !mval.path.segments.iter().count() == 1 {
            return Err(Error::new_spanned(mval.path, "unexpected path"));
        }

        let key = mval.path.segments.first().unwrap();

        let Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) = mval.value
        else {
            return Err(Error::new_spanned(mval.value, "expected a literal string"));
        };

        if key.ident == "name" {
            if primary_name_specified {
                return Err(Error::new_spanned(key, "`name` can only be specified once"));
            }
            base_fn_names.push(value.value());
            primary_name_specified = true;
        } else if key.ident == "alias" {
            base_fn_names.push(value.value());
        } else {
            return Err(Error::new_spanned(
                key,
                "unexpected key (only `name` and `alias` are accepted)",
            ));
        }
    }

    if !primary_name_specified {
        // If we don't have a name specified, use the type name as snake case
        let ty_ident = &dstruct_path.path.segments.last().unwrap().ident;
        let fn_name = AsSnakeCase(&ty_ident.to_string()).to_string();
        base_fn_names.push(fn_name);
    }

    Ok(base_fn_names)
}

/// Get the return type to use and a wrapper. Once per impl setup.
fn get_rt_and_wrapper(
    parsed: &ItemImpl,
    dstruct_path: &TypePath,
    wrapper_ident: &Ident,
) -> syn::Result<(RetType, TokenStream2)> {
    // Get the return type from the macro
    // There is only one type for this trait, which is "Returns"
    let trait_ret_ty = &parsed
        .items
        .iter()
        .find_map(match_variant!(ImplItem::Type))
        .expect("type expected")
        .ty;
    let ret_ty = make_type_list()
        .into_iter()
        .find(|x| x.type_ == *trait_ret_ty)
        .ok_or_else(|| {
            Error::new_spanned(
                trait_ret_ty,
                "expected `Returns` to be one of `i64`, `f64`, `&str`, `String`, \
                 or their `Option<...>` types",
            )
        })?;

    let ret_ty_type = &ret_ty.type_;
    let wrapper_struct = if ret_ty.type_cls == TypeClass::Bytes {
        quote! {
            type #wrapper_ident = udf::wrapper::BufConverter<#dstruct_path, #ret_ty_type>;
        }
    } else {
        quote! { type #wrapper_ident = #dstruct_path; }
    };

    Ok((ret_ty, wrapper_struct))
}

/// Create the basic function signatures (`xxx_init`, `xxx_deinit`, `xxx`)
fn make_basic_fns(
    rt: &RetType,
    base_fn_name: &str,
    dstruct_path: &TypePath,
    wrapper_ident: &Ident,
) -> TokenStream2 {
    let init_fn_name = format_ident_str!("{}_init", base_fn_name);
    let deinit_fn_name = format_ident_str!("{}_deinit", base_fn_name);
    let process_fn_name = format_ident_str!("{}", base_fn_name);

    let init_fn = make_init_fn(dstruct_path, wrapper_ident, &init_fn_name);
    let deinit_fn = make_deinit_fn(dstruct_path, wrapper_ident, &deinit_fn_name);
    let process_fn = match rt.type_cls {
        TypeClass::Bytes => make_proc_buf_fn(
            dstruct_path,
            wrapper_ident,
            &process_fn_name,
            rt.is_optional,
            false,
        ),
        TypeClass::BytesRef => make_proc_buf_fn(
            dstruct_path,
            wrapper_ident,
            &process_fn_name,
            rt.is_optional,
            true,
        ),
        TypeClass::Int => make_proc_fn(
            dstruct_path,
            wrapper_ident,
            &process_fn_name,
            &quote!(::std::ffi::c_longlong),
            rt.is_optional,
        ),
        TypeClass::Float => make_proc_fn(
            dstruct_path,
            wrapper_ident,
            &process_fn_name,
            &quote!(::std::ffi::c_double),
            rt.is_optional,
        ),
    };

    quote! {
        #init_fn

        #deinit_fn

        #process_fn
    }
}

/// Create the aggregate function signatures (`xxx_add`, `xxx_clear`, `xxx_remove`)
fn make_agg_fns(
    parsed: &ItemImpl,
    base_fn_name: &str,      // Name of the function symbols
    dstruct_path: &TypePath, // Name of the data structure
    wrapper_ident: &Ident,
) -> TokenStream2 {
    let clear_fn_name = format_ident_str!("{}_clear", base_fn_name);
    let add_fn_name = format_ident_str!("{}_add", base_fn_name);
    let remove_fn_name = format_ident_str!("{}_remove", base_fn_name);

    // Determine whether this re-implements `remove`
    let impls_remove = &parsed
        .items
        .iter()
        .filter_map(match_variant!(ImplItem::Fn))
        .map(|m| &m.sig.ident)
        .any(|id| *id == "remove");

    let clear_fn = make_clear_fn(dstruct_path, wrapper_ident, &clear_fn_name);
    let add_fn = make_add_fn(dstruct_path, wrapper_ident, &add_fn_name);
    let remove_fn_impl = make_remove_fn(dstruct_path, wrapper_ident, &remove_fn_name);

    // If we implement remove, add a remove function. Otherwise, we don't need
    // anything.
    let remove_fn = if *impls_remove {
        remove_fn_impl
    } else {
        TokenStream2::default()
    };

    quote! {
        #clear_fn

        #add_fn

        #remove_fn
    }
}

/// Given the name of a type or struct, create a function that will be evaluated (`xxx`)
fn make_init_fn(dstruct_path: &TypePath, wrapper_ident: &Ident, fn_name: &Ident) -> TokenStream2 {
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
            udf::wrapper::wrap_init::<#wrapper_ident, #dstruct_path>(initid, args, message)
        }
    }
}

/// Make the `xxx_deinit` function signature
fn make_deinit_fn(dstruct_path: &TypePath, wrapper_ident: &Ident, fn_name: &Ident) -> TokenStream2 {
    // SAFETY: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::udf_sys::UDF_INIT,
        ) {
            udf::wrapper::wrap_deinit::<#wrapper_ident, #dstruct_path>(initid)
        }
    }
}

fn make_proc_fn(
    dstruct_path: &TypePath,
    wrapper_ident: &Ident,
    fn_name: &Ident,
    ret_type: &TokenStream2,
    is_optional: bool,
) -> TokenStream2 {
    let wrap_fn_name = if is_optional {
        quote!(udf::wrapper::wrap_process_basic_option::<#wrapper_ident, #dstruct_path, _>)
    } else {
        quote!(udf::wrapper::wrap_process_basic::<#wrapper_ident, #dstruct_path, _>)
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
    dstruct_path: &TypePath,
    wrapper_ident: &Ident,
    fn_name: &Ident,
    is_optional: bool,
    is_ref: bool,
) -> TokenStream2 {
    let wrap_fn_name = if is_optional && is_ref {
        quote!(udf::wrapper::wrap_process_buf_option_ref::<#wrapper_ident, #dstruct_path, _>)
    } else if is_optional {
        quote!(udf::wrapper::wrap_process_buf_option::<#wrapper_ident, #dstruct_path, _>)
    } else {
        quote!(udf::wrapper::wrap_process_buf::<#wrapper_ident, #dstruct_path>)
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
fn make_add_fn(dstruct_path: &TypePath, wrapper_ident: &Ident, fn_name: &Ident) -> TokenStream2 {
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
            udf::wrapper::wrap_add::<#wrapper_ident, #dstruct_path>(initid, args, is_null, error)
        }
    }
}

/// Create the function signature for aggregate `xxx_clear`
fn make_clear_fn(dstruct_path: &TypePath, wrapper_ident: &Ident, fn_name: &Ident) -> TokenStream2 {
    // SAFETY: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::udf_sys::UDF_INIT,
            is_null: *mut ::std::ffi::c_uchar,
            error: *mut ::std::ffi::c_uchar,
        ) {
            udf::wrapper::wrap_clear::<#wrapper_ident, #dstruct_path>(initid, is_null, error)
        }
    }
}

/// Create the function signature for aggregate `xxx_remove`
fn make_remove_fn(dstruct_path: &TypePath, wrapper_ident: &Ident, fn_name: &Ident) -> TokenStream2 {
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
            udf::wrapper::wrap_remove::<#wrapper_ident, #dstruct_path>
                (initid, args, is_null, error)
        }
    }
}
