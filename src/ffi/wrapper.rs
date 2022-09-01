#![allow(dead_code)]
//! Interface designed to safely wrap rust definitions within C bindings
//!
//! This file ties together C types and rust types, provides a wrapper
//! Everything related to cffi wrapping goes here

use std::os::raw::{c_char, c_longlong, c_uchar, c_ulong};
use std::ffi::CString;
use std::{ptr, slice, str};

use mysqlclient_sys::MYSQL_ERRMSG_SIZE;

use crate::{BasicUdf, InitArgInfo, MaybeArg};
use crate::ffi::bindings::{Item_result, UDF_ARGS, UDF_INIT};
use crate::ffi::item_res;
use crate::types::ConstOpt;




/// Aggregate wrappers - error is a byte, not a pointer!
/// Just store something there if there is an error

// #[udf(name=MY_FUNC)]

/// Returns an error if a string is not valid UTF-8
///
/// # Panics
///
/// - Receives an invalid arg type
fn process_args<'a>(args: *const UDF_ARGS) -> Result<Vec<InitArgInfo<'a>>, String> {
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
                item_res::STRING_RESULT => MaybeArg::String(ConstOpt::NonConst),
                item_res::REAL_RESULT => MaybeArg::Real(ConstOpt::NonConst),
                item_res::INT_RESULT => MaybeArg::Int(ConstOpt::NonConst),
                item_res::DECIMAL_RESULT => MaybeArg::Decimal(ConstOpt::NonConst),
                other => panic!("invalid arg type {} received", other),
            }
        } else {
            // Args are const, so we have access to the values
            let arg_type = arg_types[i];

            if arg_type == item_res::STRING_RESULT || arg_type == item_res::DECIMAL_RESULT {
                // String and decimal are both string-like

                // Safety: we have already checked for null, caller guarantees lengths
                let bytearr =
                    unsafe { slice::from_raw_parts(arg_ptrs[i], arg_lengths[i] as usize) };
                let const_str = match str::from_utf8(bytearr) {
                    Ok(s) => s,
                    Err(_) => return Err("invalid utf8".to_owned()),
                };
                match arg_type {
                    item_res::STRING_RESULT => MaybeArg::String(ConstOpt::Const(const_str)),
                    item_res::DECIMAL_RESULT => MaybeArg::Decimal(ConstOpt::Const(const_str)),
                    _ => unreachable!(),
                }
            } else if arg_type == item_res::INT_RESULT || arg_type == item_res::REAL_RESULT {
                // Safety: both sql ints and reals are 64 bit
                // let bytearr: [u8; 8] = unsafe{slice::from_raw_parts(arg_ptrs[i], 8)};

                match arg_type {
                    item_res::INT_RESULT => {
                        MaybeArg::Int(ConstOpt::Const(unsafe { *(arg_ptrs[i] as *const i64) }))
                    }
                    item_res::REAL_RESULT => {
                        MaybeArg::Real(ConstOpt::Const(unsafe { *(arg_ptrs[i] as *const f64) }))
                    }
                    _ => unreachable!(),
                }
            } else {
                panic!("invalid arg type {} received", arg_type)
            }
        };

        let bytearr = unsafe { slice::from_raw_parts(attr_ptrs[i], attr_lengths[i] as usize) };
        let attr: &str = str::from_utf8(bytearr).expect("attribute is not valid utf-8");

        ret.push(InitArgInfo {
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
pub unsafe fn init_wrapper<T: BasicUdf>(
    initid: *mut UDF_INIT,
    args: *const UDF_ARGS,
    message: *mut c_char,
) -> bool {
    let args = match process_args(args) {
        Ok(v) => v,
        Err(e) => {
            // Safety: we know our messages are short enough to fit in the buffer
            unsafe { write_buf_unchecked(&e, message) };
            return true;
        }
    };

    // Set max_length, maybe_null, const_item, decimals from proc macro

    // If initialization fails, copy a message to the buffer
    let udf_struct = match T::init(&args) {
        Ok(v) => Box::new(v),
        Err(e) => {
            // Message must be strictly smaller than the buffer to leave room for
            // the null terminator
            assert!(
                (e.len() as u32) < MYSQL_ERRMSG_SIZE,
                "internal exception: error message too long"
            );
            // Safety: we have checked that our message fits in the buffer
            unsafe { write_buf_unchecked(&e, message) };
            return true;
        }
    };

    // Put the struct on the heap and get a pointer
    let box_ptr = Box::into_raw(udf_struct);
    // Safety: Must be cleaned up in deinit function, or will leak!
    unsafe { (*initid).ptr = box_ptr as *mut c_char };

    false
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

/// For our deinit function, all we need to do is take ownership of the
/// value on the stack. The function ends, it goes out of scope and gets
/// dropped.
unsafe fn deinit_wrapper<T: BasicUdf>(initid: *mut UDF_INIT) {
    // Safety: we constructed this box so it is formatted correctly
    unsafe { Box::from_raw((*initid).ptr as *mut T) };
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
unsafe fn write_buf_unchecked(msg: &str, buf: *mut c_char) {
    let cstr = CString::new(msg).expect("internal exception: string contains null characters");

    unsafe { ptr::copy_nonoverlapping(cstr.as_ptr(), buf, cstr.as_bytes_with_nul().len()) };
}


#[cfg(test)]
mod tests {
    use crate::{InitArgInfo, BasicUdf, ArgInfo, register};


    // #[crate::register(a1, a2=banana)]
    struct MyUdf {
        // v: Vec<u8>,
    }

    impl BasicUdf for MyUdf {
        type Returns =  String;

        fn init(args: &[InitArgInfo]) -> Result<Self, String>
        where
            Self: Sized,
        {
            todo!()
        }

        fn process<'a>(&self, args: &[ArgInfo]) -> Result<String, String> {
            todo!()
        }
    }


    fn test1() {}
}
