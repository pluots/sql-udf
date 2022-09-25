//! esigned to safely wrap rust definitions within C bindings
//!
//! This file ties together C types and rust types, providing a safe wrapper.
//! Functions in this module are generally not meant to be used directly.

#![allow(dead_code)]

use std::ffi::CString;
use std::os::raw::{c_char, c_longlong, c_uchar, c_ulong};
use std::{ptr, slice, str};

use mysqlclient_sys::MYSQL_ERRMSG_SIZE;

use crate::ffi::bindings::{Item_result, UDF_ARGS, UDF_INIT};
use crate::ffi::wrapper_impl::{process_args, write_msg_to_buf};
use crate::{BasicUdf, Init, Process, SqlArg};

const ERRMSG_SIZE: usize = MYSQL_ERRMSG_SIZE as usize;
const PROC_ARG_ERRMSG: &str = "error processing arguments";

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
/// - To specify the maximum length of the result (handled by proc macro)
/// - To specify (for REAL functions) the maximum number of decimal places in
///   the result. (handled by proc macro)
/// - To specify whether the result can be NULL. (handled by proc macro based on
///   `Returns`)
#[inline]
pub unsafe fn init_wrapper<T: BasicUdf>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    message: *mut c_char,
) -> bool {
    // Attempt to process arguments
    let processed_args: Vec<SqlArg<Init>> = match unsafe { process_args(args) } {
        Ok(v) => v,
        Err(e) => {
            // Safety: buffer size is correct
            unsafe { write_msg_to_buf::<ERRMSG_SIZE>(&e, message) };
            return true;
        }
    };

    // If initialization succeeds, put our UDF info struct on the heap
    // If initialization fails, copy a message to the buffer
    let boxed_struct = match T::init(&processed_args) {
        Ok(v) => Box::new(v),
        Err(e) => {
            // Safety: buffer size is correct
            unsafe { write_msg_to_buf::<ERRMSG_SIZE>(&e, message) };
            return true;
        }
    };

    // Set the `initid` struct to contain our struct
    // Safety: Must be cleaned up in deinit function, or we will leak!
    unsafe { box_to_initid_ptr(initid, boxed_struct) };

    // Everything OK; return false
    false
}

#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn box_from_initid_ptr<T>(initid: *const UDF_INIT) -> Box<T> {
    Box::from_raw((*initid).ptr as *mut T)
}

/// Turn the box into a pointer and set `*initid.ptr`
#[inline]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn box_to_initid_ptr<T>(initid: *mut UDF_INIT, b: Box<T>) {
    let box_ptr = Box::into_raw(b);
    (*initid).ptr = box_ptr as *mut c_char;
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
    drop(Box::from_raw((*initid).ptr as *mut T));
}

trait IntWrap {
    unsafe fn process_wrapper<T>(
        initid: *mut UDF_INIT,
        args: *mut UDF_ARGS,
        is_null: *mut c_char,
        error: *mut c_char,
    ) -> c_longlong;
}

impl<'a, X> IntWrap for X
where
    X: BasicUdf<Returns<'a> = i32> + 'a,
{
    unsafe fn process_wrapper<T>(
        initid: *mut UDF_INIT,
        args: *mut UDF_ARGS,
        is_null: *mut c_char,
        error: *mut c_char,
    ) -> c_longlong {
        10
    }
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
    let proc_args = process_args(args).expect(PROC_ARG_ERRMSG);
    let mut b = box_from_initid_ptr(initid);
    let res = T::process(&mut b, proc_args.as_slice());
    box_to_initid_ptr(initid, b);

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
    let proc_args = process_args(args).expect(PROC_ARG_ERRMSG);
    let mut b = box_from_initid_ptr(initid);
    let res = T::process(&mut b, proc_args.as_slice());
    box_to_initid_ptr(initid, b);

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
