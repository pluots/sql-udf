//! Functions designed to safely wrap rust definitions within C bindings
//!
//! This file ties together C types and rust types, providing a safe wrapper.
//! Functions in this module are generally not meant to be used directly.

use std::ffi::{c_char, c_uchar};
use std::num::NonZeroU8;

use udf_sys::{UDF_ARGS, UDF_INIT};

use crate::wrapper::write_msg_to_buf;
use crate::{AggregateUdf, ArgList, BasicUdf, Process, UdfCfg, MYSQL_ERRMSG_SIZE};

/// A wrapper that lets us handle return types when the user returns an
/// allocated buffer (rather than a reference). We wrap the user's type within
/// this struct and need to be sure to negotiate correctly
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufConverter<U, B>
where
    U: BasicUdf,
    B: Default,
{
    udf: U,
    buf: B,
}

impl<U, B> BufConverter<U, B>
where
    U: BasicUdf,
    B: Default,
{
    #[allow(dead_code)]
    fn set_retval(&mut self, val: B) {
        self.buf = val;
    }
}

/// Trait to allow interfacing a buffered or plain type
pub trait UdfConverter<U> {
    fn as_mut_ref(&mut self) -> &mut U;
    fn into_storable(source: U) -> Self;
}

impl<U, B> UdfConverter<U> for BufConverter<U, B>
where
    U: BasicUdf,
    B: Default,
{
    fn as_mut_ref(&mut self) -> &mut U {
        &mut self.udf
    }

    fn into_storable(source: U) -> Self {
        Self {
            udf: source,
            buf: B::default(),
        }
    }
}

impl<U: BasicUdf> UdfConverter<U> for U {
    fn as_mut_ref(&mut self) -> &mut U {
        self
    }

    fn into_storable(source: U) -> Self {
        source
    }
}

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
/// - Verify the number of arguments to XXX() (handled by `U::init`)
/// - Verify that the arguments are of a required type or, alternatively, to
///   tell `MySQL` to coerce arguments to the required types when the main
///   function is called. (handled by `U::init`)
/// - To allocate any memory required by the main function. (We box our struct
///   for this)
/// - To specify the maximum length of the result
/// - To specify (for REAL functions) the maximum number of decimal places in
///   the result.
/// - To specify whether the result can be NULL. (handled by proc macro based on
///   `Returns`)
#[inline]
pub unsafe fn wrap_init<W: UdfConverter<U>, U: BasicUdf>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    message: *mut c_char,
) -> bool {
    log_call!(enter: "init", U, args, message);

    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);

    // Call the user's init function
    let init_res = U::init(cfg, arglist);

    // Apply any pending coercions
    arglist.flush_all_coercions();

    // If initialization succeeds, put our UDF info struct on the heap
    // If initialization fails, copy a message to the buffer
    let ret = match init_res {
        Ok(v) => {
            // set the `initid` struct to contain our struct
            // SAFETY: must be cleaned up in deinit function, or we will leak!
            let boxed_struct: Box<W> = Box::new(W::into_storable(v));
            cfg.store_box(boxed_struct);
            false
        }
        Err(e) => {
            // SAFETY: buffer size is correct
            write_msg_to_buf::<MYSQL_ERRMSG_SIZE>(e.as_bytes(), message);
            true
        }
    };

    log_call!(exit: "init", U, &*args, &*message, ret);
    ret
}

/// For our deinit function, all we need to do is take ownership of the boxed
/// value on the stack. The function ends, it goes out of scope and gets
/// dropped.
///
/// There is no specific wrapped function here
#[inline]
pub unsafe fn wrap_deinit<W: UdfConverter<U>, U: BasicUdf>(initid: *const UDF_INIT) {
    log_call!(enter: "deinit", U, &*initid);

    // SAFETY: we constructed this box so it is formatted correctly
    // caller ensures validity of initid
    let cfg: &UdfCfg<Process> = UdfCfg::from_raw_ptr(initid);
    cfg.retrieve_box::<W>();
}

#[inline]
pub unsafe fn wrap_add<W: UdfConverter<U>, U: AggregateUdf>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    _is_null: *mut c_uchar,
    error: *mut c_uchar,
) {
    log_call!(enter: "add", U, &*initid, &*args, &*error);

    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);
    let err = *(error as *const Option<NonZeroU8>);
    let mut b = cfg.retrieve_box::<W>();
    let res = U::add(b.as_mut_ref(), cfg, arglist, err);
    cfg.store_box(b);

    if let Err(e) = res {
        *error = e.into();
    }
}

#[inline]
pub unsafe fn wrap_clear<W: UdfConverter<U>, U: AggregateUdf>(
    initid: *mut UDF_INIT,
    _is_null: *mut c_uchar,
    error: *mut c_uchar,
) {
    log_call!(enter: "clear", U, &*initid, &*error);

    let cfg = UdfCfg::from_raw_ptr(initid);
    let err = *(error as *const Option<NonZeroU8>);
    let mut b = cfg.retrieve_box::<W>();
    let res = U::clear(b.as_mut_ref(), cfg, err);
    cfg.store_box(b);

    if let Err(e) = res {
        *error = e.into();
    }
}

#[inline]
pub unsafe fn wrap_remove<W: UdfConverter<U>, U: AggregateUdf>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    _is_null: *mut c_uchar,
    error: *mut c_uchar,
) {
    log_call!(enter: "remove", U, &*initid, &*args, &*error);

    let cfg = UdfCfg::from_raw_ptr(initid);
    let arglist = ArgList::from_raw_ptr(args);
    let err = *(error as *const Option<NonZeroU8>);
    let mut b = cfg.retrieve_box::<W>();
    let res = U::remove(b.as_mut_ref(), cfg, arglist, err);
    cfg.store_box(b);

    if let Err(e) = res {
        *error = e.into();
    }
}
