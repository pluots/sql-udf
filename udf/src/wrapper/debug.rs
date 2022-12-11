//! Logging helpers

#![cfg(feature = "logging-debug")]

use std::any::type_name;
use std::ffi::{c_char, c_uchar, c_ulong};

use cfg_if::cfg_if;
use udf_sys::{UDF_ARGS, UDF_INIT};

use crate::udf_log;

pub unsafe fn pre_init_call<T>(
    initid: *const UDF_INIT,
    args: *const UDF_ARGS,
    _message: *const c_char,
) {
    udf_log!(Debug: "entering init for `{}`", type_name::<T>());

    cfg_if! {
        if  #[cfg(feature = "logging-debug-calls")] {
            udf_log!(Debug: "Data receive state at init:");
            dbg!(&*initid);
            dbg!(&*args);
        }
    }
}

pub unsafe fn post_init_call<T>(
    initid: *const UDF_INIT,
    _args: *const UDF_ARGS,
    _message: *const c_char,
    ret: bool,
) {
    cfg_if! {
        if  #[cfg(feature = "logging-debug-calls")] {
            udf_log!(Debug: "Data return state at init:");
            dbg!(&*initid);
            eprintln!("Returning {ret:?}");
        }
    }

    udf_log!(Debug: "exiting init for `{}`", type_name::<T>());
}

pub unsafe fn pre_deinit_call<T>(initid: *const UDF_INIT) {
    udf_log!(Debug: "entering deinit for `{}`", type_name::<T>());

    cfg_if! {
        if  #[cfg(feature = "logging-debug-calls")] {
            udf_log!(Debug: "Data receive state at deinit:");
            dbg!(&*initid);
        }
    }
}

pub unsafe fn pre_add_call<T>(
    initid: *const UDF_INIT,
    args: *const UDF_ARGS,
    error: *const c_uchar,
) {
    udf_log!(Debug: "entering add for `{}`", type_name::<T>());

    cfg_if! {
        if  #[cfg(feature = "logging-debug-calls")] {
            udf_log!(Debug: "Data receive state at add:");
            dbg!(&*initid);
            dbg!(&*args);
            dbg!(&*error);
        }
    }
}

pub unsafe fn pre_clear_call<T>(initid: *const UDF_INIT, error: *const c_uchar) {
    udf_log!(Debug: "entering clear for `{}`", type_name::<T>());

    cfg_if! {
        if  #[cfg(feature = "logging-debug-calls")] {
            udf_log!(Debug: "Data receive state at clear:");
            dbg!(&*initid);
            dbg!(&*error);
        }
    }
}

pub unsafe fn pre_remove_call<T>(
    initid: *const UDF_INIT,
    args: *const UDF_ARGS,
    error: *const c_uchar,
) {
    udf_log!(Debug: "entering remove for `{}`", type_name::<T>());

    cfg_if! {
        if  #[cfg(feature = "logging-debug-calls")] {
            udf_log!(Debug: "Data receive state at remove:");
            dbg!(&*initid);
            dbg!(&*args);
            dbg!(&*error);
        }
    }
}

pub unsafe fn pre_process_call<T>(
    initid: *const UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *const c_uchar,
    error: *const c_uchar,
) {
    udf_log!(Debug: "entering process for `{}`", type_name::<T>());

    cfg_if! {
        if  #[cfg(feature = "logging-debug-calls")] {
            udf_log!(Debug: "Data receive state at process:");
            dbg!(&*initid);
            dbg!(&*args);
            dbg!(&*is_null);
            dbg!(&*error);
        }
    }
}

pub unsafe fn post_process_call<T>(
    initid: *const UDF_INIT,
    args: *const UDF_ARGS,
    is_null: *const c_uchar,
    error: *const c_uchar,
) {
    udf_log!(Debug: "exiting process for `{}`", type_name::<T>());

    cfg_if! {
        if  #[cfg(feature = "logging-debug-calls")] {
            udf_log!(Debug: "Data return state at process:");
            dbg!(&*initid);
            dbg!(&*args);
            dbg!(&*is_null);
            dbg!(&*error);
        }
    }
}

pub unsafe fn pre_process_call_buf<T>(
    initid: *const UDF_INIT,
    args: *const UDF_ARGS,
    result: *const c_char,
    length: *const c_ulong,
    is_null: *const c_uchar,
    error: *const c_uchar,
) {
    udf_log!(Debug: "entering process for `{}`", type_name::<T>());

    cfg_if! {
        if  #[cfg(feature = "logging-debug-calls")] {
            udf_log!(Debug: "Data receive state at process:");
            dbg!(&*initid);
            dbg!(&*args);
            dbg!(result);
            dbg!(&*length);
            dbg!(&*is_null);
            dbg!(&*error);
        }
    }
}

pub unsafe fn post_process_call_buf<T>(
    initid: *const UDF_INIT,
    args: *const UDF_ARGS,
    result: *const c_char,
    length: *const c_ulong,
    is_null: *const c_uchar,
    error: *const c_uchar,
    ret: *const c_char,
) {
    udf_log!(Debug: "exiting process for `{}`", type_name::<T>());

    cfg_if! {
        if  #[cfg(feature = "logging-debug-calls")] {
            udf_log!(Debug: "Data return state at process:");
            dbg!(&*initid);
            dbg!(&*args);
            dbg!(result);
            dbg!(&*length);
            dbg!(&*is_null);
            dbg!(&*error);
            dbg!(ret);
        }
    }
}
