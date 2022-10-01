//! Module containing traits to be implemented by a user

use crate::types::{ArgList, Init, Process, SqlArg};
use crate::ProcessError;

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
    /// # Errors
    ///
    /// If your function is not able to work with the given arguments, return a
    /// helpful error message explaining why. Max error size is
    /// `MYSQL_ERRMSG_SIZE` (512) bits, and will be truncated if any longer.
    /// `MySql` recommends keeping these error messages under 80 characters to
    /// fit in a terminal, but personal I'd prefer a helpful message than
    /// something useless that fits in one line.
    ///
    /// Error handling options are limited in all other functions, so make sure
    /// you check thoroughly for any possible errors that may arise. These may
    /// include:
    ///
    /// - Incorrect argument quantity or position
    /// - Incorrect argument types
    /// - Values that are `maybe_null()` when you cannot accept them
    fn init<'a>(args: &'a ArgList<'a, Init>) -> Result<Self, String>;

    /// Process the actual values
    ///
    /// If you are unfamiliar with Rust, don't worry too much about the `'a` you
    /// see thrown around a lot. They are lifetime annotations and more or less
    /// say, "`self` lives at least as long as my return type does so I can
    /// return a reference to it, but `args` may not last as long so I cannot
    /// return a reference to that".
    ///
    /// # Errors
    ///
    /// If there is some sort of unrecoverable problem at this point, just return a
    /// [`ProcessError`]. This will make the SQL server will return
    /// `NULL`.
    fn process<'a>(
        &'a mut self,
        args: &ArgList<Process>,
    ) -> Result<Self::Returns<'a>, ProcessError>;
}

/// This trait must be implemented if this function performs aggregation.
pub trait AggregateUdf: BasicUdf {
    /// Clear is required
    ///
    /// # Errors
    ///
    /// Errors for aggregate functions are not super useful.
    fn clear(&mut self) -> Result<(), u8>;

    /// Add an item to aggregates
    ///
    /// # Errors
    ///
    ///
    fn add(&mut self, args: &ArgList<Process>, error: u8) -> Result<(), u8>;

    /// Remove only applies to `MariaDB`, so a default is supplied that does
    /// nothing.
    ///
    /// # Errors
    ///
    /// As with [`clear()`] and [`add()`], this method takes an error
    ///
    /// <https://mariadb.com/kb/en/user-defined-functions-calling-sequences/#x_remove>
    #[inline]
    fn remove(&mut self, _args: &ArgList<Process>, error: u8) -> Result<(), u8> {
        Ok(())
    }
}

/// A trait that is assigned by the `#[register]` proc macro for some internal
/// checks.
pub trait BasicUdfRegistered {}
