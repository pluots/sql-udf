use syn::{parse_quote, Type};

/// Allowable signatures
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImplType {
    Basic,
    Aggregate,
}

/// Possible return types in SQL
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeClass {
    Int,
    Float,
    /// Bytes that can be properly returned
    BytesRef,
    /// Bytest that must be truncated
    Bytes,
}

/// Struct containing information about a return type
#[derive(Clone, Debug)]
pub struct RetType {
    pub type_: Type,
    pub is_optional: bool,
    pub type_cls: TypeClass,
}

impl RetType {
    fn new(type_: Type, is_optional: bool, fn_sig: TypeClass) -> Self {
        Self {
            type_,
            is_optional,
            type_cls: fn_sig,
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
        // Only valid integer types
        RetType::new(parse_quote! { i64 }, false, TypeClass::Int),
        RetType::new(parse_quote! { Option<i64> }, true, TypeClass::Int),
        // Only valid float types
        RetType::new(parse_quote! { f64 }, false, TypeClass::Float),
        RetType::new(parse_quote! { Option<f64> }, true, TypeClass::Float),
        // Tons of possible byte slice references. We could probably make these
        // generic somehow in the future, but it is proving tough.
        RetType::new(parse_quote! { &'a [u8] }, false, TypeClass::BytesRef),
        RetType::new(parse_quote! { Option<&'a [u8]> }, true, TypeClass::BytesRef),
        RetType::new(parse_quote! { &str }, false, TypeClass::BytesRef),
        RetType::new(parse_quote! { Option<&str> }, true, TypeClass::BytesRef),
        RetType::new(parse_quote! { &'a str }, false, TypeClass::BytesRef),
        RetType::new(parse_quote! { Option<&'a str> }, true, TypeClass::BytesRef),
        RetType::new(parse_quote! { &'static str }, false, TypeClass::BytesRef),
        RetType::new(
            parse_quote! { Option<&'static str> },
            true,
            TypeClass::BytesRef,
        ),
        RetType::new(parse_quote! { &'a String }, false, TypeClass::BytesRef),
        RetType::new(
            parse_quote! { Option<&'a String> },
            true,
            TypeClass::BytesRef,
        ),
        // Bytes types that aren't in a reference. These will get copied if they fit,
        // truncated with a stderr message if not
        RetType::new(parse_quote! { Vec<u8> }, false, TypeClass::Bytes),
        RetType::new(parse_quote! { Option<Vec<u8>>}, true, TypeClass::Bytes),
        RetType::new(parse_quote! { String }, false, TypeClass::Bytes),
        RetType::new(parse_quote! { Option<String>}, true, TypeClass::Bytes),
    ]
}
