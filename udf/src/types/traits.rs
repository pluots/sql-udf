//! Module containing traits to be implemented by a user

use crate::types::{Init, Process, SqlArg};

/// This trait specifies the functions needed for a standard (non-aggregate) UDF
///
/// Implement this on a struct that is desired to
///
/// The struct will carry data between
///
/// The higher level overview is that this will:
///
/// - Call the `init(...)` function to perform setup
/// - Call the `process(...)` function to perform cleanup
///
/// The UDF specification also calls out a `deinit()` function to deallocate any
/// memory, but this is not needed here (this wrapper and Rust handles this for
/// you).
pub trait BasicUdf: Sized {
    /// This type represents the return type of the UDF function
    ///
    /// Allowed are `i64` (integer types), `f64` (real types), `String` (string
    /// or decimal types), and their `Option` versions `Option<i64>`,
    /// `Option<f64>`, `Option<String>`. Any `Option` version should be used if
    /// the function may return a SQL `NULL`.
    type Returns<'a>
    where
        Self: 'a;

    /// This is the initialization function
    ///
    /// It is expected that this function do the following:
    ///
    /// - Validate the type quantity of arguments
    /// - Set
    fn init(args: &[SqlArg<Init>]) -> Result<Self, String>;

    /// Process the actual values
    ///
    /// If you are unfamiliar with Rust, don't worry too much about the `'a` you
    /// see thrown around a lot. They are lifetime annotations and more or less
    /// say, "`self` lives at least as long as my return type does so I can
    /// return a reference to it, but `args` may not last as long so I cannot
    /// return a reference to that".
    ///
    /// If there is an error, the SQL server will return `NULL`.
    fn process<'a>(&'a mut self, args: &[SqlArg<Process>]) -> Result<Self::Returns<'a>, ()>;
}

/// This trait must be implemented if this function performs aggregation.
pub trait AggregateUdf: BasicUdf {
    // Clear is required
    fn clear(&mut self) -> Result<(), u8>;
    // Reset is not required
    // If there is an error, we will need to box it and put it on the stack
    // fn reset(&self, args: &[SqlArg]) -> Result<(), String>;
    fn add(&mut self, args: &[SqlArg<Process>], error: u8) -> Result<(), u8>;

    /// Remove only applies to MariaDB, so a default is supplied that does
    /// nothing.
    ///
    /// <https://mariadb.com/kb/en/user-defined-functions-calling-sequences/#x_remove>
    fn remove(&mut self, _args: &[SqlArg<Process>], error: u8) -> Result<(), u8> {
        Ok(())
    }
}
