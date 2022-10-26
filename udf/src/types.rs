//! Types and traits that represent SQL interfaces

use std::fmt;

mod arg;
mod arg_list;
mod config;
mod sql_types;

// Document everything inline
#[doc(inline)]
pub use arg::*;
#[doc(inline)]
pub use arg_list::*;
#[doc(inline)]
pub use config::*;
#[doc(inline)]
pub use sql_types::*;

/// Max error message size, 0x200 = 512 bytes
pub const MYSQL_ERRMSG_SIZE: usize = 0x200;

/// Minimum size of a buffer for string results
pub const MYSQL_RESULT_BUFFER_SIZE: usize = 255;

#[derive(Debug)]
pub struct ProcessError;

impl fmt::Display for ProcessError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processing error")
    }
}

impl std::error::Error for ProcessError {}
