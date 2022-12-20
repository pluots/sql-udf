//! Non-public module to assist macro with wrapping functions
//!
//! Warning: This module should be considered unstable and generally not for
//! public use

#[cfg(feature = "logging-debug")]
mod debug;
mod functions;
mod helpers;
mod process;

pub use functions::{wrap_add, wrap_clear, wrap_deinit, wrap_init, wrap_remove, BufConverter};
pub(crate) use helpers::*;
pub use process::{
    wrap_process_basic, wrap_process_basic_option, wrap_process_buf, wrap_process_buf_option,
};

#[cfg(test)]
mod tests;
