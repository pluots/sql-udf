//! Rust representation of SQL arguments

use core::fmt::Debug;
use std::cmp::max;
use std::marker::PhantomData;
use std::{slice, str};

use crate::types::{SqlResult, SqlType};
use crate::{ArgList, Init, UdfState};

/// A single SQL argument, including its attributes
///
/// This struct contains the argument itself. It uses a typestate pattern (`S`)
/// to have slightly different functionality when used during initialization and
/// during processing.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct SqlArg<'a, S: UdfState> {
    pub(super) base: &'a ArgList<'a, S>,
    pub(super) index: usize,
    pub(super) marker: PhantomData<S>,
}

impl<'a, T: UdfState> SqlArg<'a, T> {
    /// The actual argument type and value
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn value(&self) -> SqlResult<'a> {
        // SAFETY: Initializing API guarantees the inner struct to be valid
        unsafe {
            let base = &(*self.base.0.get());
            let arg_buf_ptr = (*base.args.add(self.index)).cast::<u8>();
            let type_ptr = base.arg_type.add(self.index);
            let arg_len = *base.lengths.add(self.index);

            // We can unwrap because the tag will be valid
            SqlResult::from_ptr(arg_buf_ptr, *type_ptr, arg_len as usize).unwrap()
        }
    }

    /// A string representation of this argument's identifier
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn attribute(&'a self) -> &'a str {
        let attr_slice;
        unsafe {
            let base = &(*self.base.0.get());
            let attr_buf_ptr: *const u8 = base.attributes.add(self.index).cast();
            let attr_len = base.attribute_lengths.add(self.index) as usize;
            attr_slice = slice::from_raw_parts(attr_buf_ptr, attr_len);
        }
        // Ok to unwrap here, attributes must be utf8. Have hit this error in
        // testing so we leave the message formatting
        str::from_utf8(attr_slice)
            .map_err(|e| {
                let subslice_min = e.valid_up_to().saturating_sub(10);
                let subslice_max = max(e.valid_up_to() + 10, attr_slice.len());
                let relevant_slice = &attr_slice[subslice_min..subslice_max];
                format!(
                    "very unexpected: attribute is not valid utf8. Error around slice {:?}",
                    &relevant_slice
                )
            })
            .unwrap()
    }
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
        match self.value() {
            SqlResult::String(v) => v.is_some(),
            SqlResult::Decimal(v) => v.is_some(),
            SqlResult::Real(v) => v.is_some(),
            SqlResult::Int(v) => v.is_some(),
        }
    }

    /// Whether or not this argument may be `NULL`
    #[inline]
    pub fn maybe_null(&self) -> bool {
        unsafe { *(*self.base.0.get()).maybe_null.add(self.index) != 0 }
    }

    /// Retrieve the current type coercision
    #[inline]
    #[allow(clippy::missing_panics_doc)] // We will have a valid type
    pub fn get_type_coercion(&self) -> Option<SqlType> {
        // `.get()` on our Cell will just copy the value
        unsafe { SqlType::try_from(*(*self.base.0.get()).arg_type).ok() }
    }

    /// Instruct the SQL application to coerce the argument's type. This does
    /// not change the underlying value visible in `.value`.
    #[inline]
    pub fn set_type_coercion(&mut self, newtype: SqlType) {
        // .replace() on our cell will do exactly what it sounds like
        unsafe { *(*self.base.0.get()).arg_type = newtype.to_item_result() };
    }
}
