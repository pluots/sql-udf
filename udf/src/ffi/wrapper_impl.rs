//! Private module that handles the implementation of the wrapper module

#![allow(dead_code)]

use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::{c_char, c_longlong, c_uchar, c_ulong};
use std::{ptr, slice, str};

use mysqlclient_sys::MYSQL_ERRMSG_SIZE;

use crate::ffi::bindings::{UDF_ARGS, UDF_INIT};
use crate::ffi::SqlTypeTag;
use crate::{BasicUdf, SqlArg, SqlResult, SqlType, UdfState};

/// Returns an error if a string is not valid UTF-8
///
/// # Panics
///
/// - Receives an invalid arg type
pub unsafe fn process_args<'a, S: UdfState>(
    args: *const UDF_ARGS,
) -> Result<Vec<SqlArg<'a, S>>, String> {
    let mut ret = Vec::new();

    let arg_count: usize;
    let arg_types: &mut [SqlTypeTag];
    let arg_ptrs: &[*const u8];
    let arg_lens: &[u64];
    let maybe_null: &[c_char];
    let attr_ptrs: &[*const u8];
    let attr_lens: &[u64];

    // Load in the C struct
    unsafe {
        // Safety: Caller must ensure that all contents are `arg_count` in length
        arg_count = (*args).arg_count as usize;
        arg_types = dbg!(slice::from_raw_parts_mut((*args).arg_type, arg_count));
        // Load as a u8 rather than i8, assuming that the encoding is utf8 or similar
        // Safety: caller guarantees this slice is valid. Contained pointers may NOT
        // be safe (this is checked later)
        arg_ptrs = slice::from_raw_parts((*args).args as *const *const u8, arg_count);
        arg_lens = slice::from_raw_parts((*args).lengths, arg_count);
        maybe_null = slice::from_raw_parts((*args).maybe_null, arg_count);
        // Same casting to u8 as above
        // Safety: caller guarantees this slice is valid. Contained pointers may NOT
        // be safe (this is checked later)
        attr_ptrs = slice::from_raw_parts((*args).attributes as *const *const u8, arg_count);
        attr_lens = slice::from_raw_parts((*args).attribute_lengths, arg_count);
    }

    // Iterate through all our argument slices
    for (_, a_type, a_ptr, a_len, a_maybe_null, a_attr_ptr, a_attr_len) in arg_types
        .iter_mut()
        .zip(arg_ptrs.iter())
        .zip(arg_lens.iter())
        .zip(maybe_null.iter())
        .zip(attr_ptrs.iter())
        .zip(attr_lens.iter())
        .enumerate()
        // Expand the nested zip tuples, perform a simple deref
        .map(|(i, (((((a, b), c), d), e), f))| (i, a, *b, *c, *d, *e, *f))
    {
        let arg = unsafe { SqlResult::from_ptr(a_ptr, *a_type, a_len as usize)? };

        let attr_slice = unsafe { slice::from_raw_parts(a_attr_ptr, a_attr_len as usize) };

        // Attributes are only ever valid utf8 so we should never expect this
        let attr =
            str::from_utf8(attr_slice).map_err(|e| format!("attribute identifier error: {e}"))?;

        ret.push(SqlArg::<S> {
            arg: arg,
            maybe_null: a_maybe_null != 0,
            attribute: attr,
            type_ptr: a_type as *mut SqlTypeTag,
            marker: PhantomData,
        });
    }

    Ok(ret)
}

