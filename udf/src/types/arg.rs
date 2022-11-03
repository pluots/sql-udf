//! Rust representation of SQL arguments

use core::fmt::Debug;
use std::cell::Cell;
use std::marker::PhantomData;

use udf_sys::Item_result;

use crate::types::{SqlResult, SqlType};
use crate::{Init, UdfState};

/// A single SQL argument, including its attributes
///
/// This struct contains the argument itself. It uses a typestate pattern (`S`)
/// to have slightly different functionality when used during initialization and
/// during processing.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct SqlArg<'a, S: UdfState> {
    /// The actual argument type and value
    pub value: SqlResult<'a>,

    /// A string representation of this argument's identifier
    pub attribute: &'a str,

    /// Whether or not this argument may be `NULL`
    ///
    /// We provide a getter for this just to keep things consistent
    pub(crate) maybe_null: bool,

    /// A pointer for location to change type. Must never be null.
    ///
    /// This is only needed when in the initialization phase, since we need to
    /// be able to set the type. We can look into moving this into an `extra`
    /// field instead of having `PhantomData`.
    ///
    /// We use a `Cell` here which is a type of smart pointer that allows
    /// "interior mutability". Essentially we have to do edits via methods
    /// like `.get()` and `.replace()`, but in exchange mutability doesn't need
    /// to propegate up.
    pub(crate) arg_type: &'a Cell<Item_result>,

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
    /// NULL" when we are in the `Process` step.
    #[inline]
    pub fn is_const(&self) -> bool {
        match self.value {
            SqlResult::String(v) => v.is_some(),
            SqlResult::Decimal(v) => v.is_some(),
            SqlResult::Real(v) => v.is_some(),
            SqlResult::Int(v) => v.is_some(),
        }
    }

    /// Whether or not this argument may be `NULL`
    #[inline]
    pub fn maybe_null(&self) -> bool {
        self.maybe_null
    }

    /// Retrieve the current type coercision
    #[inline]
    pub fn get_type_coercion(&self) -> SqlType {
        // `.get()` on our Cell will just copy the value
        SqlType::try_from(self.arg_type.get()).unwrap()
    }

    /// Instruct the SQL application to coerce the argument's type. This does
    /// not change the underlying value visible in `.value`.
    #[inline]
    pub fn set_type_coercion(&mut self, newtype: SqlType) {
        // .replace() on our cell will do exactly what it sounds like
        self.arg_type.replace(newtype.to_item_result());
    }
}
