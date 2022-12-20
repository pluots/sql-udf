#![allow(non_camel_case_types)]

/// Representation of a sequence of SQL arguments
///
/// This should be identical to `udf_sys::UDF_ARGS` except `arg_types` is a
/// `c_int` rather than an `Item_result`. This just allows us to
#[repr(C)]
#[derive(Debug, Clone)]
pub struct UDF_ARGSx {
    /// Number of arguments present
    pub arg_count: ::std::ffi::c_uint,

    /// Buffer of item_result pointers that indicate argument type
    ///
    /// Remains mutable because it can be set in `xxx_init`
    pub arg_types: *mut ::std::ffi::c_int,

    /// Buffer of pointers to the arguments. Arguments may be of any type
    /// (specified in `arg_type`).
    pub args: *const *const ::std::ffi::c_char,

    /// Buffer of lengths for string arguments
    pub lengths: *const ::std::ffi::c_ulong,

    /// Indicates whether the argument may be null or not
    pub maybe_null: *const ::std::ffi::c_char,

    /// Buffer of string pointers that hold variable names, for use with error
    /// messages
    pub attributes: *const *const ::std::ffi::c_char,

    /// Buffer of lengths of attributes
    pub attribute_lengths: *const ::std::ffi::c_ulong,

    /// Extension is currently unused
    pub extension: *const ::std::ffi::c_void,
}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use udf_sys::UDF_ARGS;

    use super::*;

    #[test]
    fn test_layout() {
        let layout_default = Layout::new::<UDF_ARGS>();
        let layout_modded = Layout::new::<UDF_ARGSx>();
        assert_eq!(layout_default, layout_modded);
    }

    // Below couple tests are taken from bindgen
    #[test]
    fn test_field_arg_type() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<UDF_ARGS>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).arg_types) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(UDF_ARGS),
                "::",
                stringify!(arg_type)
            )
        );
    }

    #[test]
    fn test_field_lengths() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<UDF_ARGS>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).lengths) as usize - ptr as usize
            },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(UDF_ARGS),
                "::",
                stringify!(lengths)
            )
        );
    }
}
