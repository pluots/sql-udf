use syn::{parse_quote, PathSegment, Type, TypePath};

/// Allowable signatures
pub enum ImplType {
    Basic,
    Aggregate,
}

/// Possible return types in SQL
pub enum FnSigType {
    Char,
    Int,
    Float,
}

/// Struct containing information about a return type
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

pub fn make_type_list() -> Vec<RetType> {
    vec![
        RetType::new(parse_quote! { i64 }, false, FnSigType::Int),
        RetType::new(parse_quote! { f64 }, false, FnSigType::Float),
        RetType::new(parse_quote! { &'a str }, false, FnSigType::Char),
        RetType::new(parse_quote! { String }, false, FnSigType::Char),
        RetType::new(parse_quote! { Option<i64> }, true, FnSigType::Int),
        RetType::new(parse_quote! { Option<f64> }, true, FnSigType::Float),
        RetType::new(parse_quote! { Option<&'a str> }, true, FnSigType::Char),
        RetType::new(parse_quote! { Option<String> }, true, FnSigType::Char),
    ]
}
