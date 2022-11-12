//! Rust representation of `UDF_INIT`

#![allow(clippy::useless_conversion, clippy::unnecessary_cast)]

use std::cell::UnsafeCell;
use std::marker::PhantomData;

use udf_sys::UDF_INIT;

use crate::{Init, UdfState};

/// Helpful constants related to the `max_length` parameter
///
/// These can be helpful when calling [`UdfCfg::set_max_len()`]
#[repr(u32)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MaxLenOptions {
    /// The default max length for integers is 21
    IntDefault = 21,

    /// The default max length of a real value is 13 plus the result of
    /// [`UdfCfg::get_decimals()`]
    RealBase = 13,

    /// A `blob` can be up to 65 KiB.
    Blob = 1 << 16,

    /// A `mediumblob` can be up to 16 MiB.
    MediumBlob = 1 << 24,
}

/// A collection of SQL arguments
///
/// This is rusty wrapper around SQL's `UDF_INIT` struct, providing methods to
/// easily and safely work with arguments.
#[derive(Debug)]
#[repr(transparent)]
pub struct UdfCfg<S: UdfState>(UnsafeCell<UDF_INIT>, PhantomData<S>);

impl<S: UdfState> UdfCfg<S> {
    /// Create an `ArgList` type on a `UDF_ARGS` struct
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `ptr` is valid and remains valid for the
    /// lifetime of the returned value
    #[inline]
    pub(crate) unsafe fn from_raw_ptr<'p>(ptr: *const UDF_INIT) -> &'p Self {
        &*ptr.cast()
    }

    /// Consume a box and store its pointer in this `UDF_INIT`
    ///
    /// This takes a boxed object, turns it into a pointer, and stores that
    /// pointer in this struct. After calling this function, [`retrieve_box`]
    /// _must_ be called to free the memory!
    pub(crate) fn store_box<T>(&self, b: Box<T>) {
        let box_ptr = Box::into_raw(b);
        // SAFETY: unsafe when called from different threads, but we are `!Sync`
        // here
        unsafe { (*self.0.get()).ptr = box_ptr.cast() };
    }

    /// Given this struct's `ptr` field is a boxed object, turn that pointer
    /// back into a box
    ///
    /// # Safety
    ///
    /// T _must_ be the type of this struct's pointer, likely created with
    /// [`store_box`]
    pub(crate) unsafe fn retrieve_box<T>(&self) -> Box<T> {
        Box::from_raw((*self.0.get()).ptr.cast::<T>())
    }

    /// Retrieve the setting for whether this UDF may return `null`
    ///
    /// This defaults to true if any argument is nullable, false otherwise
    #[inline]
    pub fn get_maybe_null(&self) -> bool {
        // SAFETY: unsafe when called from different threads, but we are `!Sync`
        unsafe { (*self.0.get()).maybe_null }
    }

    /// Retrieve the setting for number of decimal places
    ///
    /// This defaults to the longest number of digits of any argument, or 31 if
    /// there is no fixed number
    #[inline]
    pub fn get_decimals(&self) -> u32 {
        // SAFETY: unsafe when called from different threads, but we are `!Sync`
        unsafe { (*self.0.get()).decimals as u32 }
    }

    /// Set the number of decimals this function returns
    ///
    /// This can be changed at any point in the UDF (init or process)
    #[inline]
    pub fn set_decimals(&self, v: u32) {
        // SAFETY: unsafe when called from different threads, but we are `!Sync`
        unsafe { (*self.0.get()).decimals = v.into() };
    }

    /// Retrieve the current maximum length setting for this in-progress UDF
    #[inline]
    pub fn get_max_len(&self) -> u64 {
        // SAFETY: unsafe when called from different threads, but we are `!Sync`
        unsafe { (*self.0.get()).max_length as u64 }
    }

    /// Get the current `const_item` value
    #[inline]
    pub fn get_is_const(&self) -> bool {
        // SAFETY: unsafe when called from different threads, but we are `!Sync`
        unsafe { (*self.0.get()).const_item }
    }
}

/// Implementations of actions on a `UdfCfg` that are only possible during
/// initialization
impl UdfCfg<Init> {
    /// Set whether or not this function may return null
    #[inline]
    pub fn set_maybe_null(&self, v: bool) {
        // SAFETY: unsafe when called from different threads, but we are `!Sync`
        unsafe { (*self.0.get()).maybe_null = v };
    }

    /// Set the maximum possible length of this UDF's result
    ///
    /// This is mostly relevant for String and Decimal return types. See
    /// [`MaxLenOptions`] for possible defaults, including `BLOB` sizes.
    #[inline]
    pub fn set_max_len(&self, v: u32) {
        // SAFETY: unsafe when called from different threads, but we are `!Sync`
        unsafe { (*self.0.get()).decimals = v.into() };
    }

    /// Set a new `const_item` value
    ///
    /// Set this to true if your function always returns the same values with
    /// the same arguments
    #[inline]
    pub fn set_is_const(&self, v: bool) {
        // SAFETY: unsafe when called from different threads, but we are `!Sync`
        unsafe { (*self.0.get()).decimals = v.into() };
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{align_of, size_of};

    use super::*;
    use crate::{Init, Process};

    // Verify no size issues
    #[test]
    fn cfg_init_size() {
        assert_eq!(
            size_of::<UDF_INIT>(),
            size_of::<UdfCfg<Init>>(),
            concat!("Size of: ", stringify!(UDF_INIT))
        );
        assert_eq!(
            align_of::<UDF_INIT>(),
            align_of::<UdfCfg<Init>>(),
            concat!("Alignment of ", stringify!(UDF_INIT))
        );
    }

    #[test]
    fn cfg_proc_size() {
        assert_eq!(
            size_of::<UDF_INIT>(),
            size_of::<UdfCfg<Process>>(),
            concat!("Size of: ", stringify!(UDF_INIT))
        );
        assert_eq!(
            align_of::<UDF_INIT>(),
            align_of::<UdfCfg<Process>>(),
            concat!("Alignment of ", stringify!(UDF_INIT))
        );
    }
}
