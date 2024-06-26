//! Bindings to C for SQL UDF-related types
//!
//! Types in this module were autogenerated. Documentation mostly comes from the
//! C header file, but some clarifications were added. Some mut -> const changes
//! were done as makes sense.
//!
//! To regenerate this file, run:
//!
//! ```sh
//! bindgen udf_registration_types.c \
//!     --default-enum-style=rust_non_exhaustive \
//!     --no-derive-copy
//! ```
//!
//! _You're off the edge of the map, mate. Here there be monsters!_

/* automatically generated by rust-bindgen 0.60.1 */

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

/// C builtin
pub const true_: u32 = 1;

/// C builtin
pub const false_: u32 = 0;

/// C builtin
pub const __bool_true_false_are_defined: u32 = 1;

/// Type of the user defined function return slot and arguments
// This is `repr(C)` to ensure it is represented the same as C enums.
#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Item_result {
    /// Invalid value (not valid for UDFs)
    INVALID_RESULT = -1,

    /// Value representing a string (char *)
    STRING_RESULT = 0,

    /// Value representing a real (double)
    REAL_RESULT = 1,

    /// Value representing an int (long long)
    INT_RESULT = 2,

    /// Value representing a row (not valid for UDFs)
    ROW_RESULT = 3,

    /// Value representing a decimal (char *)
    DECIMAL_RESULT = 4,
}

impl TryFrom<i32> for Item_result {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::INVALID_RESULT as i32 => Ok(Self::INVALID_RESULT),
            x if x == Self::STRING_RESULT as i32 => Ok(Self::STRING_RESULT),
            x if x == Self::REAL_RESULT as i32 => Ok(Self::REAL_RESULT),
            x if x == Self::INT_RESULT as i32 => Ok(Self::INT_RESULT),
            x if x == Self::ROW_RESULT as i32 => Ok(Self::ROW_RESULT),
            x if x == Self::DECIMAL_RESULT as i32 => Ok(Self::DECIMAL_RESULT),
            _ => Err(format!("invalid arg type {value} received")),
        }
    }
}

/// Representation of a sequence of SQL arguments
#[repr(C)]
#[derive(Debug, Clone)]
pub struct UDF_ARGS {
    /// Number of arguments present
    pub arg_count: ::std::ffi::c_uint,

    /// Buffer of `item_result` pointers that indicate argument type
    ///
    /// Remains mutable because it can be set in `xxx_init`
    pub arg_types: *mut Item_result,

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

/// Information about the result of a user defined function
#[repr(C)]
#[derive(Debug, Clone)]
pub struct UDF_INIT {
    /// True if the function can return NULL
    pub maybe_null: bool,

    /// This is used for real-returning functions
    pub decimals: ::std::ffi::c_uint,

    /// This is used for string functions
    pub max_length: ::std::ffi::c_ulong,

    /// free pointer for function data
    pub ptr: *mut ::std::ffi::c_char,

    /// True if function always returns the same value
    pub const_item: bool,

