// #![allow(unused_imports)]
// #![allow(unused_variables)]
#![allow(unused)]
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

// fn make_fn_name(s: &str, ident: Ident) -> Ident {
//     let formatted = format!(s, AsSnakeCase(ident.to_string())).as_str();
//     Ident::new(formatted, Span::call_site());
// }

// #[derive(Debug)]
// struct ItemStruct {
//     impl_token: Token![impl],
//     ident: Ident,
//     // brace_token: token::Brace,
//     // fields: Punctuated<Field, Token![,]>,
// }

// impl Parse for ItemStruct {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         // let content;
//         Ok(ItemStruct {
//             impl_token: input.parse()?,
//             ident: input.parse()?,
//             // brace_token: braced!(content in input),
//             // fields: content.parse_terminated(Field::parse_named)?,
//         })
//     }
// }

macro_rules! format_ident_str {
    ($formatter: tt, $ident: ident) => {
        Ident::new(
            format!($formatter, AsSnakeCase($ident.to_string())).as_str(),
            Span::call_site(),
        )
    };
}

type PathColonPunc = Punctuated<PathSegment, Colon2>;

/// Verify that an ItemImpl implements BasicUdf (in any of its pathing options)
fn verify_impl_basicudf(itemimpl: &ItemImpl) -> bool {
    let implemented = &itemimpl.trait_.as_ref().unwrap().1.segments;
    let acceptable_impls: [PathColonPunc; 3] = [
        parse_quote! {udf::traits::BasicUdf},
        parse_quote! {udf::BasicUdf},
        parse_quote! {BasicUdf},
    ];

    if !acceptable_impls.contains(&implemented) {
        return false;
    }

    true
}

/// Verify that an ItemImpl implements BasicUdf (in any of its pathing options)
fn verify_impl_aggudf(itemimpl: &ItemImpl) -> bool {
    let implemented = &itemimpl.trait_.as_ref().unwrap().1.segments;
    let acceptable_impls: [PathColonPunc; 3] = [
        parse_quote! {udf::traits::AggregateUdf},
        parse_quote! {udf::AggregateUdf},
        parse_quote! {AggregateUdf},
    ];

    if !acceptable_impls.contains(&implemented) {
        return false;
    }
    true
}

/// # Arguments
///
/// - args: a stream of everything inside `(...)` (e.g.
/// `#[register(bin=false, a=2)]` will give the stream for `bin=false, a=2`
/// - item: the item contained within the stream
pub(crate) fn register(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_cpy = input.clone();
    let parsed = parse_macro_input!(input as ItemImpl);

    let impls_basic = verify_impl_basicudf(&parsed);
    let impls_agg = verify_impl_aggudf(&parsed);

    if !(impls_basic || impls_agg) {
        return Error::new_spanned(&parsed, "Expected trait `BasicUdf` or `AggregateUdf`")
            .into_compile_error()
            .into();
    }

    // Extract the last part of the implemented path
    // e.g. crate::mod::MyStruct will return MyStruct
    let ty = match *parsed.self_ty {
        Type::Path(v) => v.path.segments.last().unwrap().clone(),
        v => {
            return Error::new_spanned(v, "expected a path")
                .into_compile_error()
                .into()
        }
    };

    eprintln!("{ty:#?}");

    // Get the return type from the macro
    // There is only one type for this trait, which is "Returns"
    let impl_item_type = &parsed
        .items
        .iter()
        .find_map(match_variant!(ImplItem::Type))
        .unwrap()
        .ty;

        
    // let impl_item_type = &tmp.ty;

    // // eprintln!("{impl_item_type:#?}");

    // let type_str_ref: TypeReference = parse_quote! {&'a str};
    // // let type_str_ref_opt: TypeReference = parse_quote!{Option<&'a str>};

    // let type_string: TypePath = parse_quote! {String};
    // let type_string_opt: TypePath = parse_quote! {Option<String>};

    // let type_int: TypePath = parse_quote! {i64};
    // let type_int_opt: TypePath = parse_quote! {Option<i64>};

    // let type_float: TypePath = parse_quote! {f64};
    // let type_float_opt: TypePath = parse_quote! {Option<f64>};

    // // eprintln!("{:#?}\n\n", impl_item_type);

    // if let Type::Reference(xx) = impl_item_type {
    //     // eprintln!("\n\nQQ:\n{qq:#?}, {}", str_ref == *xx);
    //     eprintln!("str ref: {}", *xx == type_str_ref);
    //     // eprintln!("str ref opt: {}",*xx==type_str_ref_opt);
    // } else if let Type::Path(xx) = impl_item_type {
    //     eprintln!("string: {}", *xx == type_string);
    //     // eprintln!("str ref opt: {}",*xx==type_string_opt);
    //     eprintln!("int: {}", *xx == type_int);
    //     // eprintln!("str ref opt: {}",*xx==type_int_opt);
    //     eprintln!("float: {}", *xx == type_float);
    //     // eprintln!("str ref opt: {}",*xx==type_float_opt);
    // } else {
    //     eprintln!("panicing!");
    //     panic!(
    //         "expected `Result` to be one of `{:?}`, `{:?}`, `{:?}`, `{:?}` but got {:?}",
    //         type_str_ref, type_string, type_int, type_float, impl_item_type
    //     );
    // }

    input_cpy
}

/// Given the name of a type or struct, create a function that will be evaluated
fn make_init_fn(type_ident: Ident, fn_name: Ident) -> proc_macro2::TokenStream {
    // Safety: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::ffi::bindings::UDF_INIT,
            args: *mut udf::ffi::bindings::UDF_ARGS,
            message: *mut std::os::raw::c_char,
        ) -> bool
        {
            // We set the following values based on our proc macro args
            // - `initid.max_length`
            // - `initd.decimals`
            // - `initd.const_item`

            // The following is set based on the return type
            // - `initd.maybe_null`

            unsafe {
                udf::ffi::wrapper::wrap_init::<#type_ident>(initid, args, message)
            }
        }
    }
}

fn make_deinit_fn(struct_ident: Ident, fn_name: Ident) -> proc_macro2::TokenStream {
    // Safety: we just minimally wrap the functions here, safety is handled
    // between our caller and callee
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name (
            initid: *mut udf::ffi::bindings::UDF_INIT,
        ) {
            unsafe {
                udf::ffi::wrapper::deinit_wrapper::<#struct_ident>(initid)
            }
        }
    }
}

// pub(crate) fn register(_args: TokenStream, item: TokenStream) -> TokenStream {
//     let item_tmp = item.clone();
//     let raw_input = parse_macro_input!(item as Item);

//     // We aren't doing a derive macro but DeriveInput helps us easily get the identifier
//     // type_ident is the name of our struct (or enum)
//     let type_ident = parse_macro_input!(item_tmp as DeriveInput).ident;
//     let init_fn_name = format_ident_str!("{}_init", type_ident);
//     let process_fn_name = format_ident_str!("{}", type_ident);
//     let deinit_fn_name = format_ident_str!("{}_deinit", type_ident);

//     let init_fn = make_init_fn(type_ident.clone(), init_fn_name);
//     let deinit_fn = make_deinit_fn(type_ident, deinit_fn_name);

//     let expanded = quote! {
//         #raw_input

//         #init_fn

//         #deinit_fn
//     };

//     TokenStream::from(expanded)
// }
