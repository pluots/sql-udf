//! Helpers for parsing args simply
use std::str;

use crate::{SqlResult, SqlType};

// OPTIOANL ARGS specify these in the proc macro signature
pub enum Error<'res> {
    InvalidType(SqlType),
    UnexpectedNull,
    Utf8(&'res [u8], str::Utf8Error),
}

/// Types with this trait can easily be used as an argument
pub trait SqlArg<'res>: Sized {
    /// How to set argument coercion
    const COERCE_TYPE: SqlType;

    fn from_res(value: SqlResult<'res>) -> Result<Self, Error>;
}

impl<'res> SqlArg<'res> for &'res str {
    const COERCE_TYPE: SqlType = SqlType::String;

    fn from_res(value: SqlResult<'res>) -> Result<Self, Error> {
        match value {
            SqlResult::String(Some(v)) => Ok(str::from_utf8(v).map_err(|e| Error::Utf8(v, e))?),
            SqlResult::Decimal(Some(s)) => Ok(s),
            SqlResult::String(None) | SqlResult::Decimal(None) => todo!(),
            SqlResult::Real(_) | SqlResult::Int(_) => Err(Error::InvalidType(value.as_type())),
        }
    }
}

impl<'res> SqlArg<'res> for i64 {
    const COERCE_TYPE: SqlType = SqlType::Int;

    fn from_res(value: SqlResult<'res>) -> Result<Self, Error> {
        match value {
            SqlResult::Int(Some(v)) => Ok(v),
            SqlResult::Int(None) => Err(Error::UnexpectedNull),
            SqlResult::String(_) | SqlResult::Decimal(_) | SqlResult::Real(_) => {
                Err(Error::InvalidType(value.as_type()))
            }
        }
    }
}

impl<'res> SqlArg<'res> for f64 {
    const COERCE_TYPE: SqlType = SqlType::Real;

    fn from_res(value: SqlResult<'res>) -> Result<Self, Error> {
        match value {
            SqlResult::Real(Some(v)) => Ok(v),
            SqlResult::Real(None) => Err(Error::UnexpectedNull),
            SqlResult::String(_) | SqlResult::Decimal(_) | SqlResult::Int(_) => {
                Err(Error::InvalidType(value.as_type()))
            }
        }
    }
}

impl<'res, T: SqlArg<'res>> SqlArg<'res> for Option<T> {
    const COERCE_TYPE: SqlType = T::COERCE_TYPE;

    fn from_res(value: SqlResult<'res>) -> Result<Self, Error> {
        if value.is_null() {
            Ok(None)
        } else {
            Ok(Some(T::from_res(value)?))
        }
    }
}
