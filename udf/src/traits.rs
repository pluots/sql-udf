//! Module containing traits to be implemented by a user
//!
//! A basic UDF just needs to implement [`BasicUdf`]. An aggregate UDF needs to
//! implement both [`BasicUdf`] and [`AggregateUdf`].

use core::fmt::Debug;
use std::num::NonZeroU8;

use crate::types::{ArgList, UdfCfg};
use crate::ProcessError;

/// This trait specifies the functions needed for a standard (non-aggregate) UDF
///
/// Implement this on any struct in order to create a UDF. That struct can
/// either be empty (usually the case for simple functions), or contain data
/// that will be shared among all the UDF functions.
///
/// If the UDF is basic (non-aggregate), the process is:
///
/// - Caller (SQL server) calls `init()` with basic argument information
/// - `init()` function (defined here) validates the arguments, does
///   configuration (if needed), and configures and returns the `Self` struct
/// - For each row, the caller calls `process(...)` with the relevant arguments
/// - `process()` function (defined here) accepts an instance of `self` (created
///   during init) and updates it as needed, and produces a result for that row
///
/// The UDF specification also calls out a `deinit()` function to deallocate any
/// memory, but this is not needed here (handled by this wrapper).
pub trait BasicUdf: Sized {
    /// This type represents the return type of the UDF function.
    ///
    /// There are a lot of options, with some rules to follow. Warning! tedious
    /// explanation below, just skip to the next section if you don't need the
    /// details.
    ///
    /// - `f64` (real), `i64` (integer), and `[u8]` (string/blob) are the three
    ///   fundamental types
    /// - Any `Return` can be an `Option<something>` if the result is
    ///   potentially nullable
    /// - There is no meaningful difference between `String`, `Vec<u8>`, `str`,
    ///   and `[u8]` - return whichever is most convenient (following the below
    ///   rules), since they always just get copied to a buffer. Any of these
    ///   types are acceptable for returning `string` or `decimal` types.
    /// - Out of these buffer options, prefer returning `&'static str` or
    ///   `&'static [u8]` where possible. These are usable when only returning
    ///   const/static values.
    /// - "Owned allocated" types (`String`, `Vec<u8>`) may _only_ be returned
    ///   if their length is known to be <= 255. If this is exceeded, the result
    ///   will be truncated.
    /// - The last resort (but most powerful) options are `&'a str` and `&'a
    ///   [u8]`, for any-length return types. To use these, your function's
    ///   struct must have a `String` or `Vec<u8>` field. Just return a
    ///   reference to your struct's field (GATs allow this to work)
    ///
    /// This all sounds somewhat complex, which is just due to the limitations
    /// of the underlying C SQL API. Choosing a type may seem tricky at first
    /// but rest assured, whatever successfully compiles will work.
    ///
    /// The flow chart below helps clarify some of the decisions making:
    ///
    /// ```text
    ///     Desired                Use Option<T> if the result may be null
    ///   Return Type
    ///  ┉┉┉┉┉┉┉┉┉┉┉┉┉
    /// ╭─────────────╮
    /// │   integer   ├─> i64 / Option<i64>
    /// ╰─────────────╯
    /// ╭─────────────╮
    /// │    float    ├─> f64 / Option<f64>
    /// ╰─────────────╯
    ///                   ╭────────────╮
    /// ╭─────────────╮   │  static    ├─> &'static str / Option<&'static str>
    /// │ utf8 string ├─> │            │
    /// ╰─────────────╯   │            │   ╭────────────╮
    ///                   │  dynamic   ├─> │ len ≤ 255  ├─> String / Option<String>
    ///                   ╰────────────╯   │            │
    ///                                    │ len ?      ├─> &'a str / Option<&'a str>
    ///                                    ╰────────────╯
    /// ╭─────────────╮   ╭────────────╮
    /// │  non utf8   │   │  static    ├─> &'static str / Option<&'static str>
    /// │ string/blob ├─> │            │
    /// ╰─────────────╯   │            │   ╭────────────╮
    ///                   │  dynamic   ├─> │ len ≤ 255  ├─> Vec<u8> / Option<Vec<u8>>
    ///                   ╰────────────╯   │            │
    ///                                    │ len ?      ├─> &'a [u8] / Option<&'a [u8]>
    ///                                    ╰────────────╯
    /// ```
    ///
    /// Important note:
    type Returns<'a>
    where
        Self: 'a;

    /// This is the initialization function
    ///
    /// It is expected that this function do the following:
    ///
    /// - Check that arguments are the proper type
    /// - Check whether the arguments are const and have a usable value (can
    ///   provide some optimizations)
    ///
    /// # Errors
    ///
    /// If your function is not able to work with the given arguments, return a
    /// helpful error message explaining why. Max error size is
    /// `MYSQL_ERRMSG_SIZE` (512) bits, and will be truncated if any longer.
    ///
    /// `MySql` recommends keeping these error messages under 80 characters to
    /// fit in a terminal, but personal I'd prefer a helpful message over
    /// something useless that fits in one line.
    ///
    /// Error handling options are limited in all other functions, so make sure
    /// you check thoroughly for any possible errors that may arise, to the best
    /// of your ability. These may include:
    ///
    /// - Incorrect argument quantity or position
    /// - Incorrect argument types
    /// - Values that are `maybe_null()` when you cannot accept them
    fn init<'a>(cfg: &UdfCfg<Init>, args: &'a ArgList<'a, Init>) -> Result<Self, String>;

