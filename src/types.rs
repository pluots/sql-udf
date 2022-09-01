//! Rust wrappers for the messy C types

#[derive(Debug, PartialEq)]
pub enum ConstOpt<T> {
    Const(T),
    NonConst,
}

/// We use lifetimes so we don't copy the string converting to owned
#[derive(Debug, PartialEq)]
pub enum MaybeArg<'a> {
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
pub struct InitArgInfo<'a> {
    pub(crate) arg: MaybeArg<'a>,
    pub(crate) maybe_null: bool,
    pub(crate) attribute: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct ArgInfo<'a> {
    pub(crate) arg: RunArg<'a>,
    pub(crate) maybe_null: bool,
    pub(crate) attribute: &'a str,
}


pub trait BasicUdf {
    type Returns;
    /// Initialization function
    fn init(args: &[InitArgInfo]) -> Result<Self, String>
    where
        Self: Sized;

    fn process<'a>(&self, args: &[ArgInfo]) -> Result<Self::Returns, String>;
}

/// This trait must be implemented if this function performs aggregation.
pub trait AggregateUdf: BasicUdf {
    // Clear is required
    fn clear(&self) -> Result<(), String>;
    // Reset is not required
    // If there is an error, we will need to box it and put it on the stack
    // fn reset(&self, args: &[ArgInfo]) -> Result<(), String>;
    fn add(&self, args: &[ArgInfo]) -> Result<(), String>;

    /// Remove only applies to MariaDB
    ///
    /// https://mariadb.com/kb/en/user-defined-functions-calling-sequences/#x_remove
    fn remove(&self, _args: &[ArgInfo]) -> Result<(), String> {
        Ok(())
    }
}
