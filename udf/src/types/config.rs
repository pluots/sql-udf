

use std::ffi::c_uint;

use crate::ffi::bindings::UDF_INIT;
/// A collection of SQL arguments
///
/// This is rusty wrapper around SQL's `UDF_INIT` struct, providing methods to
/// easily work with arguments.
///
/// We really only want to use setters/getters here because the original struct
/// uses `ulong` which is a different size on Windows and Linux
pub struct InitCfg(UDF_INIT);

#[allow(clippy::useless_conversion, clippy::unnecessary_cast)]
impl InitCfg {
    pub(crate) unsafe fn from_ptr(ptr: *mut UDF_INIT) -> Self {
        // unsafe { Self { base: &mut *ptr } }
        unsafe { Self(*ptr) }
    }

    #[inline]
    pub fn get_maybe_null(&self) -> bool {
        self.0.maybe_null
    }

    #[inline]
    pub fn set_maybe_null(&mut self, v: bool) {
        self.0.maybe_null = v;
    }

    #[inline]
    pub fn get_decimals(&self) -> u8 {
        // Decimals has a max of 31
        self.0.decimals as u8
    }

    #[inline]
    pub fn set_decimals(&mut self, v: u8) {
        self.0.decimals = c_uint::from(v);
    }

    #[inline]
    pub fn get_max_len(&self) -> u64 {
        self.0.max_length as u64
    }

    #[inline]
    pub fn set_max_len(&mut self, v: u32) {
        self.0.decimals = c_uint::from(v);
    }

    #[inline]
    pub fn set_max_len_to_blob(&mut self) {
        // Will use blob planning if we set the magic value of 16 MB (1 << 24)
        // or 65 kB
        self.0.decimals = 1 << 24;
    }

    /// Get the current `const_item` value
    #[inline]
    pub fn get_const_item(&self) -> bool {
        self.0.const_item
    }

    /// Set a new `const_item` value
    ///
    /// Set this to true if your function always returns the same values with
    /// the same arguments
    #[inline]
    pub fn set_const_item(&mut self, v: bool) {
        self.0.const_item = v;
    }
}