    /// Process the actual values and return a result
    ///
    /// If you are unfamiliar with Rust, don't worry too much about the `'a` you
    /// see thrown around a lot. They are lifetime annotations and more or less
    /// say, "`self` lives at least as long as my return type does so I can
    /// return a reference to it, but `args` may not last as long so I cannot
    /// return a reference to that".
    ///
    /// # Arguments
    ///
    /// - `args`: Iterable list of arguments of the `Process` type
    /// - `error`: This is only applicable when using aggregate functions and
    ///   can otherwise be ignored. If using aggregate functions, this provides
    ///   the current error value as described in [`AggregateUdf::add()`].
    ///
    /// # Return Value
    ///
    /// Assuming success, this function must return something of type
    /// `Self::Returns`. This will be the value for the row (standard functions)
    /// or for the entire group (aggregate functions).
    ///
    /// # Errors
    ///
    /// If there is some sort of unrecoverable problem at this point, just
    /// return a [`ProcessError`]. This will make the SQL server return `NULL`.
    /// As mentioned, there really aren't any good error handling options at
    /// this point other than that, so try to catch all possible errors in
    /// [`BasicUdf::init`].
    ///
    /// [`ProcessError`] is just an empty type.
    fn process<'a>(
        &'a mut self,
        cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError>;
}

/// This trait must be implemented if this function performs aggregation.
///
/// The basics of aggregation are simple:
///
/// - `init` is called once per result set (same as non-aggregate)
/// - `clear` is called once per group within the result set, and should reset
///   your struct
/// - `add` is called once per row in the group, and should add the current row
///   to the struct as needed
/// - `process` is called at the end of each group, and should produce the
///   result value for that group
///
/// # Aggregate Error Handling
///
/// Error handling for aggregate functions is weird, and does not lend itself to
/// easy understandability. The following is my best understanding of the
/// process:
///
/// - Any aggregate function may set a nonzero error (Represented here in return
///   value by `Err(NonZeroU8)`). The value is not important, can be something
///   internal
/// - These errors do not stop the remaining `add()`/`remove()` functions from
///   being called, but these functions do receive the error (and so may choose
///   to do nothing if there is an error set)
/// - Errors are not reset on `clear()`; you must do this manually (Hence
///   `error` being mutable in this function signature)
///
/// In order to enforce some of these constraints, we use `NonZeroU8` to
/// represent error types (which has the nice side effect of being optimizable).
/// Unfortunately, it is somewhat cumbersome to use, e.g.: `return
/// Err(NonZeroU8::new(1).unwrap());`
pub trait AggregateUdf: BasicUdf {
    /// Clear is run once at the beginning of each aggregate group and should
    /// reset everything needed in the struct.
    ///
    /// # Errors
    ///
    /// The `error` arg provides the error value from the previous group, and
    /// this function may choose to reset it (that is probably a good idea to
    /// do). `error` will be `None` if there is currently no error.
    ///
    /// To clear the error, simply return `Ok(())`.
    ///
    /// Return an error if something goes wrong within this function, or if you
    /// would like to propegate the previous error.
    fn clear(&mut self, cfg: &UdfCfg<Process>, error: Option<NonZeroU8>) -> Result<(), NonZeroU8>;

    /// Add an item to the aggregate
    ///
    /// Usually this is implemented by adding something to an intemdiate value
    /// inside the core struct type.
    ///
    /// # Errors
    ///
    /// Hit a problem? Return an integer, which may or may not be meaningful to
    /// you. This can be done with `return Err(NonZeroU8::new(1).unwrap());`.
    ///
    /// The `error` argument tells you if there has been an error at some point,
    /// and the return value also detemines whether to propegate/modify the
    /// error (probably what you want) or clear it (I can't think of any good
    /// reason to do this in `add()`). If you would like to propegate the error
    /// without action, just add the following as the first line of the
    /// function:
    ///
    /// ```
    /// # use std::num::NonZeroU8;
    /// # fn tmp(error: Option<NonZeroU8>) -> Result<(), NonZeroU8> {
    /// error.map_or(Ok(()), Err)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// If you do this,
    fn add(
        &mut self,
        cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8>;

    /// Remove only applies to `MariaDB`, for use with window functions; i.e.,
    /// `remove` will be called on a row that should be removed from the current
    /// set (has moved out of the window).
    ///
    /// This is optional; a default is supplied so no action is needed. If you
    /// would like to use `remove`, just reimplement it.
    ///
    /// <https://mariadb.com/kb/en/user-defined-functions-calling-sequences/#x_remove>
    ///
    /// # Errors
    ///
    /// Errors are handled the same as with [`AggregateUdf::add()`], see the
    /// description there
    #[inline]
    #[allow(unused_variables)] // Allow without an underscore for cleaner docs
    fn remove(
        &mut self,
        cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        unimplemented!()
    }
}

/// A state of the UDF, representing either [`Init`] or [`Process`]
///
/// This is a zero-sized type used to control what operations are allowed at
/// different times.
pub trait UdfState: Debug + PartialEq {}

/// Typestate marker for the initialization phase
///
/// This is a zero-sized type. It just allows for specific methods to be
/// implemented only on types that were created during the `init` function.
#[derive(Debug, PartialEq, Eq)]
pub struct Init;

/// Typestate marker for the processing phase
///
/// This is a zero-sized type, indicating that a type was created in the
/// `process` function.
#[derive(Debug, PartialEq, Eq)]
pub struct Process;

impl UdfState for Init {}
impl UdfState for Process {}
