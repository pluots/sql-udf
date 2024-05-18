use heck::AsSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, parse_quote, DeriveInput, Error, Expr, ExprLit, Ident, ImplItem,
    ImplItemType, Item, ItemFn, ItemImpl, Lit, Meta, Path, PathSegment, Token, Type, TypePath,
    TypeReference, Visibility,
};

use crate::match_variant;
use crate::types::{make_type_list, ImplType, RetType, TypeClass};

pub fn simple_udf(args: &TokenStream, input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as ItemFn);
    if !matches!(parsed.vis, Visibility::Public(_)) {
        return Error::new_spanned(parsed.vis, "UDFs must be marked `pub`")
            .into_compile_error()
            .into();
    }
    // let sig = parsed.sig
    // input
    todo!()
}
