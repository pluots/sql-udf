//! Define a list of arguments to a SQL function

#![allow(dead_code)]

use std::cell::Cell;
use std::ffi::{c_char, c_longlong, c_uchar, c_uint, c_ulong, CString};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Index;
use std::slice::SliceIndex;
use std::{fmt, panic, ptr, slice, str};

use crate::ffi::bindings::{Item_result, UDF_ARGS, UDF_INIT};
use crate::ffi::wrapper_impl::write_msg_to_buf;
use crate::{BasicUdf, Init, Process, SqlArg, SqlResult, UdfState};

/// A collection of SQL arguments
///
/// This is rusty wrapper around SQL's `UDF_ARGS` struct, providing methods to
/// easily work with arguments.
pub struct ArgList<'a, S: UdfState> {
    base: UDF_ARGS,
    // We use this zero-sized marker to hold our state
    _marker: PhantomData<&'a S>,
}

/// Derived formatting is a bit ugly, so we clean it up by using the `Vec`
/// format.
impl<'a, S: UdfState> Debug for ArgList<'a, S> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArgList")
            .field("items", &self.as_vec())
            .finish()
    }
}

impl<'a, S: UdfState> ArgList<'a, S> {
    /// Create an `ArgList` type on a `UDF_ARGS` struct
    ///
    /// Need to verify whether static lifetime is correct
    #[inline]
    #[allow(unsafe_op_in_unsafe_fn)]
    pub(crate) unsafe fn from_arg_ptr<'p>(ptr: *const UDF_ARGS) -> &'p Self {
        unsafe { &*ptr.cast::<ArgList<'_, S>>() }
    }

    /// Create a vector of arguments for easy use
    #[inline]
    pub fn as_vec(&'a self) -> Vec<SqlArg<'a, S>> {
        self.iter().collect()
    }

    /// Construct an iterator over arguments
    #[inline]
    pub fn iter(&'a self) -> Iter<'a, S> {
        self.into_iter()
    }

    /// Return `true` if there are no arguments
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the number of arguments
    #[inline]
    pub fn len(&self) -> usize {
        self.base.arg_count as usize
    }

    /// Attempt to get an argument at a given index
    #[allow(clippy::missing_inline_in_public_items)]
    pub fn get(&self, index: usize) -> Option<SqlArg<'a, S>> {
        // convenience
        let base = &self.base;

        if index >= base.arg_count as usize {
            return None;
        }

        unsafe {
            let arg_buf_ptr = (*base.args.add(index)).cast::<u8>();
            let attr_buf_ptr = (*base.attributes.add(index)).cast::<u8>();
            let type_ptr = base.arg_type.add(index);
            let arg_len = *base.lengths.add(index);
            let attr_len = *base.attribute_lengths.add(index);
            let maybe_null = *base.maybe_null.add(index) != 0;

            // Attributes are identifiers in SQL and are always UTF8
            let attr_slice = slice::from_raw_parts(attr_buf_ptr, attr_len as usize);
            let attribute = str::from_utf8(attr_slice).unwrap();
            let arg = SqlResult::from_ptr(arg_buf_ptr, *type_ptr, arg_len as usize).unwrap();

            Some(SqlArg {
                value: arg,
                maybe_null,
                attribute,
                arg_type: &*(type_ptr as *const Cell<Item_result>),
                marker: PhantomData,
            })
        }
    }
}

/// Trait for being able to iterate arguments
impl<'a, S: UdfState> IntoIterator for &'a ArgList<'a, S> {
    type Item = SqlArg<'a, S>;

    type IntoIter = Iter<'a, S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

/// Trait for being able to iterate arguments
impl<'a, S: UdfState> IntoIterator for &'a mut ArgList<'a, S> {
    type Item = SqlArg<'a, S>;

    type IntoIter = Iter<'a, S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

/// Iterator over arguments in a [`ArgList`]
///
/// This struct is produced by invoking `into_iter()` on a [`ArgList`]
// #[derive(Debug, PartialEq, Clone)]
pub struct Iter<'a, S: UdfState> {
    base: &'a ArgList<'a, S>,
    // Keep consistent with underlying UDF_ARGS
    n: c_uint,
}

impl<'a, S: UdfState> Iter<'a, S> {
    fn new(base: &'a ArgList<'a, S>) -> Self {
        Self { base, n: 0 }
    }
}

impl<'a, S: UdfState> Iterator for Iter<'a, S> {
    type Item = SqlArg<'a, S>;

    /// Get the next argument
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Increment counter, check if we are out of bounds
        if self.n >= self.base.base.arg_count {
            return None;
        }

        let ret = self.base.get(self.n as usize);
        self.n += 1;

        ret
    }

    /// We know exactly how many items we have remaining, so can implement this
    /// (which allows some optimizations).
    ///
    /// See [`std::iter::Iterator::size_hint`] for this method's use.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.base.base.arg_count - self.n) as usize;
        (remaining, Some(remaining))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{align_of, size_of};

    use super::*;

    // Verify no size issues
    #[test]
    fn args_size_init() {
        assert_eq!(
            size_of::<UDF_ARGS>(),
            size_of::<ArgList<Init>>(),
            concat!("Size of: ", stringify!(UDF_ARGS))
        );
        assert_eq!(
            align_of::<UDF_ARGS>(),
            align_of::<ArgList<Init>>(),
            concat!("Alignment of ", stringify!(UDF_ARGS))
        );
    }

    // Verify no size issues
    #[test]
    fn args_size_process() {
        assert_eq!(
            size_of::<UDF_ARGS>(),
            size_of::<ArgList<Process>>(),
            concat!("Size of: ", stringify!(UDF_ARGS))
        );
        assert_eq!(
            align_of::<UDF_ARGS>(),
            align_of::<ArgList<Process>>(),
            concat!("Alignment of ", stringify!(UDF_ARGS))
        );
    }
}
