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
mod udf_types_c;
mod wrapper;

pub use udf_types::{InitArg,UdfArg  };

enum ItemUdfType {}

pub trait UdfString {
    fn init();
}

pub trait UdfInt {
    // fn init(initid: UdfInit);
    fn process();
}
pub trait UdfReal {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
