//! Wrapper
//!
//! Everything related to cffi wrapping goes here

use crate::udf_types_c::{UDF_ARGS, UDF_INIT};
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use mysqlclient_sys::MYSQL_ERRMSG_SIZE;

use crate::UdfInt;

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
    fn init(args: []) -> Result<Self, String> {
        Ok(MyUdf { v: Vec::new() })
    }
}

/// Return true if there is an error
/// 
/// # Arguments
/// 
/// - 
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
            let cstr =
                CString::new(e).expect("internal exception: string contains null characters");

            // Safety: we have checked that our message fits in the buffer
            // as_ptr() is valid for the internal length (with null)
            unsafe {
                ptr::copy_nonoverlapping(cstr.as_ptr(), message, cstr.as_bytes_with_nul().len());
            }

            return true;
        }
    };

    Box::into_raw(udf_struct);

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
