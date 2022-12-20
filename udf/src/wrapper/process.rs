//! Functions related to strictly the `process` UDF components

#![allow(clippy::module_name_repetitions)]
#![allow(clippy::option_if_let_else)]

use std::ffi::{c_char, c_uchar, c_ulong};
use std::num::NonZeroU8;
use std::ptr;

use udf_sys::{UDF_ARGS, UDF_INIT};

#[cfg(feature = "logging-debug")]
use super::debug;
use super::functions::UdfConverter;
use super::helpers::{buf_result_callback, BufOptions};
use crate::{ArgList, BasicUdf, ProcessError, UdfCfg};

/// Callback for properly unwrapping and setting values for `Option<T>`
///
/// Returns `None` if the value is `Err` or `None`, `Some` otherwise
#[inline]
unsafe fn ret_callback_option<R>(
    res: Result<Option<R>, ProcessError>,
    error: *mut c_uchar,
    is_null: *mut c_uchar,
) -> Option<R> {
    let transposed = res.transpose();

    // Perform action for if internal is `None`
    let Some(res_some) = transposed else {
        // We have a None result
        *is_null = 1;
        return None;
    };

    // Rest of the behavior is in `ret_callback`
    ret_callback(res_some, error, is_null)
}

/// Callback for properly unwrapping and setting values for any `T`
///
/// Returns `None` if the value is `Err`, `Some` otherwise
#[inline]
unsafe fn ret_callback<R>(
    res: Result<R, ProcessError>,
    error: *mut c_uchar,
    is_null: *mut c_uchar,
) -> Option<R> {
    // Error case: set an error, and set length to 0 if applicable
    let Ok(val) = res else {
        *error = 1;
        return None;
    };

    // Ok case: just return the desired value
    *is_null = c_uchar::from(false);

    Some(val)
}

/// Apply the `process` function for any implementation returning a nonbuffer type
/// (`f64`, `i64`)
#[inline]
#[allow(clippy::let_and_return)]
pub unsafe fn wrap_process_basic<W, U, R>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> R
where
    W: UdfConverter<U>,
    for<'a> U: BasicUdf<Returns<'a> = R>,
    R: Default,
{
    #[cfg(feature = "logging-debug")]
    debug::pre_process_call::<U>(initid, args, is_null, error);

    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);
    let mut b = cfg.retrieve_box::<W>();
    let err = *(error as *const Option<NonZeroU8>);
    let proc_res = U::process(b.as_mut_ref(), cfg, arglist, err);
    cfg.store_box(b);

    let ret = ret_callback(proc_res, error, is_null).unwrap_or_default();

    #[cfg(feature = "logging-debug")]
    debug::post_process_call::<U>(initid, args, is_null, error);

    ret
}

/// Apply the `process` function for any implementation returning an optional
/// nonbuffer type (`Option<f64>`, `Option<i64>`)
#[inline]
#[allow(clippy::let_and_return)]
pub unsafe fn wrap_process_basic_option<W, U, R>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> R
where
    W: UdfConverter<U>,
    for<'a> U: BasicUdf<Returns<'a> = Option<R>>,
    R: Default,
{
    #[cfg(feature = "logging-debug")]
    debug::pre_process_call::<U>(initid, args, is_null, error);

    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);
    let mut b = cfg.retrieve_box::<W>();
    let err = *(error as *const Option<NonZeroU8>);
    let proc_res = U::process(b.as_mut_ref(), cfg, arglist, err);
    cfg.store_box(b);

    let ret = ret_callback_option(proc_res, error, is_null).unwrap_or_default();

    #[cfg(feature = "logging-debug")]
    debug::post_process_call::<U>(initid, args, is_null, error);

    ret
}

/// Apply the `process` function for any implementation returning a buffer type
/// (`String`, `Vec<u8>`, `str`, `[u8]`)
#[inline]
pub unsafe fn wrap_process_buf<W, U>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    result: *mut c_char,
    length: *mut c_ulong,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
    can_return_ref: bool,
) -> *const c_char
where
    W: UdfConverter<U>,
    for<'b> U: BasicUdf,
    for<'a> <U as BasicUdf>::Returns<'a>: AsRef<[u8]>,
{
    #[cfg(feature = "logging-debug")]
    debug::pre_process_call_buf::<U>(initid, args, result, length, is_null, error);

    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);
    let mut b = cfg.retrieve_box::<W>();
    let err = *(error as *const Option<NonZeroU8>);
    let binding = b.as_mut_ref();
    let proc_res = U::process(binding, cfg, arglist, err);
    let buf_opts = BufOptions::new(result, length, can_return_ref);

    let post_effects_val = ret_callback(proc_res, error, is_null);

    let ret = match post_effects_val {
        Some(ref v) => buf_result_callback::<U, _>(v, &buf_opts).unwrap_or_else(|| {
            *error = 1;
            ptr::null()
        }),
        None => ptr::null(),
    };

    std::mem::forget(post_effects_val);
    cfg.store_box(b);

    #[cfg(feature = "logging-debug")]
    debug::post_process_call_buf::<U>(initid, args, result, length, is_null, error, ret);

    ret
}

/// Apply the `process` function for any implementation returning a buffer type
/// (`Option<String>`, `Option<Vec<u8>>`, `Option<str>`, `Option<[u8]>`)
#[inline]
pub unsafe fn wrap_process_buf_option<W, U, B>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    result: *mut c_char,
    length: *mut c_ulong,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
    can_return_ref: bool,
) -> *const c_char
where
    W: UdfConverter<U>,
    for<'a> U: BasicUdf<Returns<'a> = Option<B>>,
    B: AsRef<[u8]>,
{
    #[cfg(feature = "logging-debug")]
    debug::pre_process_call_buf::<U>(initid, args, result, length, is_null, error);

    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);
    let err = *(error as *const Option<NonZeroU8>);
    let mut b = cfg.retrieve_box::<W>();
    let proc_res = U::process(b.as_mut_ref(), cfg, arglist, err);
    let buf_opts = BufOptions::new(result, length, can_return_ref);

    let post_effects_val = ret_callback_option(proc_res, error, is_null);

    let ret = match post_effects_val {
        Some(ref v) => {
            if let Some(x) = buf_result_callback::<U, _>(v, &buf_opts) {
                x
            } else {
                *error = 1;
                ptr::null()
            }
        }
        None => ptr::null(),
    };

    std::mem::forget(post_effects_val);
    cfg.store_box(b);

    #[cfg(feature = "logging-debug")]
    debug::post_process_call_buf::<U>(initid, args, result, length, is_null, error, ret);

    ret
}
