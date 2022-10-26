use syn::{parse_quote, Type};

/// Allowable signatures
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImplType {
    Basic,
    Aggregate,
}

/// Possible return types in SQL
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FnSigType {
    Bytes,
    Int,
    Float,
}

/// Struct containing information about a return type
#[derive(Clone, Debug)]
pub struct RetType {
    pub type_: Type,
    pub is_optional: bool,
    pub fn_sig: FnSigType,
}

impl RetType {
    fn new(type_: Type, is_optional: bool, fn_sig: FnSigType) -> Self {
        Self {
            type_,
            is_optional,
            fn_sig,
        }
    }
}

/// Brute force list of acceptable types
///
/// We cannot accept `String` directly because that would imply allocation that
/// we can't allow (we would have to turn the `String` into a pointer to return
/// it, and we would never get the pointer back to free it).
pub fn make_type_list() -> Vec<RetType> {
    vec![
        RetType::new(parse_quote! { i64 }, false, FnSigType::Int),
        RetType::new(parse_quote! { Option<i64> }, true, FnSigType::Int),
        RetType::new(parse_quote! { f64 }, false, FnSigType::Float),
        RetType::new(parse_quote! { Option<f64> }, true, FnSigType::Float),
        RetType::new(parse_quote! { &'a [u8] }, false, FnSigType::Bytes),
        RetType::new(parse_quote! { Option<&'a [u8]> }, true, FnSigType::Bytes),
        RetType::new(parse_quote! { &str }, false, FnSigType::Bytes),
        RetType::new(parse_quote! { Option<&str> }, true, FnSigType::Bytes),
        RetType::new(parse_quote! { &'a str }, false, FnSigType::Bytes),
        RetType::new(parse_quote! { Option<&'a str> }, true, FnSigType::Bytes),
        RetType::new(parse_quote! { &'static str }, false, FnSigType::Bytes),
        RetType::new(
            parse_quote! { Option<&'static str> },
            true,
            FnSigType::Bytes,
        ),
        RetType::new(parse_quote! { &'a String }, false, FnSigType::Bytes),
        RetType::new(parse_quote! { Option<&'a String> }, true, FnSigType::Bytes),
    ]
}
