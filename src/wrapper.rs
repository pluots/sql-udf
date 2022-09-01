//! Interface
//!
//!
//! This file ties together C types and rust types, provides a wrapper
//! Everything related to cffi wrapping goes here

use crate::udf_types::item_res;
use crate::udf_types::ConstOpt;
use crate::udf_types_c::{Item_result, UDF_ARGS, UDF_INIT};
use std::ffi::CString;
use std::os::raw::c_char;
use std::str;
use std::{ptr, slice};

use mysqlclient_sys::MYSQL_ERRMSG_SIZE;

use crate::{InitArg, UdfArg};

// From the MySQL docs, the init function has the following purposes:
//
// - To check the number of arguments to XXX().
// - To verify that the arguments are of a required type or, alternatively, to
//   tell the server to coerce arguments to the required types when the main
//   function is called.
// - To allocate any memory required by the main function.
// - To specify the maximum length of the result.
// - To specify (for REAL functions) the maximum number of decimal places in
//   the result.
// - To specify whether the result can be NULL.

struct MyUdf {
    v: Vec<u8>,
}

impl MyUdf {
    // #[udf(maybe_null)]
    fn init() -> Result<Self, String> {
        Ok(MyUdf { v: Vec::new() })
    }
}

/// Returns an error if a string is not valid UTF-8
///
/// # Panics
///
/// - Receives an invalid arg type
fn process_args<'a>(args: *mut UDF_ARGS) -> Result<Vec<UdfArg<'a>>, String> {
    let mut ret = Vec::new();

    let arg_count: usize;
    let arg_types: &[Item_result];
    let arg_ptrs: &[*const u8];
    let arg_lengths: &[u64];
    let maybe_null: &[c_char];
    let attr_ptrs: &[*const u8];
    let attr_lengths: &[u64];

    // Load in the C struct
    unsafe {
        // Safety: Caller must ensure that all contents are `arg_count` in length
        arg_count = (*args).arg_count as usize;
        arg_types = slice::from_raw_parts((*args).arg_type, arg_count);
        // Load as a u8 rather than i8, assuming that the encoding is utf8 or similar
        // TODO: look more into UDF metadata
        // Safety: these pointers may not be valid. We check that later.
        arg_ptrs = slice::from_raw_parts((*args).args as *const *const u8, arg_count);
        arg_lengths = slice::from_raw_parts((*args).lengths, arg_count);
        maybe_null = slice::from_raw_parts((*args).maybe_null, arg_count);
        // Same casting to u8 as above
        // Safety: these pointers may not be valid. We check that later.
        attr_ptrs = slice::from_raw_parts((*args).attributes as *const *const u8, arg_count);
        attr_lengths = slice::from_raw_parts((*args).attribute_lengths, arg_count);
    }

    for i in 0..arg_count {
        // for const args, args->args[i] is the value
        // for nonconst args, args->args[i] is 0
        let arg_enum = if arg_ptrs[i].is_null() {
            // Args are not const, so we can't check values
            match arg_types[i] {
                item_res::STRING_RESULT => InitArg::String(ConstOpt::NonConst),
                item_res::REAL_RESULT => InitArg::Real(ConstOpt::NonConst),
                item_res::INT_RESULT => InitArg::Int(ConstOpt::NonConst),
                item_res::DECIMAL_RESULT => InitArg::Decimal(ConstOpt::NonConst),
                other => panic!("invalid arg type {} received", other),
            }
        } else {
            // Args are const, so we have access to the values
            let arg_type = arg_types[i];

            if arg_type == item_res::STRING_RESULT || arg_type == item_res::DECIMAL_RESULT {
                // String and decimal are both string-like

                // Safety: we have already checked for null, caller guarantees
                // lengths
                let bytearr =
                    unsafe { slice::from_raw_parts(arg_ptrs[i], arg_lengths[i] as usize) };
                let const_str = match str::from_utf8(bytearr) {
                    Ok(s) => s,
                    Err(_) => return Err("invalid utf8".to_owned()),
                };
                match arg_type {
                    item_res::STRING_RESULT => InitArg::String(ConstOpt::Const(const_str)),
                    item_res::DECIMAL_RESULT => InitArg::Decimal(ConstOpt::Const(const_str)),
                    _ => unreachable!(),
                }
            } else if arg_type == item_res::INT_RESULT || arg_type == item_res::REAL_RESULT {
                // Safety: both sql ints and reals are 64 bit
                // let bytearr: [u8; 8] = unsafe{slice::from_raw_parts(arg_ptrs[i], 8)};

                match arg_type {
                    item_res::INT_RESULT => {
                        InitArg::Int(ConstOpt::Const(unsafe { *(arg_ptrs[i] as *const i64) }))
                    }
                    item_res::REAL_RESULT => {
                        InitArg::Real(ConstOpt::Const(unsafe { *(arg_ptrs[i] as *const f64) }))
                    }
                    _ => unreachable!(),
                }
            } else {
                panic!("invalid arg type {} received", arg_type)
            }
        };

        let bytearr = unsafe { slice::from_raw_parts(attr_ptrs[i], attr_lengths[i] as usize) };
        let attr: &str = str::from_utf8(bytearr).expect("attribute is not valid utf-8");

        ret.push(UdfArg {
            arg: arg_enum,
            maybe_null: maybe_null[i] != 0,
            attribute: attr,
        })
    }

    Ok(ret)
}

/// Return true if there is an error
///
/// # Arguments
///
/// - initd:
/// - args:
///
/// # Sets
///
/// - initid.max_length
/// - initd.maybe_null
/// - initd.decimals
///
/// # Panics
///
/// - Panics if the error message contains "\0", or if the message is too long (
///   greater than 511 bytes).
/// - Panics if the provides error message string contains null characters
fn udf_func_init(initid: *mut UDF_INIT, args: *mut UDF_ARGS, message: *mut c_char) -> bool {
    // If initialization fails, copy a message to the buffer
    let udf_struct = match MyUdf::init() {
        Ok(v) => Box::new(v),
        Err(e) => {
            // Message must be strictly smaller than the buffer to leave room for
            // the null terminator
            assert!(
                (e.len() as u32) < MYSQL_ERRMSG_SIZE,
                "internal exception: error message too long"
            );
            // Safety: we have checked that our message fits in the buffer
            unsafe { write_msg(&e, message) };
            return true;
        }
    };

    Box::into_raw(udf_struct);

    let args = match process_args(args) {
        Ok(v) => v,
        Err(e) => {
            unsafe { write_msg(&e, message) };
            return true;
        }
    };

    // Function needs to set:
    //

    // initid.

    // if (*args).arg_count != 1 {
    //     write_result(message, b"blake3_hash must have one argument");
    //     return true;
    // }
    // *((*args).arg_type) = Item_result_STRING_RESULT;
    // (*initid).maybe_null = true;

    false
}

/// Write a string message to a buffer
///
/// # Safety
///
/// It must be checked that the message fits in the buffer _with a null terminator_.
/// I.e., msg.len() < buf_size.
///
/// # Panics
///
/// Panics if the message to be written contains \0
unsafe fn write_msg(msg: &str, buf: *mut c_char) {
    let cstr = CString::new(msg).expect("internal exception: string contains null characters");

    ptr::copy_nonoverlapping(cstr.as_ptr(), buf, cstr.as_bytes_with_nul().len());
}
