//! Module containing traits to be implemented by a user

use crate::{types::{ArgList, Init, Process, SqlArg}, ProcessError};

/// This trait specifies the functions needed for a standard (non-aggregate) UDF
///
/// Implement this on a struct that is desired to carry data between calls to,
/// `init`, `process`, `clear`, `add`, and `remove`. If there is no data to be
/// shared, it may be zero-sized (e.g. `struct MyFunc{}`).
///
/// If the UDF is only basic, the process is:
///
/// - Call the `init(...)` function to perform setup and validate arguments
/// - Call the `process(...)` function to create a result from those arguments
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
    /// - Set argument type coercion if needed
    ///
    /// Error handling options are limited in all other functions, so make sure
    /// you check
    fn init<'a>(args: &'a ArgList<'a, Init>) -> Result<Self, String>;

    /// Process the actual values
    ///
    /// If you are unfamiliar with Rust, don't worry too much about the `'a` you
    /// see thrown around a lot. They are lifetime annotations and more or less
    /// say, "`self` lives at least as long as my return type does so I can
    /// return a reference to it, but `args` may not last as long so I cannot
    /// return a reference to that".
    ///
    /// If there is an error, the SQL server will return `NULL`.
    fn process<'a>(&'a mut self, args: &ArgList<Process>) -> Result<Self::Returns<'a>, ProcessError>;
}

/// This trait must be implemented if this function performs aggregation.
pub trait AggregateUdf: BasicUdf {
    // Clear is required
    fn clear(&mut self) -> Result<(), u8>;

    // Reset is not required
    // If there is an error, we will need to box it and put it on the stack
    // fn reset(&self, args: &[SqlArg]) -> Result<(), String>;
    fn add(&mut self, args: &ArgList<Process>, error: u8) -> Result<(), u8>;

    /// Remove only applies to MariaDB, so a default is supplied that does
    /// nothing.
    ///
    /// <https://mariadb.com/kb/en/user-defined-functions-calling-sequences/#x_remove>
    fn remove(&mut self, _args: &ArgList<Process>, error: u8) -> Result<(), u8> {
        Ok(())
    }
}


/// A trait that is assigned by the `#[register]` proc macro for some internal
/// checks.
pub trait BasicUdfRegistered {}
