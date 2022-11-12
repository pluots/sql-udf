//! Functions designed to safely wrap rust definitions within C bindings
//!
//! This file ties together C types and rust types, providing a safe wrapper.
//! Functions in this module are generally not meant to be used directly.

use std::ffi::{c_char, c_double, c_longlong, c_uchar, c_ulong};
use std::num::NonZeroU8;
use std::panic::{self, AssertUnwindSafe};
use std::ptr;

use udf_sys::{UDF_ARGS, UDF_INIT};

use crate::wrapper::write_msg_to_buf;
use crate::{AggregateUdf, ArgList, BasicUdf, Process, UdfCfg, MYSQL_ERRMSG_SIZE};

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
pub unsafe fn wrap_init<T: BasicUdf>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    message: *mut c_char,
) -> bool {
    // ret holds our return type, we need to tell the compiler it is safe across
    // unwind boundaries
    let mut ret = false;
    let mut ret_wrap = AssertUnwindSafe(&mut ret);

    // Unwinding into C is UB so we need to catch potential panics at the FFI
    // boundary Note to possible code readers: `panic::catch_unwind` should NOT
    // be used anywhere except the FFI boundary
    panic::catch_unwind(move || {
        let cfg = UdfCfg::from_raw_ptr(initid);
        let arglist = ArgList::from_arg_ptr(args);

        // Call the user's init function
        // If initialization succeeds, put our UDF info struct on the heap
        // If initialization fails, copy a message to the buffer

        let boxed_struct: Box<T> = match T::init(cfg, arglist) {
            Ok(v) => Box::new(v),
            Err(e) => {
                // SAFETY: buffer size is correct
                write_msg_to_buf::<MYSQL_ERRMSG_SIZE>(e.as_bytes(), message);
                **ret_wrap = true;
                return;
            }
        };

        // Set the `initid` struct to contain our struct
        // SAFETY: Must be cleaned up in deinit function, or we will leak!
        cfg.store_box(boxed_struct);
    })
    .unwrap_or_else(|_| ret = true);

    ret
}

/// For our deinit function, all we need to do is take ownership of the boxed
/// value on the stack. The function ends, it goes out of scope and gets
/// dropped.
///
/// There is no specific wrapped function here
#[inline]
pub unsafe fn wrap_deinit<T: BasicUdf>(initid: *const UDF_INIT) {
    // SAFETY: we constructed this box so it is formatted correctly
    // caller ensures validity of initid
    let cfg: &UdfCfg<Process> = UdfCfg::from_raw_ptr(initid);
    let cfg_wrap = AssertUnwindSafe(cfg);
    panic::catch_unwind(|| *cfg_wrap.retrieve_box::<T>()).ok();
}

// NOTE: the below sections are super redundant and ugly, we will aim to clean
// them up with a macro or some other architecture

unsafe fn process_return<T: Default, E>(res: Result<T, E>, error: *mut c_uchar) -> T {
    let Ok(val) = res else {
        *error = 1;
        return T::default();
    };
    val
}

unsafe fn process_return_null<T: Default, E>(
    res: Result<Option<T>, E>,
    error: *mut c_uchar,
    is_null: *mut c_uchar,
) -> T {
    match res {
        Ok(Some(v)) => v,
        Ok(None) => {
            *is_null = 1;
            T::default()
        }
        Err(_) => {
            *error = 1;
            T::default()
        }
    }
}

#[inline]
pub unsafe fn wrap_process_int<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    _is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> c_longlong
where
    for<'a> T: BasicUdf<Returns<'a> = i64>,
{
    // SAFETY: caller guarantees validity
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_arg_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let res = T::process(&mut b, cfg, arglist, err);
    cfg.store_box(b);

    process_return(res, error)
}

#[inline]
pub unsafe fn wrap_process_int_null<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> c_longlong
where
    for<'a> T: BasicUdf<Returns<'a> = Option<i64>>,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_arg_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let res = T::process(&mut b, cfg, arglist, err);
    cfg.store_box(b);

    process_return_null(res, error, is_null)
}

#[inline]
pub unsafe fn wrap_process_float<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    _is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> c_double
where
    for<'a> T: BasicUdf<Returns<'a> = f64>,
{
    // SAFETY: caller guarantees validity
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_arg_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let res = T::process(&mut b, cfg, arglist, err);
    cfg.store_box(b);

    process_return(res, error)
}

#[inline]
pub unsafe fn wrap_process_float_null<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> c_double
where
    for<'a> T: BasicUdf<Returns<'a> = Option<f64>>,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_arg_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let res = T::process(&mut b, cfg, arglist, err);
    cfg.store_box(b);

    process_return_null(res, error, is_null)
}

#[inline]
pub unsafe fn wrap_process_buf_ref<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    result: *mut c_char,
    length: *mut c_ulong,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> *const c_char
where
    T: BasicUdf,
    for<'a> T::Returns<'a>: AsRef<[u8]>,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_arg_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let proc_res = T::process(&mut b, cfg, arglist, err);

    let ret: *const c_char;
    if let Ok(ref s) = proc_res {
        // Cast u8 to c_char (u8/i8)
        let s_ref: &[u8] = s.as_ref();
        let s_ptr = s_ref.as_ptr().cast::<c_char>();
        *is_null = c_uchar::from(false);

        // If we fit within the buffer, just copy our output. Otherwise,
        // return the pointer to s.
        let res_ptr: *const c_char = if s_ref.len() as u64 <= *length {
            ptr::copy(s_ptr, result, s_ref.len());
            result
        } else {
            s_ptr
        };

        *length = s_ref.len() as u64;
        ret = res_ptr;
    } else {
        *error = 1;
        *length = 0;
        ret = result;
    };

    std::mem::forget(proc_res);
    cfg.store_box(b);

    // let ret = result;

    // Need to get the pointer after, since the reference is in `b`.

    ret
}

