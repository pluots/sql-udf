//! Rust representation of SQL arguments

use core::fmt::Debug;
use std::marker::PhantomData;

use crate::ffi::{SqlType, SqlTypeTag};
use crate::types::SqlResult;

/// A single SQL argument, including its attributes
///
/// This struct contains the argument itself. It uses a typestate pattern (`S`)
/// to have slightly different functionality when used during initialization and
/// during processing.
///
/// ```
/// # use udf::ffi::{SqlResult, SqlResultTag};
/// # use udf::prelude::*;
/// # let type_ptr = SqlResult::String as SqlResultTag;
/// # let content = "this is the argument";
///
/// use udf::mock::MockSqlArg;
///
/// let stype = SqlType::Real(Some(100.0f64));
/// let sql_arg: MockSqlArg<Init> = MockSqlArg::new(&stype, true, "attribute");
///
/// ```
#[derive(Debug, PartialEq)]
pub struct SqlArg<'a, S: UdfState> {
    /// The actual argument type and value
    pub arg: SqlResult<'a>,

    /// Whether or not this argument may be `NULL`
    pub maybe_null: bool,

    /// A string representation of this argument's identifier
    pub attribute: &'a str,

    /// A pointer for location to change type. Must never be null.
    ///
    /// This is only needed when in the initialization phase, since we need to
    /// be able to set the type. We can look into moving this into an `extra`
    /// field instead of having `PhantomData`.
    pub(crate) type_ptr: *mut SqlTypeTag,

    /// Internal marker for typestate pattern
    pub(crate) marker: PhantomData<S>,
}

/// This includes functions that are only applicable during initialization
impl<'a> SqlArg<'a, Init> {
    /// Determine whether an argument **may** be constant
    ///
    /// During initialization, a value is const if it is not `None`. This
    /// provides a simple test to see if this is true.
    ///
    /// There is no way to differentiate between "not const" and "const but
    /// NULL", hence the naming if this function
    pub fn maybe_const(&self) -> bool {
        match self.arg {
            SqlResult::String(v) => v.is_some(),
            SqlResult::Real(v) => v.is_some(),
            SqlResult::Int(v) => v.is_some(),
            SqlResult::Decimal(v) => v.is_some(),
        }
    }

    /// Retrieve the current type coercision
    pub fn get_type_coercion(&self) -> SqlType {
        unsafe { *self.type_ptr }.try_into().unwrap()
    }

    /// Instruct the SQL application to coerce the argument's type. This does
    /// not change the underlying
    pub fn set_type_coercion(&self, newtype: SqlType) {
        // Safety: pointer validity guaranteed by struct constructor
        unsafe { (*self.type_ptr) = newtype as SqlTypeTag };
    }
}

/// Typestate marker for the initialization phase
///
/// This is a zero-sized type that is just used to hint to the compiler that an
/// [`SqlArg`] was created in the `init` function, which allows for some special
/// methods.
#[derive(Debug, PartialEq)]
pub struct Init {}

/// Typestate marker for the processing phase
///
/// This is a zero-sized type that indicates that an [`SqlArg`] was created in
/// the `process` function. Currently there are no special methods when in this
/// state.
#[derive(Debug, PartialEq)]
pub struct Process {}

/// A state of the UDF, representing either `Init` or `Process`
///
/// This is a zero-sized type used to control what operations are allowed at
/// different times.
pub trait UdfState: Debug + PartialEq {}

impl UdfState for Init {}
impl UdfState for Process {}
