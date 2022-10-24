//! Private module that handles the implementation of the wrapper module

#![allow(dead_code)]

use std::cmp::min;
use std::ffi::CString;
use std::marker::PhantomData;
use std::num::NonZeroU8;
use std::os::raw::{c_char, c_longlong, c_uchar, c_ulong};
use std::{ptr, slice, str};

use crate::ffi::bindings::{UDF_ARGS, UDF_INIT};
use crate::ffi::SqlTypeTag;
use crate::{ArgList, BasicUdf, ProcessError, SqlArg, SqlResult, SqlType, UdfState};

/// Add methods to the raw C struct
impl UDF_INIT {
    /// Consume a box and store its pointer in this `UDF_INIT`
    ///
    /// After calling this function, the caller is responsible for
    /// cleaning up the
    pub(crate) fn store_box<T>(&mut self, b: Box<T>) {
        let box_ptr = Box::into_raw(b);
        self.ptr = box_ptr.cast::<c_char>();
    }

    /// Given a generic type T, assume
    ///
    /// Safety: T _must_ be the type of this pointer
    #[allow(unsafe_op_in_unsafe_fn)]
    pub(crate) unsafe fn retrieve_box<T>(&self) -> Box<T> {
        Box::from_raw(self.ptr.cast::<T>())
    }
}

/// Write a string message to a buffer. Accepts a const generic size `N` that
/// length of the message will check against (N must be the size of the buffer)
///
/// # Safety
///
/// `N` must be the buffer size. If it is inaccurate, memory safety cannot be
/// guaranteed.
pub(crate) unsafe fn write_msg_to_buf<const N: usize>(msg: &[u8], buf: *mut c_char) {
    // message plus null terminator must fit in buffer
    let bytes_to_write = min(msg.len(), N - 1);

    unsafe {
        ptr::copy_nonoverlapping(msg.as_ptr().cast::<c_char>(), buf, bytes_to_write);
        *buf.add(bytes_to_write) = 0;
    }
}

// WIP
// pub unsafe fn handle_panic_res<const N: usize>(e: Box<dyn Any + Send>,buf: *mut c_char) {
//     // message plus null terminator must fit in buffer
//     let bytes_to_write = min(msg.len(), N - 1);

//     unsafe {
//         ptr::copy_nonoverlapping(msg.as_ptr().cast::<c_char>(), buf, bytes_to_write);
//         *buf.add(bytes_to_write) = 0;
//     }
// }

// pub unsafe fn write_panic_res_to_buf<const N: usize>(msg: &str, buf: *mut c_char) {
//     // message plus null terminator must fit in buffer
//     let bytes_to_write = min(msg.len(), N - 1);

//     unsafe {
//         ptr::copy_nonoverlapping(msg.as_ptr().cast::<c_char>(), buf, bytes_to_write);
//         *buf.add(bytes_to_write) = 0;
//     }
// }

#[cfg(test)]
mod tests {
    use std::ffi::{c_int, c_void, CStr};

    use super::*;
    use crate::types::ArgList;
    use crate::Init;

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

        let udf_args = UDF_ARGS {
            arg_count: ARG_COUNT as u32,
            arg_type: arg_types.as_mut_ptr(),
            args: arg_ptrs.as_mut_ptr() as *const *const c_char,
            lengths: arg_lens.as_mut_ptr(),
            maybe_null: maybe_null.as_mut_ptr() as *mut c_char,
            attributes: attr_ptrs.as_mut_ptr() as *const *const c_char,
            attribute_lengths: attr_lens.as_mut_ptr() as *mut c_ulong,
            extension: ptr::null_mut::<c_void>(),
        };

        let arglist: &ArgList<Init> = unsafe { ArgList::from_arg_ptr(&udf_args) };
        let res: Vec<_> = arglist.into_iter().collect();

        let expected_args = [
            SqlResult::Int(Some(IVAL)),
            SqlResult::Real(Some(RVAL)),
            SqlResult::String(Some(SVAL.as_bytes())),
            SqlResult::Decimal(Some(DVAL.as_bytes())),
        ];

        for i in 0..ARG_COUNT {
            assert_eq!(res[i].value, expected_args[i]);
            assert_eq!(res[i].maybe_null, maybe_null[i]);
            assert_eq!(res[i].attribute, attrs[i]);
            // assert_eq!(unsafe { *res[i].type_ptr }, arg_types[i]);
        }
    }
}
