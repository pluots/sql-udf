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

pub mod ffi;
pub mod types;

pub use types::*;
