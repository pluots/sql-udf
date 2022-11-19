//! Define a list of arguments to a SQL function

#![allow(dead_code)]

use std::cell::UnsafeCell;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;

use udf_sys::UDF_ARGS;

use crate::{Init, SqlArg, UdfState};

/// A collection of SQL arguments
///
/// This is rusty wrapper around SQL's `UDF_ARGS` struct, providing methods to
/// easily work with arguments.
#[repr(transparent)]
pub struct ArgList<'a, S: UdfState>(
    /// UnsafeCell indicates to the compiler that this struct may have interior
    /// mutability (i.e., cannot make som optimizations)
    pub(super) UnsafeCell<UDF_ARGS>,
    /// We use this zero-sized marker to hold our state
    PhantomData<&'a S>,
);

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
    #[inline]
    pub(crate) unsafe fn from_arg_ptr<'p>(ptr: *const UDF_ARGS) -> &'p Self {
        &*ptr.cast()
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
        // SAFETY: unsafe when called from different threads, but we are `!Sync`
        unsafe { (*self.0.get()).arg_count as usize }
    }

    /// Safely get an argument at a given index. It it is not available, `None`
    /// will be returned.
    #[inline]
    #[allow(clippy::missing_panics_doc)] // Attributes are identifiers in SQL and are always UTF8
    pub fn get(&'a self, index: usize) -> Option<SqlArg<'a, S>> {
        let base = unsafe { &*self.0.get() };

        if index >= base.arg_count as usize {
            return None;
        }
        Some(SqlArg {
            base: self,
            index,
            marker: PhantomData,
        })
    }
}

impl<'a> ArgList<'a, Init> {
    /// Apply the pending coercion for all arguments. Meant to be run just
    /// before exiting the `init` function within proc macro calls.
    #[inline]
    #[doc(hidden)]
    pub fn flush_all_coercions(&self) {
        self.iter().for_each(|mut a| a.flush_coercion());
    }
}

/// Trait for being able to iterate arguments
impl<'a, S: UdfState> IntoIterator for &'a ArgList<'a, S> {
    type Item = SqlArg<'a, S>;

    type IntoIter = Iter<'a, S>;

    /// Construct a new
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

/// Iterator over arguments in a [`ArgList`]
///
/// This struct is produced by invoking `into_iter()` on a [`ArgList`]
#[derive(Debug)]
pub struct Iter<'a, S: UdfState> {
    base: &'a ArgList<'a, S>,
    n: usize,
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
        if self.n >= self.base.len() {
            return None;
        }

        let ret = self.base.get(self.n);
        self.n += 1;

        ret
    }

    /// We know exactly how many items we have remaining, so can implement this
    /// (which allows some optimizations).
    ///
    /// See [`std::iter::Iterator::size_hint`] for this method's use.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.base.len() - self.n;
        (remaining, Some(remaining))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{align_of, size_of};

    use super::*;
    use crate::prelude::*;

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
