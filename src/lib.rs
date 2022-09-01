#![deny(unsafe_op_in_unsafe_fn)]

//!
//!
//! # Example
//!
//! Your struct type should hold anything that you want to carry between the
//! functions.
//!
//! ```
//!
//!
//! ```
//!
//! # Behind the Scenes
//!
//! Store the struct to the *ptr before exit
//!
//! Define the basic traits here

mod udf_types;
mod udf_types_ffi;
mod wrapper;

use udf_types::ArgInfo;
pub use udf_types::{InitArgInfo, MaybeArg};

enum ItemUdfType {}

pub trait UdfBase {
    /// Initialization function
    fn init(args: &[InitArgInfo]) -> Result<Self, String>
    where
        Self: Sized;
}
pub trait Str: UdfBase {
    fn process<'a>(&self, args: &[ArgInfo]) -> Result<Option<&'a str>, String>;
}
pub trait StrNotNull: UdfBase {
    fn process<'a>(&self, args: &[ArgInfo]) -> Result<&'a str, String>;
}

// Maybe decimals need to set their lengths?
pub trait Decimal: UdfBase {
    fn process<'a>(&self, args: &[ArgInfo]) -> Result<Option<&'a str>, String>;
}
pub trait DecNotNull: UdfBase {
    fn process<'a>(&self, args: &[ArgInfo]) -> Result<&'a str, String>;
}

pub trait Float: UdfBase {
    fn process(&self, args: &[ArgInfo]) -> Result<Option<f64>, String>;
}
pub trait FloatNotNull: UdfBase {
    fn process(&self, args: &[ArgInfo]) -> Result<f64, String>;
}

pub trait Integer: UdfBase {
    fn process(&self, args: &[ArgInfo]) -> Result<Option<u64>, String>;
}
pub trait IntNotNull: UdfBase {
    fn process(&self, args: &[ArgInfo]) -> Result<u64, String>;
}

pub trait Aggregate {
    // Clear is required
    fn clear(&self) -> Result<(), String>;
    // Reset is not required
    // If there is an error, we will need to box it and put it on the stack
    // fn reset(&self, args: &[ArgInfo]) -> Result<(), String>;
    fn add(&self, args: &[ArgInfo]) -> Result<(), String>;

    /// Remove only applies to MariaDB
    ///
    /// https://mariadb.com/kb/en/user-defined-functions-calling-sequences/#x_remove
    fn remove(&self, args: &[ArgInfo]) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
