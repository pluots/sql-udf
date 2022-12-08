//! Private module that handles the implementation of the wrapper module

// #![allow(dead_code)]

use std::any::type_name;
use std::cmp::min;
use std::ffi::{c_char, c_ulong};
use std::ptr;

use crate::udf_log;

/// Write a string message to a buffer. Accepts a const generic size `N` that
/// length of the message will check against (N must be the size of the buffer)
///
/// # Safety
///
/// `N` must be the buffer size. If it is inaccurate, memory safety cannot be
/// guaranteed.
///
/// This is public within the crate, since the parent model is not public
pub unsafe fn write_msg_to_buf<const N: usize>(msg: &[u8], buf: *mut c_char) {
    // message plus null terminator must fit in buffer
    let bytes_to_write = min(msg.len(), N - 1);

    unsafe {
        ptr::copy_nonoverlapping(msg.as_ptr().cast::<c_char>(), buf, bytes_to_write);
        *buf.add(bytes_to_write) = 0;
    }
}

/// Data that is only relevant to buffer return types
pub struct BufOptions {
    res_buf: *mut c_char,
    length: *mut c_ulong,
    /// True if we can return a reference to our source buffer. If false, we must
    /// truncate
    can_return_ref: bool,
}

impl BufOptions {
    /// Create a new `BufOptions` struct
    pub fn new(res_buf: *mut c_char, length: *mut c_ulong, can_return_ref: bool) -> Self {
        Self {
            res_buf,
            length,
            can_return_ref,
        }
    }
}

/// Handle the result of SQL function that returns a buffer
///
/// Accept any input byte slice and a set of buffer options. Performs one of
/// three:
///
/// - If slice fits in buffer: copy to buffer, return pointer to the buffer
/// - If slice does not fit in the buffer and returning references are
///   permitted: return pointer to the source slice
/// - If slice does not fit and returning references is not permitted: print
///   an error message, return None
///
/// The `U` type parameter is just used for output formatting
pub unsafe fn buf_result_callback<U, T: AsRef<[u8]>>(
    input: T,
    opts: &BufOptions,
) -> Option<*const c_char> {
    let slice_ref = input.as_ref();
    let slice_len = slice_ref.len();
    let slice_ptr: *const c_char = slice_ref.as_ptr().cast();
    let buf_len = *opts.length as usize;

    if slice_len <= buf_len {
        // If we fit in the buffer, just copy
        ptr::copy(slice_ptr, opts.res_buf, slice_len);
        *opts.length = slice_len as u64;
        return Some(opts.res_buf);
    }

    if !opts.can_return_ref {
        // We can't return a reference but also can't fit in the buffer
        *opts.length = 0;
        udf_log!(
            Critical: "output overflow, returning NULL. Buffer size: {}, data length: {} at `{}::process`",
            buf_len, slice_len, type_name::<U>()
        );
        udf_log!(Critical: "contact your UDF vendor as this is a serious bug");
        return None;
    }

    // If we don't fit in the buffer but can return a reference, do so
    *opts.length = slice_len as u64;
    Some(slice_ptr)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::similar_names)]

    use std::ffi::{c_ulong, c_void, CStr};
    use std::ptr;

    use udf_sys::{Item_result, UDF_ARGS};

    use super::*;
    use crate::prelude::*;

    const MSG: &str = "message";
    const BUF_SIZE: usize = MSG.len() + 1;

    #[test]
    fn write_msg_ok() {
        let mut mbuf = [1 as c_char; BUF_SIZE];

        unsafe {
            write_msg_to_buf::<BUF_SIZE>(MSG.as_bytes(), mbuf.as_mut_ptr());
            let s = CStr::from_ptr(mbuf.as_ptr()).to_str().unwrap();

            assert_eq!(s, MSG);
        }
    }

    #[test]
    fn write_message_too_long() {
        const NEW_BUF_SIZE: usize = BUF_SIZE - 1;

        let mut mbuf = [1 as c_char; NEW_BUF_SIZE];
        unsafe {
            write_msg_to_buf::<NEW_BUF_SIZE>(MSG.as_bytes(), mbuf.as_mut_ptr());
            let s = CStr::from_ptr(mbuf.as_ptr()).to_str().unwrap();
            assert_eq!(*s, MSG[..MSG.len() - 1]);
        };
    }

    #[test]
    fn argtype_from_ptr_null() {
        // Just test null pointers here
        unsafe {
            assert_eq!(
                SqlResult::from_ptr(ptr::null(), Item_result::INT_RESULT, 0),
                Ok(SqlResult::Int(None))
            );
            assert_eq!(
                SqlResult::from_ptr(ptr::null(), Item_result::REAL_RESULT, 0),
                Ok(SqlResult::Real(None))
            );
            assert_eq!(
                SqlResult::from_ptr(ptr::null(), Item_result::STRING_RESULT, 0),
                Ok(SqlResult::String(None))
            );
            assert_eq!(
                SqlResult::from_ptr(ptr::null(), Item_result::DECIMAL_RESULT, 0),
                Ok(SqlResult::Decimal(None))
            );
            assert!(SqlResult::from_ptr(ptr::null(), Item_result::INVALID_RESULT, 0).is_err());
        }
    }

    #[test]
    fn argtype_from_ptr_notnull() {
        // Just test null pointers here
        unsafe {
            let ival = -1000i64;
            assert_eq!(
                SqlResult::from_ptr(ptr::addr_of!(ival).cast(), Item_result::INT_RESULT, 0),
                Ok(SqlResult::Int(Some(ival)))
            );

            let rval = -1000.0f64;
            assert_eq!(
                SqlResult::from_ptr(ptr::addr_of!(rval).cast(), Item_result::REAL_RESULT, 0),
                Ok(SqlResult::Real(Some(rval)))
            );

            let sval = "this is a string";
            assert_eq!(
                SqlResult::from_ptr(sval.as_ptr(), Item_result::STRING_RESULT, sval.len()),
                Ok(SqlResult::String(Some(sval.as_bytes())))
            );

            let dval = "123.456";
            assert_eq!(
                SqlResult::from_ptr(dval.as_ptr(), Item_result::DECIMAL_RESULT, dval.len()),
                Ok(SqlResult::Decimal(Some(dval)))
            );

            assert!(
                SqlResult::from_ptr(dval.as_ptr(), Item_result::INVALID_RESULT, dval.len())
                    .is_err()
            );
        }
    }

    const ARG_COUNT: usize = 4;

    static IVAL: i64 = -1000i64;
    static RVAL: f64 = -1234.5678f64;
    static SVAL: &str = "this is a string";
    static DVAL: &str = "123.456";

    #[test]
    fn process_args_ok() {
        let mut arg_types = [
            Item_result::INT_RESULT,
            Item_result::REAL_RESULT,
            Item_result::STRING_RESULT,
            Item_result::DECIMAL_RESULT,
        ];

        let mut arg_ptrs: [*const u8; ARG_COUNT] = [
            ptr::addr_of!(IVAL).cast(),
            ptr::addr_of!(RVAL).cast(),
            SVAL.as_ptr(),
            DVAL.as_ptr(),
        ];

        let mut arg_lens = [0u64, 0, SVAL.len() as u64, DVAL.len() as u64];
        let mut maybe_null = [true, true, false, false];
        let attrs = ["ival", "rval", "sval", "dval"];
        let mut attr_ptrs = [
            attrs[0].as_ptr(),
            attrs[1].as_ptr(),
            attrs[2].as_ptr(),
            attrs[3].as_ptr(),
        ];
        let mut attr_lens = [
            attrs[0].len(),
            attrs[1].len(),
            attrs[2].len(),
            attrs[3].len(),
        ];

        let mut udf_args = UDF_ARGS {
            arg_count: ARG_COUNT as u32,
            arg_types: arg_types.as_mut_ptr(),
            args: arg_ptrs.as_mut_ptr() as *const *const c_char,
            lengths: arg_lens.as_mut_ptr(),
            maybe_null: maybe_null.as_mut_ptr() as *const c_char,
            attributes: attr_ptrs.as_mut_ptr() as *const *const c_char,
            attribute_lengths: attr_lens.as_mut_ptr() as *const c_ulong,
            extension: ptr::null_mut::<c_void>(),
        };

        let arglist: &ArgList<Init> = unsafe { ArgList::from_raw_ptr(&mut udf_args) };
        let res: Vec<_> = arglist.into_iter().collect();

        let expected_args = [
            SqlResult::Int(Some(IVAL)),
            SqlResult::Real(Some(RVAL)),
            SqlResult::String(Some(SVAL.as_bytes())),
            SqlResult::Decimal(Some(DVAL)),
        ];

        for i in 0..ARG_COUNT {
            assert_eq!(res[i].value(), expected_args[i]);
            assert_eq!(res[i].maybe_null(), maybe_null[i]);
            assert_eq!(res[i].attribute(), attrs[i]);
            // assert_eq!(unsafe { *res[i].type_ptr }, arg_types[i]);
        }
    }
}

