//! Rust representation of `UDF_INIT`

#![allow(clippy::useless_conversion, clippy::unnecessary_cast)]

use std::cell::Cell;
use std::ffi::{c_char, c_uint, c_ulong, c_void};
use std::marker::PhantomData;

use udf_sys::UDF_INIT;

use crate::{Init, UdfState};

/// Helpful constants related to the `max_length` parameter
///
/// These can be helpful when calling [`set_max_len()`]
#[non_exhaustive]
#[repr(u32)]
pub enum MaxLenOptions {
    /// The default max length for integers is 21
    IntDefault = 21,

    /// The default max length of a real value is 13 plus the result of
    /// [`get_decimals()`]
    RealBase = 13,

    /// A `blob` can be up to 65 KiB.
    Blob = 1 << 16,

    /// A `mediumblob` can be up to 16 MiB.
    MediumBlob = 1 << 24,
}

/// A collection of SQL arguments
///
/// This is rusty wrapper around SQL's `UDF_INIT` struct, providing methods to
/// easily work with arguments.
///
/// We really only use setters/getters here because the original struct uses
/// `ulong` which is a different size on Windows and Linux. There is
#[repr(C)]
#[derive(Debug)]
pub struct UdfCfg<S: UdfState> {
    // This is identical to UDF_INIT, see documentation there for fields
    // We just turn fields into cells for interior mutability
    pub(crate) maybe_null: Cell<bool>,
    pub(crate) decimals: Cell<c_uint>,
    pub(crate) max_length: Cell<c_ulong>,
    pub(crate) ptr: *mut c_char,
    pub(crate) const_item: Cell<bool>,
    pub(crate) extension: *const c_void,
    pub(crate) marker: PhantomData<S>,
}

impl<S: UdfState> UdfCfg<S> {
    /// Create an `ArgList` type on a `UDF_ARGS` struct
    #[inline]
    #[allow(unsafe_op_in_unsafe_fn)]
    pub(crate) unsafe fn from_init_ptr_mut<'p>(ptr: *mut UDF_INIT) -> &'p mut Self {
        unsafe { &mut *ptr.cast::<Self>() }
    }

    /// Create an `ArgList` type on a `UDF_ARGS` struct
    #[inline]
    #[allow(unsafe_op_in_unsafe_fn)]
    pub(crate) unsafe fn from_init_ptr<'p>(ptr: *const UDF_INIT) -> &'p Self {
        unsafe { &*ptr.cast::<Self>() }
    }

    /// Consume a box and store its pointer in this `UDF_INIT`
    ///
    /// This takes a boxed object, turns it into a pointer, and stores that
    /// pointer in this struct. After calling this function, [`retrieve_box`]
    /// _must_ be called to free the memory!
    pub(crate) fn store_box<T>(&mut self, b: Box<T>) {
        let box_ptr = Box::into_raw(b);
        self.ptr = box_ptr.cast::<c_char>();
    }

    /// Given this struct's `ptr` field is a boxed object, turn that pointer
    /// back into a box
    ///
    /// # Safety
    ///
    /// T _must_ be the type of this struct's pointer, likely created with
    /// [`store_box`]
    #[allow(unsafe_op_in_unsafe_fn)]
    pub(crate) unsafe fn retrieve_box<T>(&self) -> Box<T> {
        Box::from_raw(self.ptr.cast::<T>())
    }

    /// Retrieve the setting for whether this UDF may return `null`
    ///
    /// This defaults to true if any argument is nullable, false otherwise
    #[inline]
    pub fn get_maybe_null(&self) -> bool {
        self.maybe_null.get()
    }

    /// Retrieve the setting for number of decimal places
    ///
    /// This defaults to the longest number of digits of any argument, or 31 if
    /// there is no fixed number
    #[inline]
    pub fn get_decimals(&self) -> u32 {
        self.decimals.get() as u32
    }

    /// Set the number of decimals this function returns
    ///
    /// This can be changed at any point in the UDF (init or process)
    #[inline]
    pub fn set_decimals(&self, v: u32) {
        self.decimals.replace(c_uint::from(v));
    }

    /// Retrieve the current maximum length setting for this in-progress UDF
    #[inline]
    pub fn get_max_len(&self) -> u64 {
        self.max_length.get() as u64
    }

    /// Get the current `const_item` value
    #[inline]
    pub fn get_is_const(&self) -> bool {
        self.const_item.get()
    }
}

/// Implementations of actions on a `UdfCfg` that are only possible during
/// initialization
impl UdfCfg<Init> {
    /// Set whether or not this function may return null
    #[inline]
    pub fn set_maybe_null(&self, v: bool) {
        self.maybe_null.replace(v);
    }

    /// Set the maximum possible length of this UDF's result
    ///
    /// This is mostly relevant for String and Decimal return types. See
    /// [`MaxLenOptions`] for possible defaults, including `BLOB` sizes.
    #[inline]
    pub fn set_max_len(&self, v: u32) {
        self.decimals.replace(c_uint::from(v));
    }

    /// Set a new `const_item` value
    ///
    /// Set this to true if your function always returns the same values with
    /// the same arguments
    #[inline]
    pub fn set_is_const(&self, v: bool) {
        self.const_item.replace(v);
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{align_of, size_of};

    use super::*;

    // Verify no size issues
    #[test]
    fn initcfg_size() {
        assert_eq!(
            size_of::<UDF_INIT>(),
            size_of::<UdfCfg>(),
            concat!("Size of: ", stringify!(UDF_INIT))
        );
        assert_eq!(
            align_of::<UDF_INIT>(),
            align_of::<UdfCfg>(),
            concat!("Alignment of ", stringify!(UDF_ARGS))
        );
    }
}
