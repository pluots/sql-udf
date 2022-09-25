//! Rust representation of SQL types

use std::slice;
use std::str;

use crate::ffi::bindings::Item_result;
use crate::ffi::{SqlType, SqlTypeTag};

/// A possible SQL result consisting of a type and nullable value
///
/// This enum is similar to [`SqlType`], but actually contains the object.
///
/// It is of note that both [`SqlResult::String`] and [`SqlResult::Decimal`] contain
/// slices of `u8` rather than a representation like `&str`. This is because
/// there is no guarantee that the data is `utf8`.
#[derive(Debug, PartialEq, Clone)]
pub enum SqlResult<'a> {
    // INVALID_RESULT and ROW_RESULT are other options, but not valid for UDFs
    /// A string result
    String(Option<&'a [u8]>),
    /// A floating point result
    Real(Option<f64>),
    /// A nullable integer
    Int(Option<i64>),
    /// This is a string that is to be represented as a decimal
    Decimal(Option<&'a [u8]>),
}

impl<'a> SqlResult<'a> {
    /// Safety: pointer must not be null. If a string or decimal result, must be
    /// exactly `len` long.
    pub(crate) unsafe fn from_ptr(
        ptr: *const u8,
        tag: SqlTypeTag,
        len: usize,
    ) -> Result<SqlResult<'a>, String> {
        // Handle nullptr right away here

        let marker =
            SqlType::try_from(tag).map_err(|_| format!("invalid arg type {tag} received"))?;

        let arg = if ptr.is_null() {
            match marker {
                SqlType::Int => SqlResult::Int(None),
                SqlType::Real => SqlResult::Real(None),
                SqlType::String => SqlResult::String(None),
                SqlType::Decimal => SqlResult::Decimal(None),
            }
        } else {
            // Safety: `tag` guarantees type. If decimal or String, caller
            // guarantees length
            unsafe {
                match marker {
                    SqlType::Int => SqlResult::Int(Some(*(ptr as *const i64))),
                    SqlType::Real => SqlResult::Real(Some(*(ptr as *const f64))),
                    SqlType::String => SqlResult::String(Some(slice::from_raw_parts(ptr, len))),
                    SqlType::Decimal => SqlResult::Decimal(Some(slice::from_raw_parts(ptr, len))),
                }
            }
        };

        Ok(arg)
    }

    /// Simply convert to a string
    ///
    /// Does not distinguish among errors (wrong type, `None` value, or invalid utf8)
    pub fn as_str(&'a self) -> Option<&'a str> {
        match self {
            Self::String(Some(v)) | Self::Decimal(Some(v)) => Some(str::from_utf8(v).ok()?),
            _ => None,
        }
    }

    pub fn as_int(&'a self) -> Option<i64> {
        match self {
            Self::Int(Some(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn as_real(&'a self) -> Option<f64> {
        match self {
            Self::Real(Some(v)) => Some(*v),
            _ => None,
        }
    }
}