/// Write a string message to a buffer. Accepts a const generic size `N` that
/// length of the message will check against (N must be the size of the buffer)
///
/// # Safety
///
/// `N` must be the buffer size. If it is inaccurate, memory safety cannot be
/// guaranteed.
///
/// # Panics
///
/// Panics if the message to be written contains \0 or if the message does not
/// fit in the buffer
pub unsafe fn write_msg_to_buf<const N: usize>(msg: &str, buf: *mut c_char) {
    assert!(msg.len() < N, "internal exception: message overflow");
    assert!(
        !msg.contains('\0'),
        "internal exception: string contains null characters"
    );

    unsafe {
        ptr::copy_nonoverlapping(msg.as_ptr() as *const c_char, buf, msg.bytes().len());
        *buf.add(msg.len()) = 0;
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::{c_int, c_void, CStr};

    use crate::Init;

    use super::*;

    const MSG: &str = "message";
    const BUF_SIZE: usize = MSG.len() + 1;

    #[test]
    fn write_msg_ok() {
        let mut mbuf = [1 as c_char; BUF_SIZE];

        unsafe {
            write_msg_to_buf::<BUF_SIZE>(MSG, mbuf.as_mut_ptr());
            let s = CStr::from_ptr(mbuf.as_ptr()).to_str().unwrap();

            assert_eq!(s, MSG);
        }
    }

    #[test]
    #[should_panic]
    fn write_message_too_long() {
        const NEW_BUF_SIZE: usize = BUF_SIZE - 1;

        let mut mbuf = [1 as c_char; NEW_BUF_SIZE];
        unsafe { write_msg_to_buf::<NEW_BUF_SIZE>(MSG, mbuf.as_mut_ptr()) };
    }

    #[test]
    fn argtype_from_ptr_null() {
        // Just test null pointers here
        unsafe {
            assert_eq!(
                SqlResult::from_ptr(ptr::null(), SqlType::Int as i32, 0),
                Ok(SqlResult::Int(None))
            );
            assert_eq!(
                SqlResult::from_ptr(ptr::null(), SqlType::Real as i32, 0),
                Ok(SqlResult::Real(None))
            );
            assert_eq!(
                SqlResult::from_ptr(ptr::null(), SqlType::String as i32, 0),
                Ok(SqlResult::String(None))
            );
            assert_eq!(
                SqlResult::from_ptr(ptr::null(), SqlType::Decimal as i32, 0),
                Ok(SqlResult::Decimal(None))
            );
            assert!(SqlResult::from_ptr(ptr::null(), -1, 0).is_err());
        }
    }

    #[test]
    fn argtype_from_ptr_notnull() {
        // Just test null pointers here
        unsafe {
            let ival = -1000i64;
            assert_eq!(
                SqlResult::from_ptr(&ival as *const i64 as *const u8, SqlType::Int as i32, 0),
                Ok(SqlResult::Int(Some(ival)))
            );

            let rval = -1000.0f64;
            assert_eq!(
                SqlResult::from_ptr(&rval as *const f64 as *const u8, SqlType::Real as i32, 0),
                Ok(SqlResult::Real(Some(rval)))
            );

            let sval = "this is a string";
            assert_eq!(
                SqlResult::from_ptr(sval.as_ptr(), SqlType::String as i32, sval.len()),
                Ok(SqlResult::String(Some(sval.as_bytes())))
            );

            let dval = "123.456";
            assert_eq!(
                SqlResult::from_ptr(dval.as_ptr(), SqlType::Decimal as i32, dval.len()),
                Ok(SqlResult::Decimal(Some(dval.as_bytes())))
            );

            assert!(SqlResult::from_ptr(dval.as_ptr(), -1, dval.len()).is_err());
        }
    }

    const ARG_COUNT: usize = 4;

    const IVAL: i64 = -1000i64;
    const RVAL: f64 = -1234.5678f64;
    const SVAL: &str = "this is a string";
    const DVAL: &str = "123.456";

    #[test]
    fn process_args_ok() {
        let mut arg_types = [
            SqlType::Int as c_int,
            SqlType::Real as c_int,
            SqlType::String as c_int,
            SqlType::Decimal as c_int,
        ];

        let mut arg_ptrs: [*const u8; ARG_COUNT] = [
            &IVAL as *const i64 as *const u8,
            &RVAL as *const f64 as *const u8,
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
            arg_type: arg_types.as_mut_ptr(),
            args: arg_ptrs.as_mut_ptr() as *mut *mut c_char,
            lengths: arg_lens.as_mut_ptr(),
            maybe_null: maybe_null.as_mut_ptr() as *mut c_char,
            attributes: attr_ptrs.as_mut_ptr() as *mut *mut c_char,
            attribute_lengths: attr_lens.as_mut_ptr() as *mut c_ulong,
            extension: ptr::null_mut::<c_void>(),
        };

        let res = unsafe { process_args::<Init>(&mut udf_args as *mut _) }.unwrap();

        // println!("{res:#?}");

        let expected_args = [
            SqlResult::Int(Some(IVAL)),
            SqlResult::Real(Some(RVAL)),
            SqlResult::String(Some(SVAL.as_bytes())),
            SqlResult::Decimal(Some(DVAL.as_bytes())),
        ];

        for i in 0..ARG_COUNT {
            assert_eq!(res[i].arg, expected_args[i]);
            assert_eq!(res[i].maybe_null, maybe_null[i]);
            assert_eq!(res[i].attribute, attrs[i]);
            assert_eq!(unsafe { *res[i].type_ptr }, arg_types[i]);
        }
    }
}
