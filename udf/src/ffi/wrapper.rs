//! Functions designed to safely wrap rust definitions within C bindings
//!
//! This file ties together C types and rust types, providing a safe wrapper.
//! Functions in this module are generally not meant to be used directly.

#![allow(dead_code)]
use std::cell::Cell;
use std::ffi::{c_char, c_double, c_longlong, c_uchar, c_uint, c_ulong, CString};
use std::marker::PhantomData;
use std::num::NonZeroU8;
use std::ops::Index;
use std::panic::{self, AssertUnwindSafe};
use std::slice::SliceIndex;
use std::{ptr, slice, str};

use crate::ffi::bindings::{Item_result, UDF_ARGS, UDF_INIT};
use crate::ffi::wrapper_impl::write_msg_to_buf;
use crate::{
    ArgList, BasicUdf, Init, InitCfg, Process, ProcessError, SqlArg, SqlResult, UdfState,
    MYSQL_ERRMSG_SIZE,
};

/// This function provides the same signature as the C FFI expects. It is used
/// to perform setup within a renamed function, and will apply it to a specific
/// type that implements `BasicUDF`.
///
/// # Arguments
///
/// - initd: Settable nformation about the return type of the function
/// - args: A list of arguments passed to theis function
///
/// # Result
///
/// Return true if there is an error, as expected by the UDF interface
///
/// # Modifies
///
/// - `initid.ptr` is set to the contained struct
///
/// # Panics
///
/// - Panics if the error message contains "\0", or if the message is too long (
///   greater than 511 bytes).
/// - Panics if the provides error message string contains null characters
///
/// # Interface
///
/// Based on the SQL UDF spec, we need to perform the following here:
/// - Verify the number of arguments to XXX() (handled by `T::init`)
/// - Verify that the arguments are of a required type or, alternatively, to
///   tell `MySQL` to coerce arguments to the required types when the main
///   function is called. (handled by `T::init`)
/// - To allocate any memory required by the main function. (We box our struct
///   for this)
/// - To specify the maximum length of the result
/// - To specify (for REAL functions) the maximum number of decimal places in
///   the result.
/// - To specify whether the result can be NULL. (handled by proc macro based on
///   `Returns`)
#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn wrap_init<T: BasicUdf>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    message: *mut c_char,
) -> bool {
    dbg!();
    // SAFETY: caller guarantees validity of args ptr
    let arglist = ArgList::new(*args);

    // ret holds our return type, we need to tell the compiler it is safe across
    // unwind boundaries
    let mut ret = false;
    let mut ret_wrap = AssertUnwindSafe(&mut ret);
    dbg!();
    // Unwinding into C is UB so we need to catch potential panics at the FFI
    // boundary Note to possible code readers: `panic::catch_unwind` should NOT
    // be used anywhere except the FFI boundary,
    panic::catch_unwind(move || {
        dbg!();
        // Call the user's init function
        // If initialization succeeds, put our UDF info struct on the heap
        // If initialization fails, copy a message to the buffer
        let mut init_cfg = InitCfg::from_ptr(initid);
        let boxed_struct: Box<T> = match T::init(&mut init_cfg, &arglist) {
            Ok(v) => Box::new(v),
            Err(e) => {
                // Safety: buffer size is correct
                write_msg_to_buf::<MYSQL_ERRMSG_SIZE>(e.as_bytes(), message);
                **ret_wrap = true;
                return;
            }
        };

        // Set the `initid` struct to contain our struct
        // Safety: Must be cleaned up in deinit function, or we will leak!
        (*initid).store_box(boxed_struct);
    })
    .unwrap_or_else(|e| ret = true);
    dbg!();

    ret
}

/// For our deinit function, all we need to do is take ownership of the boxed
/// value on the stack. The function ends, it goes out of scope and gets
/// dropped.
///
/// There is no specific wrapped function here
#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn wrap_deinit<T: BasicUdf>(initid: *const UDF_INIT) {
    // SAFETY: we constructed this box so it is formatted correctly
    // caller ensures validity of initid
    dbg!();
    panic::catch_unwind(|| (*initid).retrieve_box::<T>()).ok();
    dbg!();
}

#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn wrap_process_int<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> c_longlong
where
    for<'a> T: BasicUdf<Returns<'a> = i64>,
{
    // SAFETY: caller guarantees validity
    dbg!();
    let arglist = ArgList::new(*args);
    let mut b = (*initid).retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);

    let res = T::process(&mut b, &arglist, err);

    (*initid).store_box(b);
    dbg!();

    if let Ok(v) = res {
        dbg!();
        v
    } else {
        dbg!();
        *error = 1;
        0
    }
}

#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn wrap_process_int_null<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> c_longlong
where
    for<'a> T: BasicUdf<Returns<'a> = Option<i64>>,
{
    let arglist = ArgList::new(*args);
    let mut b = (*initid).retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);

    let res = T::process(&mut b, &arglist, err);

    (*initid).store_box(b);

    if let Ok(res_ok) = res {
        // Result is an Ok(); set null as needed
        if let Some(v) = res_ok {
            v
        } else {
            *is_null = 1;
            0
        }
    } else {
        // Result is an Err()
        *error = 1;
        0
    }
}

#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn wrap_process_float<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> c_double
where
    for<'a> T: BasicUdf<Returns<'a> = f64>,
{
    // SAFETY: caller guarantees validity
    let arglist = ArgList::new(*args);
    let mut b = (*initid).retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);

    let res = T::process(&mut b, &arglist, err);

    (*initid).store_box(b);

    if let Ok(v) = res {
        v
    } else {
        *error = 1;
        0.0
    }
}

#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn wrap_process_float_null<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> c_double
where
    for<'a> T: BasicUdf<Returns<'a> = Option<f64>>,
{
    let arglist = ArgList::new(*args);
    let mut b = (*initid).retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);

    let res = T::process(&mut b, &arglist, err);

    (*initid).store_box(b);

    if let Ok(res_ok) = res {
        // Result is an Ok(); set null as needed
        if let Some(v) = res_ok {
            v
        } else {
            *is_null = 1;
            0.0
        }
    } else {
        // Result is an Err()
        *error = 1;
        0.0
    }
}

unsafe extern "C" fn udf_func_str(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    result: *mut c_char,
    length: *mut c_ulong,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) // -> *mut c_char
{
}