#[cfg(test)]
mod buffer_tests {
    use core::slice;

    use super::*;

    const BUF_LEN: usize = 10;

    #[test]
    fn test_buf_fits() {
        // Test a buffer that simply fits into the available result buffer
        let input = b"1234";
        let mut res_buf = [0u8; BUF_LEN];
        let zeroes = [0u8; BUF_LEN];
        let mut len = res_buf.len() as u64;
        let buf_opts = BufOptions::new(res_buf.as_mut_ptr().cast(), &mut len, false);

        let res_ptr: *const u8 = unsafe { buf_result_callback::<u8, _>(input, &buf_opts) }
            .unwrap()
            .cast();
        let res_slice = unsafe { slice::from_raw_parts(res_ptr, len as usize) };

        assert_eq!(len as usize, input.len());
        assert_eq!(res_slice, input);
        assert_eq!(res_ptr.cast(), res_buf.as_ptr());
        // Check residual buffer
        assert_eq!(
            res_buf[input.len()..res_buf.len()],
            zeroes[input.len()..res_buf.len()]
        );
    }

    #[test]
    fn test_buf_no_fit_ref() {
        // Test a buffer that does not fit but can be used as a ref
        let input = b"123456789012345";
        let mut res_buf = [0u8; BUF_LEN];
        let mut len = res_buf.len() as u64;
        let buf_opts = BufOptions::new(res_buf.as_mut_ptr().cast(), &mut len, true);

        let res_ptr: *const u8 = unsafe { buf_result_callback::<u8, _>(input, &buf_opts) }
            .unwrap()
            .cast();
        let res_slice = unsafe { slice::from_raw_parts(res_ptr, len as usize) };

        assert_eq!(len as usize, input.len());
        assert_eq!(res_slice, input);
        assert_eq!(res_ptr.cast(), input.as_ptr());
    }

    #[test]
    fn test_buf_no_fit_no_ref() {
        // Test a buffer that does not fit but can not be used as a ref
        // This must return an error
        let input = b"123456789012345";
        let mut res_buf = [0u8; BUF_LEN];
        let mut len = res_buf.len() as u64;
        let buf_opts = BufOptions::new(res_buf.as_mut_ptr().cast(), &mut len, false);

        let res = unsafe { buf_result_callback::<u8, _>(input, &buf_opts) };
        assert_eq!(len, 0);
        assert_eq!(res, None);
    }
}
