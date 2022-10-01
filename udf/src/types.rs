//! Types and traits that represent SQL interfaces

use std::fmt;

mod sql_arg;
mod sql_arg_list;
mod sql_result;

#[doc(inline)]
pub use sql_arg::*;
#[doc(inline)]
pub use sql_arg_list::*;
#[doc(inline)]
pub use sql_result::*;

#[doc(inline)]
pub use crate::ffi::SqlType;

#[derive(Debug)]
pub struct ProcessError;

impl fmt::Display for ProcessError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processing error")
    }
}

impl std::error::Error for ProcessError {}
