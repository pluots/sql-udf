//! Functions related to strictly the `process` UDF components

#![allow(clippy::module_name_repetitions)]
#![allow(clippy::option_if_let_else)]

use std::ffi::{c_char, c_uchar, c_ulong};
use std::num::NonZeroU8;
use std::ptr;

use udf_sys::{UDF_ARGS, UDF_INIT};

use super::helpers::{buf_result_callback, BufOptions};
use crate::{ArgList, BasicUdf, ProcessError, UdfCfg};

/// Callback for properly unwrapping and setting values for `Option<T>`
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
pub unsafe fn wrap_process_basic<U, R>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> R
where
    for<'a> U: BasicUdf<Returns<'a> = R>,
    R: Default,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let proc_res = U::process(&mut b, cfg, arglist, err);
    cfg.store_box(b);

    ret_callback(proc_res, error, is_null).unwrap_or_default()
}

/// Apply the `process` function for any implementation returning an optional
/// nonbuffer type (`Option<f64>`, `Option<i64>`)
#[inline]
pub unsafe fn wrap_process_basic_option<U, R>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
) -> R
where
    for<'a> U: BasicUdf<Returns<'a> = Option<R>>,
    R: Default,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let proc_res = U::process(&mut b, cfg, arglist, err);
    cfg.store_box(b);

    ret_callback_option(proc_res, error, is_null).unwrap_or_default()
}

/// Apply the `process` function for any implementation returning a buffer type
/// (`String`, `Vec<u8>`, `str`, `[u8]`)
#[inline]
pub unsafe fn wrap_process_buf<U>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    result: *mut c_char,
    length: *mut c_ulong,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
    can_return_ref: bool,
) -> *const c_char
where
    for<'b> U: BasicUdf,
    for<'a> <U as BasicUdf>::Returns<'a>: AsRef<[u8]>,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);
    let mut b = cfg.retrieve_box();
    let err = *(error as *const Option<NonZeroU8>);
    let proc_res = U::process(&mut b, cfg, arglist, err);
    let buf_opts = BufOptions::new(result, length, can_return_ref);

    let post_effects_val = ret_callback(proc_res, error, is_null);

    let ret = match post_effects_val {
        Some(ref v) => buf_result_callback(v, &buf_opts),
        None => ptr::null(),
    };

    std::mem::forget(post_effects_val);
    cfg.store_box(b);

    ret
}

/// Apply the `process` function for any implementation returning a buffer type
/// (`Option<String>`, `Option<Vec<u8>>`, `Option<str>`, `Option<[u8]>`)
#[inline]
pub unsafe fn wrap_process_buf_option<U, B>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    result: *mut c_char,
    length: *mut c_ulong,
    is_null: *mut c_uchar,
    error: *mut c_uchar,
    can_return_ref: bool,
) -> *const c_char
where
    for<'a> U: BasicUdf<Returns<'a> = Option<B>>,
    B: AsRef<[u8]>,
{
    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);
    let err = *(error as *const Option<NonZeroU8>);
    let mut b = cfg.retrieve_box();
    let proc_res = U::process(&mut b, cfg, arglist, err);
    let buf_opts = BufOptions::new(result, length, can_return_ref);

    let post_effects_val = ret_callback_option(proc_res, error, is_null);

    let ret = match post_effects_val {
        Some(ref v) => buf_result_callback(v, &buf_opts),
        None => ptr::null(),
    };

    std::mem::forget(post_effects_val);
    cfg.store_box(b);

    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ExampleInt;
    struct ExampleIntOpt;
    struct ExampleBuf;
    struct ExampleBufOpt;

    impl BasicUdf for ExampleInt {
        type Returns<'a> = i64;

        fn init(_cfg: &UdfCfg<crate::Init>, _args: &ArgList<crate::Init>) -> Result<Self, String> {
            todo!()
        }

        fn process<'a>(
            &'a mut self,
            _cfg: &UdfCfg<crate::Process>,
            _args: &ArgList<crate::Process>,
            _error: Option<NonZeroU8>,
        ) -> Result<Self::Returns<'a>, ProcessError> {
            todo!()
        }
    }
    impl BasicUdf for ExampleIntOpt {
        type Returns<'a> = Option<i64>;

        fn init(_cfg: &UdfCfg<crate::Init>, _args: &ArgList<crate::Init>) -> Result<Self, String> {
            todo!()
        }

        fn process<'a>(
            &'a mut self,
            _cfg: &UdfCfg<crate::Process>,
            _args: &ArgList<crate::Process>,
            _error: Option<NonZeroU8>,
        ) -> Result<Self::Returns<'a>, ProcessError> {
            todo!()
        }
    }

    impl BasicUdf for ExampleBuf {
        type Returns<'a> = &'a str;

        fn init(_cfg: &UdfCfg<crate::Init>, _args: &ArgList<crate::Init>) -> Result<Self, String> {
            todo!()
        }

        fn process<'a>(
            &'a mut self,
            _cfg: &UdfCfg<crate::Process>,
            _args: &ArgList<crate::Process>,
            _error: Option<NonZeroU8>,
        ) -> Result<Self::Returns<'a>, ProcessError> {
            todo!()
        }
    }
    impl BasicUdf for ExampleBufOpt {
        type Returns<'a> = Option<Vec<u8>>;

        fn init(_cfg: &UdfCfg<crate::Init>, _args: &ArgList<crate::Init>) -> Result<Self, String> {
            todo!()
        }

        fn process<'a>(
            &'a mut self,
            _cfg: &UdfCfg<crate::Process>,
            _args: &ArgList<crate::Process>,
            _error: Option<NonZeroU8>,
        ) -> Result<Self::Returns<'a>, ProcessError> {
            todo!()
        }
    }

    #[test]
    #[should_panic]
    #[allow(unreachable_code)]
    #[allow(clippy::diverging_sub_expression)]
    fn test_fn_sig() {
        // Just validate our function signatures with compile tests

        unsafe {
            wrap_process_basic::<ExampleInt, _>(todo!(), todo!(), todo!(), todo!());
            wrap_process_basic_option::<ExampleIntOpt, _>(todo!(), todo!(), todo!(), todo!());
            wrap_process_buf::<ExampleBuf>(
                todo!(),
                todo!(),
                todo!(),
                todo!(),
                todo!(),
                todo!(),
                todo!(),
            );
            wrap_process_buf_option::<ExampleBufOpt, _>(
                todo!(),
                todo!(),
                todo!(),
                todo!(),
                todo!(),
                todo!(),
                todo!(),
            );
        }
    }
}
