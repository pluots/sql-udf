//! Rust representation of SQL arguments

use core::fmt::Debug;
use std::marker::PhantomData;
use std::{mem, slice, str};

use coerce::{get_coercion, get_current_type, get_desired_or_current, set_coercion};
use udf_sys::{Item_result, UDF_ARGS};

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
            let base = self.get_base();
            let arg_buf_ptr: *const u8 = (*base.args.add(self.index)).cast();
            let arg_type = *base.arg_type.add(self.index);
            let arg_len = *base.lengths.add(self.index);

            // We can unwrap because the tag will be valid
            SqlResult::from_ptr(arg_buf_ptr, arg_type, arg_len as usize).unwrap()
        }
    }

    /// A string representation of this argument's identifier
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn attribute(&'a self) -> &'a str {
        let attr_slice;
        unsafe {
            let base = self.get_base();
            let attr_buf_ptr: *const u8 = *base.attributes.add(self.index).cast();
            let attr_len = *base.attribute_lengths.add(self.index) as usize;
            attr_slice = slice::from_raw_parts(attr_buf_ptr, attr_len);
        }
        // Ok to unwrap here, attributes must be utf8
        str::from_utf8(attr_slice)
            .map_err(|e| format!("unexpected: attribute is not valid utf8. Error: {e:?}"))
            .unwrap()
    }

    /// Simple helper method to get the internal base
    unsafe fn get_base(&'a self) -> &'a UDF_ARGS {
        &(*self.base.0.get())
    }

    /// Helper method to get a pointer to this item's arg type
    unsafe fn arg_type_ptr(&self) -> *mut Item_result {
        self.get_base().arg_type.add(self.index)
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
        unsafe { *self.get_base().maybe_null.add(self.index) != 0 }
    }

    /// Instruct the SQL application to coerce the argument's type. This does
    /// not change the underlying value visible in `.value`.
    #[inline]
    #[allow(clippy::missing_panics_doc)] // We will have a valid type
    pub fn set_type_coercion(&mut self, newtype: SqlType) {
        // We use some tricks here to store both the current type and the
        // desired coercion in `*arg_ptr`. See the `coerce` module for more
        // info.
        unsafe {
            // SAFETY: caller guarantees validity of memory location
            let arg_ptr = self.arg_type_ptr();

            // SAFETY: our tests validate size & align line up, so a C enum will
            // be the same layout as a C `int`
            *arg_ptr = mem::transmute(set_coercion(*arg_ptr as i32, newtype as i32));
        }
    }

    /// Retrieve the current type coercision
    #[inline]
    #[allow(clippy::missing_panics_doc)] // We will have a valid type
    pub fn get_type_coercion(&self) -> SqlType {
        // SAFETY: Caller guarantees
        unsafe {
            let arg_type = *self.arg_type_ptr() as i32;
            let coerced_type = get_coercion(arg_type).unwrap_or_else(|| get_current_type(arg_type));
            SqlType::try_from(coerced_type as i8).expect("critical: invalid sql type")
        }
    }

    /// Assign the currently desired coercion
    #[inline]
    pub(crate) fn flush_coercion(&mut self) {
        unsafe {
            *self.arg_type_ptr() = get_desired_or_current(*self.arg_type_ptr() as i32)
                .try_into()
                .unwrap();
        }
    }
}

mod coerce {
    //! Represent a current type and a future type within a single `.arg_type` value
    //!
    //! The purpose here is to avoid UB when we set a type coercion then try to
    //! recreate a the value-containing enum. This was only a change when we moved to
    //! the index-based representation.
    //!
    //! Representation: First byte: mask indicating if coercion is set Second byte:
    //! unused Third byte: Desired coercion Final byte: Current type

    const COERCION_SET: i32 = 0b1010_1010 << (3 * 8);
    const COERCION_SET_MASK: i32 = 0b1111_1111 << (3 * 8);
    const DESIRED_MASK: i32 = 0b1111_1111 << 8;
    const BYTE_MASK: i32 = 0b1111_1111;
    // Undo both the set mask and the desired mask
    const RESET_COERCION_DESIRED_MASK: i32 = !(COERCION_SET_MASK | DESIRED_MASK);

    /// Check if coercion is set
    fn coercion_is_set(value: i32) -> bool {
        value & COERCION_SET_MASK == COERCION_SET
    }

    /// Set coercion to a desired value
    pub fn set_coercion(current: i32, desired: i32) -> i32 {
        eprintln!("current: {current:#032b}\ndesired: {desired:#032b}");
        let val =
            RESET_COERCION_DESIRED_MASK & current | COERCION_SET | ((desired & BYTE_MASK) << 8);
        eprintln!("val: {val:#032b}");
        val
    }

    /// Get the desired coercion, ignoring currently active type
    #[allow(clippy::cast_lossless)]
    pub fn get_coercion(value: i32) -> Option<i32> {
        if coercion_is_set(value) {
            Some(((value & DESIRED_MASK) >> 8) as i8 as i32)
        } else {
            None
        }
    }

    /// Get the currently active type, ignoring coercion
    #[allow(clippy::cast_lossless)]
    pub fn get_current_type(value: i32) -> i32 {
        // We use these casts to easily sign extend
        (value & BYTE_MASK) as i8 as i32
    }

    /// Get the desiered coercion if set, otherwise the current type
    pub fn get_desired_or_current(value: i32) -> i32 {
        get_coercion(value).unwrap_or_else(|| get_current_type(value))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const TESTVALS: [i32; 8] = [-10, -5, -1, 0, 1, 5, 10, 20];

        #[test]
        fn test_unset_coercion() {
            for val in TESTVALS.iter().map(|v| *v) {
                assert_eq!(coercion_is_set(val), false);
                assert_eq!(get_coercion(val), None);
                assert_eq!(get_current_type(val), val);
                assert_eq!(get_desired_or_current(val), val);
            }
        }

        #[test]
        fn test_coercion() {
            for current in TESTVALS.iter().map(|v| *v) {
                for desired in TESTVALS.iter().map(|v| *v) {
                    let res = set_coercion(current, desired);

                    assert_eq!(coercion_is_set(res), true);
                    assert_eq!(get_coercion(res), Some(desired));
                    assert_eq!(get_current_type(res), current);
                    assert_eq!(get_desired_or_current(res), desired);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure our transmutes are sound
    #[test]
    fn verify_item_result_layout() {
        assert_eq!(mem::size_of::<Item_result>(), mem::size_of::<i32>());
        assert_eq!(mem::align_of::<Item_result>(), mem::align_of::<i32>());
    }
}
