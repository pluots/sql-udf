use std::cell::Cell;
use std::ffi::{c_char, c_uint, c_ulong, c_void};
use std::fmt::Display;
use std::marker::PhantomData;

use crate::UdfState;
use crate::ffi::bindings::UDF_INIT;

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
/// We really only want to use setters/getters here because the original struct
/// uses `ulong` which is a different size on Windows and Linux
#[repr(C)]
#[derive(Debug)]
pub struct UdfCfg<S: UdfState> {
    // This is a wrapper for UDF_INIT, see documentation there for fields
    // We just turn fields into cells for interior mutability
    pub(crate) maybe_null: Cell<bool>,
    pub(crate) decimals: Cell<c_uint>,
    pub(crate) max_length: Cell<c_ulong>,
    pub(crate) ptr: *mut c_char,
    pub(crate) const_item: Cell<bool>,
    pub(crate) extension: *mut c_void,
    pub(crate) marker: PhantomData<S>
}

#[allow(clippy::useless_conversion, clippy::unnecessary_cast)]
impl<S: UdfState> UdfCfg<S> {
    /// Consume a box and store its pointer in this `UDF_INIT`
    ///
    /// After calling this function, the caller is responsible for
    /// cleaning up the
    pub(crate) fn store_box<T>(&mut self, b: Box<T>) {
        let box_ptr = Box::into_raw(b);
        self.ptr = box_ptr.cast::<c_char>();
    }

    /// Given a generic type T, assume\
    /// 
    ///
    /// Safety: T _must_ be the type of this pointer
    #[allow(unsafe_op_in_unsafe_fn)]
    pub(crate) unsafe fn retrieve_box<T>(&self) -> Box<T> {
        Box::from_raw(self.ptr.cast::<T>())
    }

    #[inline]
    pub fn get_maybe_null(&self) -> bool {
        self.maybe_null.get()
    }

    #[inline]
    pub fn set_maybe_null(&self, v: bool) {
        self.maybe_null.replace(v);
    }

    #[inline]
    pub fn get_decimals(&self) -> u32 {
        self.decimals.get() as u32
    }

    #[inline]
    pub fn set_decimals(&self, v: u32) {
        self.decimals.replace(c_uint::from(v));
    }

    #[inline]
    pub fn get_max_len(&self) -> u64 {
        self.max_length.get() as u64
    }

    #[inline]
    pub fn set_max_len(&self, v: u32) {
        self.decimals.replace(c_uint::from(v));
    }

    /// Get the current `const_item` value
    #[inline]
    pub fn get_is_const(&self) -> bool {
        self.const_item.get()
    }

    /// Set a new `const_item` value
    ///
    /// Set this to true if your function always returns the same values with
    /// the same arguments
    #[inline]
    pub fn set_is_const(&mut self, v: bool) {
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
