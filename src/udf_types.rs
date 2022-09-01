//! Rust wrappers for the messy C types


#[derive(Debug, PartialEq)]
pub enum ConstOpt<T>  {
    Const(T),
    NonConst
}


/// We use lifetimes so we don't copy the string converting to owned
#[derive(Debug, PartialEq)]
pub enum InitArg<'a> {
    // INVALID_RESULT and ROW_RESULT are other options, but not valid for UDFs
    /// String result
    String(ConstOpt<&'a str>),
    /// Floating point
    Real(ConstOpt<f64>),
    /// Integer
    Int(ConstOpt<i64>),
    /// This is a string that is to be represented as a decimal
    Decimal(ConstOpt<&'a str>),
}

#[derive(Debug, PartialEq)]
pub enum RunArg<'a> {
    // INVALID_RESULT and ROW_RESULT are other options, but not valid for UDFs
    /// String result
    String(&'a str),
    /// Floating point
    Real(f64),
    /// Integer
    Int(i64),
    /// This is a string that is to be represented as a decimal
    Decimal(&'a str),
}


#[derive(Debug, PartialEq)]
pub struct UdfArg <'a>{
    arg: InitArg<'a>,
    maybe_null: bool,
    attribute: &'a str
}