#[inline]
pub unsafe fn wrap_process_buf_ref_null<T, S>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    result: *mut c_char,
    length: *mut c_ulong,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> *const c_char
where
    for<'a> T: BasicUdf<Returns<'a> = Option<S>>,
    S: AsRef<[u8]>, // for<'a> T::Returns<'a>: AsRef<[u8]>,
                    // for<'a> T: BasicUdf<Returns<'a> = Option<f64>>,1
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_arg_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let res = T::process(&mut b, cfg, arglist, err);
    cfg.store_box(b);

    let Ok(res_ok) = res else {
        *error = 1;
        *length = 0;
        return result;
    };

    // Result is an Ok(); set null as needed
    let Some(s) = res_ok else {
        *is_null = 1;
        *length = 0;
        return result;
    };

    // Cast u8 to c_char (u8/i8)
    let s_ref = s.as_ref();
    let s_ptr = s_ref.as_ptr().cast::<c_char>();
    *is_null = c_uchar::from(false);

    // If we fit within the buffer, just copy our output. Otherwise,
    // return the pointer to s.
    let res_ptr = if s_ref.len() as u64 <= *length {
        ptr::copy(s_ptr, result, s_ref.len());
        result
    } else {
        s_ptr
    };
    *length = s_ref.len() as u64;

    res_ptr
}

#[inline]
pub unsafe fn wrap_process_buf<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    result: *mut c_char,
    length: *mut c_ulong,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> *const c_char
where
    T: BasicUdf,
    for<'a> T::Returns<'a>: AsRef<[u8]>,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_arg_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let proc_res = T::process(&mut b, cfg, arglist, err);

    let ret: *const c_char;
    if let Ok(ref s) = proc_res {
        // Cast u8 to c_char (u8/i8)
        let s_ref: &[u8] = s.as_ref();
        let s_ptr = s_ref.as_ptr().cast::<c_char>();
        *is_null = c_uchar::from(false);

        // If we fit within the buffer, just copy our output. Otherwise,
        // return the pointer to s.
        let res_ptr: *const c_char = if s_ref.len() as u64 <= *length {
            ptr::copy(s_ptr, result, s_ref.len());
            result
        } else {
            s_ptr
        };

        *length = s_ref.len() as u64;
        ret = res_ptr;
    } else {
        *error = 1;
        *length = 0;
        ret = result;
    };

    std::mem::forget(proc_res);
    cfg.store_box(b);

    // let ret = result;

    // Need to get the pointer after, since the reference is in `b`.

    ret
}

#[inline]
pub unsafe fn wrap_process_buf_null<T, S>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    result: *mut c_char,
    length: *mut c_ulong,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> *const c_char
where
    for<'a> T: BasicUdf<Returns<'a> = Option<S>>,
    S: AsRef<[u8]>, // for<'a> T::Returns<'a>: AsRef<[u8]>,
                    // for<'a> T: BasicUdf<Returns<'a> = Option<f64>>,1
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_arg_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let res = T::process(&mut b, cfg, arglist, err);
    cfg.store_box(b);

    let Ok(res_ok) = res else {
        *error = 1;
        *length = 0;
        return result;
    };

    // Result is an Ok(); set null as needed
    let Some(s) = res_ok else {
        *is_null = 1;
        *length = 0;
        return result;
    };

    // Cast u8 to c_char (u8/i8)
    let s_ref = s.as_ref();
    let s_ptr = s_ref.as_ptr().cast::<c_char>();
    *is_null = c_uchar::from(false);

    // If we fit within the buffer, just copy our output. Otherwise,
    // return the pointer to s.
    let res_ptr = if s_ref.len() as u64 <= *length {
        ptr::copy(s_ptr, result, s_ref.len());
        result
    } else {
        s_ptr
    };
    *length = s_ref.len() as u64;

    res_ptr
}

#[inline]
pub unsafe fn wrap_add<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    _is_null: *mut c_uchar,
    error: *mut c_uchar,
) where
    T: AggregateUdf,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_arg_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let res = T::add(&mut b, cfg, arglist, err);
    cfg.store_box(b);

    if let Err(e) = res {
        *error = e.into();
    }
}

#[inline]
pub unsafe fn wrap_clear<T>(initid: *mut UDF_INIT, _is_null: *mut c_uchar, error: *mut c_uchar)
where
    T: AggregateUdf,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let res = T::clear(&mut b, cfg, err);
    cfg.store_box(b);

    if let Err(e) = res {
        *error = e.into();
    }
}

#[inline]
pub unsafe fn wrap_remove<T>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    _is_null: *mut c_uchar,
    error: *mut c_uchar,
) where
    T: AggregateUdf,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_arg_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let res = T::remove(&mut b, cfg, arglist, err);
    cfg.store_box(b);

    if let Err(e) = res {
        *error = e.into();
    }
}
