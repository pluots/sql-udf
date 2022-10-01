//! Functions designed to safely wrap rust definitions within C bindings
//!
//! This file ties together C types and rust types, providing a safe wrapper.
//! Functions in this module are generally not meant to be used directly.

#![allow(dead_code)]

use std::ffi::{c_char, c_longlong, c_uchar, c_uint, c_ulong, CString};
use std::marker::PhantomData;
use std::ops::Index;
use std::slice::SliceIndex;
use std::{panic, ptr, slice, str};

use mysqlclient_sys::MYSQL_ERRMSG_SIZE;

use crate::ffi::bindings::{Item_result, UDF_ARGS, UDF_INIT};
use crate::ffi::wrapper_impl::write_msg_to_buf;
use crate::{ArgList, BasicUdf, Init, Process, SqlArg, SqlResult, UdfState};

const ERRMSG_SIZE: usize = MYSQL_ERRMSG_SIZE as usize;

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
///   tell MySQL to coerce arguments to the required types when the main
///   function is called. (handled by `T::init`)
/// - To allocate any memory required by the main function. (We box our struct
///   for this)
/// - To specify the maximum length of the result
/// - To specify (for REAL functions) the maximum number of decimal places in
///   the result.
/// - To specify whether the result can be NULL. (handled by proc macro based on
///   `Returns`)
#[inline]
pub unsafe fn init_wrapper<T: BasicUdf>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    message: *mut c_char,
) -> bool {
    // Safety: caller guarantees validity
    let arglist = ArgList::new(unsafe { *args });

    // Call the user's init function
    // If initialization succeeds, put our UDF info struct on the heap
    // If initialization fails, copy a message to the buffer
    let boxed_struct: Box<T> = match T::init(&arglist) {
        Ok(v) => Box::new(v),
        Err(e) => {
            // Safety: buffer size is correct
            unsafe { write_msg_to_buf::<ERRMSG_SIZE>(&e, message) };
            return true;
        }
    };

    // Set the `initid` struct to contain our struct
    // Safety: Must be cleaned up in deinit function, or we will leak!
    unsafe { (*initid).store_box(boxed_struct) };

    // Everything OK; return false
    false
}

/// For our deinit function, all we need to do is take ownership of the boxed
/// value on the stack. The function ends, it goes out of scope and gets
/// dropped.
///
/// There is no specific wrapped function here
#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn deinit_wrapper<T: BasicUdf>(initid: *const UDF_INIT) {
    // Safety: we constructed this box so it is formatted correctly
    (*initid).retrieve_box::<T>();
}

#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn process_wrapper_int<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *mut c_char,
    error: *mut c_char,
) -> c_longlong
where
    for<'a> T: BasicUdf<Returns<'a> = i64>,
{
    // Safety: caller guarantees validity
    let arglist = ArgList::new(unsafe { *args });
    let mut b = (*initid).retrieve_box();
    let res = T::process(&mut b, &arglist);
    (*initid).store_box(b);

    match res {
        Ok(v) => v,
        Err(_) => {
            *error = 1;
            0
        }
    }
}

#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn process_wrapper_nul_int<T>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    is_null: *mut c_char,
    error: *mut c_char,
) -> c_longlong
where
    for<'a> T: BasicUdf<Returns<'a> = Option<i64>>,
{
    // Safety: caller guarantees validity
    let arglist = ArgList::new(unsafe { *args });
    let mut b = (*initid).retrieve_box();
    let res = T::process(&mut b, &arglist);
    (*initid).store_box(b);

    let tmp = match res {
        Ok(v) => v,
        Err(_) => {
            *error = 1;
            return 0;
        }
    };

    match tmp {
        Some(v) => v,
        None => {
            *is_null = 1;
            0
        }
    }
}

unsafe extern "C" fn udf_func_double(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) // -> f64
{
}
unsafe extern "C" fn udf_func_longlong(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) // -> ::std::os::raw::c_longlong
{
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
