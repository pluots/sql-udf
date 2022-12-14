//! Functions designed to safely wrap rust definitions within C bindings
//!
//! This file ties together C types and rust types, providing a safe wrapper.
//! Functions in this module are generally not meant to be used directly.

use std::any::type_name;
use std::ffi::{c_char, c_uchar};
use std::num::NonZeroU8;
use std::panic::{self, AssertUnwindSafe};

use udf_sys::{UDF_ARGS, UDF_INIT};

#[cfg(feature = "logging-debug")]
use crate::wrapper::debug;
use crate::wrapper::write_msg_to_buf;
use crate::{udf_log, AggregateUdf, ArgList, BasicUdf, Process, UdfCfg, MYSQL_ERRMSG_SIZE};

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
    #[cfg(feature = "logging-debug")]
    debug::pre_init_call::<U>(initid, args, message);

    // ret holds our return type, we need to tell the compiler it is safe across
    // unwind boundaries
    let mut ret = false;
    let mut ret_wrap = AssertUnwindSafe(&mut ret);

    // Unwinding into C is UB so we need to catch potential panics at the FFI
    // boundary Note to possible code readers: `panic::catch_unwind` should NOT
    // be used anywhere except the FFI boundary
    panic::catch_unwind(move || {
        let cfg = UdfCfg::from_raw_ptr(initid);
        let arglist = ArgList::from_raw_ptr(args);

        // Call the user's init function
        // If initialization succeeds, put our UDF info struct on the heap
        // If initialization fails, copy a message to the buffer
        let boxed_struct: Box<W> = match U::init(cfg, arglist) {
            Ok(v) => Box::new(W::into_storable(v)),
            Err(e) => {
                // SAFETY: buffer size is correct
                write_msg_to_buf::<MYSQL_ERRMSG_SIZE>(e.as_bytes(), message);
                **ret_wrap = true;
                return;
            }
        };

        // Apply any pending coercions
        arglist.flush_all_coercions();

        // Set the `initid` struct to contain our struct
        // SAFETY: Must be cleaned up in deinit function, or we will leak!
        cfg.store_box(boxed_struct);
    })
    .unwrap_or_else(|_| {
        write_msg_to_buf::<MYSQL_ERRMSG_SIZE>(b"(critical) init function panicked", message);
        udf_log!(Critical: "init function panicked for `{}`", type_name::<U>());
        ret = true;
    });

    #[cfg(feature = "logging-debug")]
    debug::post_init_call::<U>(initid, args, message, ret);

    ret
}

/// For our deinit function, all we need to do is take ownership of the boxed
/// value on the stack. The function ends, it goes out of scope and gets
/// dropped.
///
/// There is no specific wrapped function here
#[inline]
pub unsafe fn wrap_deinit<W: UdfConverter<U>, U: BasicUdf>(initid: *const UDF_INIT) {
    #[cfg(feature = "logging-debug")]
    debug::pre_deinit_call::<U>(initid);

    // SAFETY: we constructed this box so it is formatted correctly
    // caller ensures validity of initid
    panic::catch_unwind(|| {
        let cfg: &UdfCfg<Process> = UdfCfg::from_raw_ptr(initid);
        cfg.retrieve_box::<W>();
    })
    .unwrap_or_else(|_| udf_log!(Critical: "deinit function panicked for `{}`", type_name::<U>()));
}

#[inline]
pub unsafe fn wrap_add<W: UdfConverter<U>, U: AggregateUdf>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    _is_null: *mut c_uchar,
    error: *mut c_uchar,
) {
    #[cfg(feature = "logging-debug")]
    debug::pre_add_call::<U>(initid, args, error);

    panic::catch_unwind(|| {
        let cfg = UdfCfg::from_raw_ptr(initid);
        let arglist = ArgList::from_raw_ptr(args);
        let err = *(error as *const Option<NonZeroU8>);
        let mut b = cfg.retrieve_box::<W>();
        let res = U::add(b.as_mut_ref(), cfg, arglist, err);
        cfg.store_box(b);

        if let Err(e) = res {
            *error = e.into();
        }
    })
    .unwrap_or_else(|_| udf_log!(Critical: "add function panicked for `{}`", type_name::<U>()));
}

#[inline]
pub unsafe fn wrap_clear<W: UdfConverter<U>, U: AggregateUdf>(
    initid: *mut UDF_INIT,
    _is_null: *mut c_uchar,
    error: *mut c_uchar,
) {
    #[cfg(feature = "logging-debug")]
    debug::pre_clear_call::<U>(initid, error);

    panic::catch_unwind(|| {
        let cfg = UdfCfg::from_raw_ptr(initid);
        let err = *(error as *const Option<NonZeroU8>);
        let mut b = cfg.retrieve_box::<W>();
        let res = U::clear(b.as_mut_ref(), cfg, err);
        cfg.store_box(b);

        if let Err(e) = res {
            *error = e.into();
        }
    })
    .unwrap_or_else(|_| udf_log!(Critical: "clear function panicked for `{}`", type_name::<U>()));
}

#[inline]
pub unsafe fn wrap_remove<W: UdfConverter<U>, U: AggregateUdf>(
    initid: *mut UDF_INIT,
    args: *mut UDF_ARGS,
    _is_null: *mut c_uchar,
    error: *mut c_uchar,
) {
    #[cfg(feature = "logging-debug")]
    debug::pre_remove_call::<U>(initid, args, error);

    panic::catch_unwind(|| {
        let cfg = UdfCfg::from_raw_ptr(initid);
        let arglist = ArgList::from_raw_ptr(args);
        let err = *(error as *const Option<NonZeroU8>);
        let mut b = cfg.retrieve_box::<W>();
        let res = U::remove(b.as_mut_ref(), cfg, arglist, err);
        cfg.store_box(b);

        if let Err(e) = res {
            *error = e.into();
        }
    })
    .unwrap_or_else(|_| udf_log!(Critical: "remove function panicked for `{}`", type_name::<U>()));
}