    /// Unused at this time
    pub extension: *mut ::std::ffi::c_void,
}

/// A UDF function type indicator, currently unused
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Item_udftype {
    UDFTYPE_FUNCTION = 1,
    UDFTYPE_AGGREGATE = 2,
}

/// Function signature of an `xxx_init(...)` function
pub type Udf_func_init = Option<
    unsafe extern "C" fn(
        initid: *mut UDF_INIT,
        args: *mut UDF_ARGS,
        message: *mut ::std::ffi::c_char,
    ) -> bool,
>;

/// Function signature of an `xxx_deinit(...)` function
pub type Udf_func_deinit = Option<unsafe extern "C" fn(arg1: *mut UDF_INIT)>;

/// Function signature of an `xxx_add(...)` aggregate function
pub type Udf_func_add = Option<
    unsafe extern "C" fn(
        initid: *mut UDF_INIT,
        args: *const UDF_ARGS,
        is_null: *mut ::std::ffi::c_uchar,
        error: *mut ::std::ffi::c_uchar,
    ),
>;

/// Function signature of an `xxx_clear(...)` aggregate function
pub type Udf_func_clear = Option<
    unsafe extern "C" fn(
        initid: *mut UDF_INIT,
        is_null: *mut ::std::ffi::c_uchar,
        error: *mut ::std::ffi::c_uchar,
    ),
>;

/// Function signature of an `xxx(...)` function returning a SQL real
pub type Udf_func_double = Option<
    unsafe extern "C" fn(
        initid: *mut UDF_INIT,
        args: *const UDF_ARGS,
        is_null: *mut ::std::ffi::c_uchar,
        error: *mut ::std::ffi::c_uchar,
    ) -> ::std::ffi::c_double,
>;

/// Function signature of an `xxx(...)` function returning a SQL int
pub type Udf_func_longlong = Option<
    unsafe extern "C" fn(
        initid: *mut UDF_INIT,
        args: *const UDF_ARGS,
        is_null: *mut ::std::ffi::c_uchar,
        error: *mut ::std::ffi::c_uchar,
    ) -> ::std::ffi::c_longlong,
>;

/// Function signature of an `xxx(...)` function returning a SQL string
pub type Udf_func_string = Option<
    unsafe extern "C" fn(
        initid: *mut UDF_INIT,
        args: *const UDF_ARGS,
        result: *mut ::std::ffi::c_char,
        length: *mut ::std::ffi::c_ulong,
        is_null: *mut ::std::ffi::c_uchar,
        error: *mut ::std::ffi::c_uchar,
    ) -> *mut ::std::ffi::c_char,
>;

/// Function signature of a void functin (unused)
pub type Udf_func_any = Option<unsafe extern "C" fn()>;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn bindgen_test_layout_UDF_ARGS() {
        assert_eq!(
            ::std::mem::size_of::<UDF_ARGS>(),
            64usize,
            concat!("Size of: ", stringify!(UDF_ARGS))
        );
        assert_eq!(
            ::std::mem::align_of::<UDF_ARGS>(),
            8usize,
            concat!("Alignment of ", stringify!(UDF_ARGS))
        );
        fn test_field_arg_count() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_ARGS>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).arg_count) as usize - ptr as usize
                },
                0usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_ARGS),
                    "::",
                    stringify!(arg_count)
                )
            );
        }
        test_field_arg_count();
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
        test_field_arg_type();
        fn test_field_args() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_ARGS>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).args) as usize - ptr as usize
                },
                16usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_ARGS),
                    "::",
                    stringify!(args)
                )
            );
        }
        test_field_args();
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
        test_field_lengths();
        fn test_field_maybe_null() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_ARGS>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).maybe_null) as usize - ptr as usize
                },
                32usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_ARGS),
                    "::",
                    stringify!(maybe_null)
                )
            );
        }
        test_field_maybe_null();
        fn test_field_attributes() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_ARGS>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).attributes) as usize - ptr as usize
                },
                40usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_ARGS),
                    "::",
                    stringify!(attributes)
                )
            );
        }
        test_field_attributes();
        fn test_field_attribute_lengths() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_ARGS>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).attribute_lengths) as usize - ptr as usize
                },
                48usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_ARGS),
                    "::",
                    stringify!(attribute_lengths)
                )
            );
        }
        test_field_attribute_lengths();
        fn test_field_extension() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_ARGS>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).extension) as usize - ptr as usize
                },
                56usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_ARGS),
                    "::",
                    stringify!(extension)
                )
            );
        }
        test_field_extension();
    }

    #[test]
    fn bindgen_test_layout_UDF_INIT() {
        assert_eq!(
            ::std::mem::size_of::<UDF_INIT>(),
            40usize,
            concat!("Size of: ", stringify!(UDF_INIT))
        );
        assert_eq!(
            ::std::mem::align_of::<UDF_INIT>(),
            8usize,
            concat!("Alignment of ", stringify!(UDF_INIT))
        );
        fn test_field_maybe_null() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_INIT>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).maybe_null) as usize - ptr as usize
                },
                0usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_INIT),
                    "::",
                    stringify!(maybe_null)
                )
            );
        }
        test_field_maybe_null();
        fn test_field_decimals() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_INIT>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).decimals) as usize - ptr as usize
                },
                4usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_INIT),
                    "::",
                    stringify!(decimals)
                )
            );
        }
        test_field_decimals();
        fn test_field_max_length() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_INIT>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).max_length) as usize - ptr as usize
                },
                8usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_INIT),
                    "::",
                    stringify!(max_length)
                )
            );
        }
        test_field_max_length();
        fn test_field_ptr() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_INIT>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).ptr) as usize - ptr as usize
                },
                16usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_INIT),
                    "::",
                    stringify!(ptr)
                )
            );
        }
        test_field_ptr();
        fn test_field_const_item() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_INIT>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).const_item) as usize - ptr as usize
                },
                24usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_INIT),
                    "::",
                    stringify!(const_item)
                )
            );
        }
        test_field_const_item();
        fn test_field_extension() {
            assert_eq!(
                unsafe {
                    let uninit = ::std::mem::MaybeUninit::<UDF_INIT>::uninit();
                    let ptr = uninit.as_ptr();
                    ::std::ptr::addr_of!((*ptr).extension) as usize - ptr as usize
                },
                32usize,
                concat!(
                    "Offset of field: ",
                    stringify!(UDF_INIT),
                    "::",
                    stringify!(extension)
                )
            );
        }
        test_field_extension();
    }
}
