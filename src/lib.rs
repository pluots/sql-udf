#![deny(unsafe_op_in_unsafe_fn)]

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

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

extern crate udf_derive;
pub use udf_derive::register;

pub mod types;
pub mod ffi;

use types::ArgInfo;
pub use types::{InitArgInfo, MaybeArg};


/// May be &str, f64, or i64
/// Use result() if not null
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
